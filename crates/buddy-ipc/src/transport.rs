use crate::{Endpoint, endpoint};
use buddy_core::{
    Engine,
    notifications::{Delivery, MAX_WIRE_BYTES, Notification, NotificationQueue},
};
use interprocess::local_socket::tokio::{Listener, Stream, prelude::*};
#[cfg(unix)]
use interprocess::local_socket::{ListenerOptions, traits::StreamCommon};
use serde::{Deserialize, Serialize};
use std::{
    fmt, io,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    runtime::Builder,
    sync::oneshot,
    time::timeout,
};
const IO_TIMEOUT: Duration = Duration::from_millis(1000);
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ack {
    pub v: u8,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl Ack {
    fn result(result: Result<(), String>) -> Self {
        Self {
            v: 1,
            ok: result.is_ok(),
            error: result.err(),
        }
    }
}
#[derive(Debug)]
pub enum ClientError {
    Invalid(String),
    Unavailable(io::Error),
    Rejected(String),
    Timeout,
}
impl ClientError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Invalid(_) => 2,
            Self::Unavailable(_) => 3,
            Self::Rejected(_) => 4,
            Self::Timeout => 5,
        }
    }
}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(e) => write!(f, "Ungültige Meldung: {e}"),
            Self::Unavailable(e) => write!(
                f,
                "Beni empfängt nicht. App starten und Statusmeldungen aktivieren: {e}"
            ),
            Self::Rejected(e) => write!(f, "Meldung abgelehnt: {e}"),
            Self::Timeout => write!(f, "Beni antwortet nicht rechtzeitig"),
        }
    }
}
impl std::error::Error for ClientError {}
fn runtime() -> io::Result<tokio::runtime::Runtime> {
    Builder::new_current_thread().enable_all().build()
}
async fn read_line(stream: &mut Stream) -> io::Result<Vec<u8>> {
    let mut reader = BufReader::new(stream.take((MAX_WIRE_BYTES + 1) as u64));
    let mut bytes = Vec::new();
    reader.read_until(b'\n', &mut bytes).await?;
    if bytes.len() > MAX_WIRE_BYTES || bytes.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Eine JSON-Zeile mit höchstens 4 KiB erwartet",
        ));
    }
    Ok(bytes)
}
fn check_peer(stream: &Stream) -> io::Result<()> {
    #[cfg(unix)]
    if stream.peer_creds()?.euid() != Some(nix::unistd::geteuid().as_raw()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Fremder Benutzer",
        ));
    }
    #[cfg(windows)]
    let _ = stream; // The protected pipe DACL and PIPE_REJECT_REMOTE_CLIENTS enforce access.
    Ok(())
}
pub fn notify(endpoint: &Endpoint, message: Notification) -> Result<(), ClientError> {
    let engine = Engine::embedded().map_err(|e| ClientError::Invalid(e.to_string()))?;
    let message = message
        .validate(&engine)
        .map_err(|e| ClientError::Invalid(e.to_string()))?;
    let mut bytes =
        serde_json::to_vec(&message).map_err(|e| ClientError::Invalid(e.to_string()))?;
    bytes.push(b'\n');
    #[cfg(unix)]
    endpoint::check_client_endpoint(endpoint).map_err(ClientError::Unavailable)?;
    runtime()
        .map_err(ClientError::Unavailable)?
        .block_on(async {
            timeout(Duration::from_millis(1500), async {
                let mut stream =
                    Stream::connect(endpoint.name().map_err(ClientError::Unavailable)?)
                        .await
                        .map_err(ClientError::Unavailable)?;
                check_peer(&stream).map_err(ClientError::Unavailable)?;
                stream
                    .write_all(&bytes)
                    .await
                    .map_err(ClientError::Unavailable)?;
                let response = read_line(&mut stream)
                    .await
                    .map_err(ClientError::Unavailable)?;
                let ack: Ack = serde_json::from_slice(&response)
                    .map_err(|_| ClientError::Rejected("Ungültige Empfangsbestätigung".into()))?;
                if ack.v != 1 || !ack.ok {
                    return Err(ClientError::Rejected(
                        ack.error
                            .unwrap_or_else(|| "Ungültige Protokollversion".into()),
                    ));
                }
                Ok(())
            })
            .await
            .map_err(|_| ClientError::Timeout)?
        })
}
pub struct Server {
    queue: Arc<Mutex<NotificationQueue>>,
    error: Arc<Mutex<Option<String>>>,
    started: Instant,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Server {
    pub fn start(wake: impl Fn() + Send + 'static) -> io::Result<Self> {
        Self::start_at(Endpoint::for_user()?, wake)
    }
    pub fn start_at(endpoint: Endpoint, wake: impl Fn() + Send + 'static) -> io::Result<Self> {
        let queue = Arc::new(Mutex::new(NotificationQueue::default()));
        let error = Arc::new(Mutex::new(None));
        let started = Instant::now();
        let (stop, stopped) = oneshot::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (worker_queue, worker_error) = (queue.clone(), error.clone());
        let thread = thread::Builder::new()
            .name("beni-status".into())
            .spawn(move || {
                let rt = match runtime() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let _entered = rt.enter();
                #[cfg(unix)]
                let _lock = match endpoint::prepare_directory(&endpoint) {
                    Ok(lock) => lock,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                #[cfg(unix)]
                let options = endpoint
                    .name()
                    .map(|name| ListenerOptions::new().name(name));
                #[cfg(windows)]
                let options = endpoint::listener_options(&endpoint);
                let listener = match options.and_then(|o| o.create_tokio()) {
                    Ok(listener) => listener,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Err(e) = std::fs::set_permissions(
                        &endpoint.0,
                        std::fs::Permissions::from_mode(0o600),
                    ) {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                }
                let engine = match Engine::embedded() {
                    Ok(engine) => engine,
                    Err(e) => {
                        let _ = ready_tx.send(Err(io::Error::other(e)));
                        return;
                    }
                };
                if ready_tx.send(Ok(())).is_err() {
                    return;
                }
                rt.block_on(async {
                    if let Err(e) =
                        serve(listener, stopped, worker_queue, started, engine, wake).await
                    {
                        *worker_error.lock().unwrap() = Some(e.to_string());
                    }
                });
            })?;
        match ready_rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(Self {
                queue,
                error,
                started,
                stop: Some(stop),
                thread: Some(thread),
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = stop.send(());
                let _ = thread.join();
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Statusschnittstelle startet nicht rechtzeitig",
                ))
            }
        }
    }
    pub fn pop(&self) -> Option<Delivery> {
        self.queue
            .lock()
            .unwrap()
            .pop(self.started.elapsed().as_millis() as u64)
    }
    pub fn pending(&self) -> usize {
        self.queue
            .lock()
            .unwrap()
            .len(self.started.elapsed().as_millis() as u64)
    }
    pub fn take_error(&self) -> Option<String> {
        self.error.lock().unwrap().take()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
async fn serve(
    listener: Listener,
    mut stop: oneshot::Receiver<()>,
    queue: Arc<Mutex<NotificationQueue>>,
    started: Instant,
    engine: Engine,
    wake: impl Fn(),
) -> io::Result<()> {
    loop {
        let mut stream =
            tokio::select! { _ = &mut stop => return Ok(()), conn = listener.accept() => conn? };
        let handle = async {
            let result = async {
                check_peer(&stream)?;
                let bytes = read_line(&mut stream).await?;
                let message = Notification::parse(&bytes, &engine).map_err(io::Error::other)?;
                queue
                    .lock()
                    .unwrap()
                    .enqueue(message, started.elapsed().as_millis() as u64)
                    .map_err(io::Error::other)?;
                wake();
                Ok(())
            }
            .await;
            let ack = Ack::result(result.map_err(|e: io::Error| e.to_string()));
            let mut bytes = serde_json::to_vec(&ack).map_err(io::Error::other)?;
            bytes.push(b'\n');
            stream.write_all(&bytes).await
        };
        tokio::select! { _ = &mut stop => return Ok(()), _ = timeout(IO_TIMEOUT, handle) => {} }
    }
}
