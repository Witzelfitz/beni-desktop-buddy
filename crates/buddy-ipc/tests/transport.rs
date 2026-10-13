mod common;
use buddy_core::notifications::Notification;
use buddy_ipc::{Ack, ClientError, Server, notify};
use interprocess::local_socket::tokio::{Stream, prelude::*};
#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, ToFsName};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
async fn connect(endpoint: &buddy_ipc::Endpoint) -> Stream {
    #[cfg(unix)]
    let name = endpoint.0.as_str().to_fs_name::<GenericFilePath>().unwrap();
    #[cfg(windows)]
    let name = endpoint
        .0
        .as_str()
        .to_ns_name::<GenericNamespaced>()
        .unwrap();
    Stream::connect(name).await.unwrap()
}
#[test]
fn real_transport_accepts_normalizes_wakes_rejects_flood_and_stops() {
    let endpoint = common::TestEndpoint::new();
    let count = Arc::new(AtomicUsize::new(0));
    let wake = count.clone();
    let server = Server::start_at(endpoint.endpoint.clone(), move || {
        wake.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();
    let mut m = Notification::new("  fertig\n ");
    m.state = Some("success".into());
    notify(&endpoint.endpoint, m).unwrap();
    let got = server.pop().unwrap();
    assert_eq!(got.message.text, "fertig");
    assert_eq!(got.message.state.as_deref(), Some("daumen"));
    assert_eq!(count.load(Ordering::SeqCst), 1);
    for _ in 0..4 {
        notify(&endpoint.endpoint, Notification::new("ok")).unwrap();
    }
    assert!(matches!(
        notify(&endpoint.endpoint, Notification::new("ok")),
        Err(ClientError::Rejected(_))
    ));
    assert_eq!(server.pending(), 4);
    let received = server.drain_received();
    assert_eq!(received.len(), 5, "rejected flood must not enter history");
    assert_eq!(received[0].message.text, "fertig");
    assert_eq!(received[0].message.state.as_deref(), Some("daumen"));
    assert_eq!(
        server.pending(),
        4,
        "receipt draining must not consume delivery"
    );
    assert!(
        server.drain_received().is_empty(),
        "receipts are drained exactly once"
    );
    assert!(Server::start_at(endpoint.endpoint.clone(), || {}).is_err());
    drop(server);
    assert!(matches!(
        notify(&endpoint.endpoint, Notification::new("ok")),
        Err(ClientError::Unavailable(_))
    ));
    let restart = Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
    assert!(restart.drain_received().is_empty());
}
#[test]
fn invalid_and_oversized_wire_messages_are_rejected_and_never_queued() {
    let endpoint = common::TestEndpoint::new();
    let server = Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        for bytes in [
            b"{bad}\n".to_vec(),
            b"{\"v\":2,\"text\":\"ok\"}\n".to_vec(),
            vec![b'x'; 4097],
        ] {
            let mut stream = connect(&endpoint.endpoint).await;
            stream.write_all(&bytes).await.unwrap();
            let mut line = String::new();
            tokio::time::timeout(
                Duration::from_secs(3),
                BufReader::new(&mut stream).read_line(&mut line),
            )
            .await
            .unwrap()
            .unwrap();
            let ack: Ack = serde_json::from_str(&line).unwrap();
            assert!(!ack.ok);
        }
    });
    assert_eq!(server.pending(), 0);
    assert!(server.drain_received().is_empty());
}
#[test]
fn expired_delivery_still_has_a_receipt_and_draining_does_not_replay_it() {
    let endpoint = common::TestEndpoint::new();
    let server = Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
    let mut message = Notification::new("Kurzlebig");
    message.ttl_s = 1;
    notify(&endpoint.endpoint, message).unwrap();
    std::thread::sleep(Duration::from_millis(1100));
    assert!(server.pop().is_none());
    let received = server.drain_received();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].message.text, "Kurzlebig");
    assert!(received[0].received_at.elapsed() >= Duration::from_secs(1));
    assert!(server.drain_received().is_empty());
    assert!(server.pop().is_none());
}
#[test]
fn an_idle_client_cannot_block_shutdown_or_restarting() {
    let endpoint = common::TestEndpoint::new();
    let server = Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let connection = runtime.block_on(connect(&endpoint.endpoint));
    let now = Instant::now();
    drop(server);
    assert!(now.elapsed() < Duration::from_secs(2));
    drop(connection);
    let _restart = Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
}
#[cfg(unix)]
#[test]
fn unsafe_directory_or_socket_is_rejected_without_deleting_files() {
    use std::os::unix::fs::PermissionsExt;
    let endpoint = common::TestEndpoint::new();
    let path = std::path::Path::new(&endpoint.endpoint.0);
    let directory = path.parent().unwrap();
    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Server::start_at(endpoint.endpoint.clone(), || {}).is_err());
    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(path, "must stay").unwrap();
    assert!(Server::start_at(endpoint.endpoint.clone(), || {}).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "must stay");
}
#[cfg(unix)]
#[test]
fn stale_owned_socket_is_reclaimed_but_bad_socket_permissions_are_not() {
    use std::os::unix::{fs::PermissionsExt, net::UnixListener};
    let endpoint = common::TestEndpoint::new();
    let socket = UnixListener::bind(&endpoint.endpoint.0).unwrap();
    std::fs::set_permissions(&endpoint.endpoint.0, std::fs::Permissions::from_mode(0o600)).unwrap();
    drop(socket);
    let server = Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
    assert_eq!(
        std::fs::metadata(&endpoint.endpoint.0)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    std::fs::set_permissions(&endpoint.endpoint.0, std::fs::Permissions::from_mode(0o666)).unwrap();
    assert!(matches!(
        notify(&endpoint.endpoint, Notification::new("ok")),
        Err(ClientError::Unavailable(_))
    ));
    drop(server);
}

#[test]
fn client_times_out_if_an_endpoint_never_acknowledges() {
    let endpoint = common::TestEndpoint::new();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let worker_endpoint = endpoint.endpoint.clone();
    let worker = std::thread::spawn(move || {
        use interprocess::local_socket::{ListenerOptions, traits::Listener as _};
        #[cfg(unix)]
        let name = worker_endpoint
            .0
            .as_str()
            .to_fs_name::<GenericFilePath>()
            .unwrap();
        #[cfg(windows)]
        let name = worker_endpoint
            .0
            .as_str()
            .to_ns_name::<GenericNamespaced>()
            .unwrap();
        let listener = ListenerOptions::new().name(name).create_sync().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&worker_endpoint.0, std::fs::Permissions::from_mode(0o600))
                .unwrap();
        }
        ready_tx.send(()).unwrap();
        let _connection = listener.accept().unwrap();
        std::thread::sleep(Duration::from_secs(2));
    });
    ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(matches!(
        notify(&endpoint.endpoint, Notification::new("ok")),
        Err(ClientError::Timeout)
    ));
    worker.join().unwrap();
}
