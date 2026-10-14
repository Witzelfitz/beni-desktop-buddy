use super::*;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Instant,
};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn tags(names: &[&str]) -> String {
    json!({"models":names.iter().map(|name| json!({"name":name,"size":1024,"digest":DIGEST})).collect::<Vec<_>>()}).to_string()
}
fn text(text: &str, done: bool) -> String {
    json!({"model":"test:small","message":{"role":"assistant","content":text},"done":done,"done_reason":"stop","prompt_eval_count":12,"eval_count":4}).to_string()
}
fn model() -> Model {
    Model {
        name: "test:small".into(),
        size: 1024,
        digest: DIGEST.into(),
    }
}
fn info() -> String {
    json!({"capabilities":["completion"]}).to_string()
}

struct Fixture {
    provider: Ollama,
    requests: Arc<Mutex<Vec<(String, Value)>>>,
    closed: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}
impl Fixture {
    fn new(responses: Vec<(u16, String)>, hold_last: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let saved = requests.clone();
        let closed = Arc::new(AtomicBool::new(false));
        let closed_copy = closed.clone();
        let handle = std::thread::spawn(move || {
            let count = responses.len();
            for (i, (status, body)) in responses.into_iter().enumerate() {
                let start = Instant::now();
                let mut connection = loop {
                    if let Ok((connection, _)) = listener.accept() {
                        break connection;
                    }
                    if start.elapsed() > Duration::from_secs(5) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                };
                connection.set_nonblocking(false).unwrap();
                connection
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                connection
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let (path, payload) = read_request(&mut connection);
                if hold_last && i + 1 == count {
                    connection.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n").unwrap();
                    saved.lock().unwrap().push((path, payload));
                    let mut byte = [0];
                    let result = connection.read(&mut byte);
                    closed_copy.store(
                        matches!(result, Ok(0))
                            || result.is_err_and(|e| {
                                matches!(
                                    e.kind(),
                                    std::io::ErrorKind::ConnectionReset
                                        | std::io::ErrorKind::ConnectionAborted
                                )
                            }),
                        Ordering::Release,
                    );
                } else {
                    saved.lock().unwrap().push((path, payload));
                    let header = format!(
                        "HTTP/1.1 {status} Test\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\nLocation: http://127.0.0.1:9/forbidden\r\n\r\n",
                        body.len()
                    );
                    let _ = connection.write_all(header.as_bytes());
                    let _ = connection.write_all(body.as_bytes());
                }
            }
        });
        Self {
            provider: Ollama {
                endpoint,
                selected: Some(model()),
            },
            requests,
            closed,
            handle: Some(handle),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let result = handle.join();
            if !std::thread::panicking() {
                assert!(result.is_ok());
            }
        }
    }
}
fn read_request(connection: &mut TcpStream) -> (String, Value) {
    let mut bytes = Vec::new();
    let mut chunk = [0; 4096];
    let end = loop {
        let n = connection.read(&mut chunk).unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&chunk[..n]);
        assert!(bytes.len() < 256 * 1024);
        if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
            break end + 4;
        }
    };
    let header = std::str::from_utf8(&bytes[..end]).unwrap();
    let path = header.split_whitespace().nth(1).unwrap().to_owned();
    let length = header
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|n| n.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    while bytes.len() < end + length {
        let n = connection.read(&mut chunk).unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&chunk[..n]);
    }
    (
        path,
        serde_json::from_slice(&bytes[end..end + length]).unwrap_or(Value::Null),
    )
}

#[test]
fn discovery_excludes_cloud_embedding_and_image_models() {
    let fixture = Fixture::new(vec![(200,tags(&["test:small","embed:latest","image:latest","remote:cloud","remote:local"])),
        (200,info()),(200,json!({"capabilities":["embedding"]}).to_string()),
        (200,json!({"capabilities":["image"]}).to_string()),
        (200,json!({"capabilities":["completion"],"remote_host":"https://example.test","remote_model":"remote"}).to_string())],false);
    let models = fixture
        .provider
        .discover(&CancellationToken::default())
        .unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "test:small");
    assert!(
        fixture
            .requests
            .lock()
            .unwrap()
            .iter()
            .all(|(path, _)| path != "/api/chat")
    );
}

#[test]
fn real_http_chat_rechecks_model_and_sends_only_explicit_conversation() {
    let fixture = Fixture::new(
        vec![
            (200, tags(&["test:small"])),
            (200, info()),
            (
                200,
                format!("{}\n{}\n", text("Hoi 😀", false), text("!", true)),
            ),
        ],
        false,
    );
    let mut session = buddy_chat::ChatSession::default();
    let (request, cancellation) = session.start("test:small", "Nur diese Frage", 0).unwrap();
    let mut events = Vec::new();
    fixture
        .provider
        .stream(&request, &cancellation, &mut |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();
    for event in events {
        session.receive(request.id(), event, 1);
    }
    assert_eq!(session.response(), "Hoi 😀!");
    assert_eq!(session.history_turns(), 1);
    let requests = fixture.requests.lock().unwrap();
    assert_eq!(
        requests
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>(),
        ["/api/tags", "/api/show", "/api/chat"]
    );
    let body = &requests[2].1;
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
    assert_eq!(body["messages"][0]["content"], buddy_chat::PERSONA);
    assert_eq!(body["messages"][1]["content"], "Nur diese Frage");
    assert_eq!(body["options"]["num_predict"], 512);
    assert!(body.get("tools").is_none());
    assert!(body.get("images").is_none());
}

#[test]
fn cancellation_interrupts_pending_http_body_and_closes_socket() {
    let fixture = Fixture::new(
        vec![
            (200, tags(&["test:small"])),
            (200, info()),
            (200, String::new()),
        ],
        true,
    );
    let mut session = buddy_chat::ChatSession::default();
    let (request, token) = session.start("test:small", "cancel", 0).unwrap();
    let provider = fixture.provider.clone();
    let cancel = token.clone();
    let worker = std::thread::spawn(move || provider.stream(&request, &token, &mut |_| Ok(())));
    let start = Instant::now();
    while fixture.requests.lock().unwrap().len() < 3 {
        assert!(start.elapsed() < Duration::from_secs(3));
        std::thread::sleep(Duration::from_millis(5));
    }
    cancel.cancel();
    assert_eq!(worker.join().unwrap(), Err(ChatError::Cancelled));
    let start = Instant::now();
    while !fixture.closed.load(Ordering::Acquire) {
        assert!(start.elapsed() < Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn changed_or_cloud_model_is_rejected_before_chat_payload() {
    for remote in [false, true] {
        let fixtures = if remote {
            vec![
                (200, tags(&["test:small"])),
                (
                    200,
                    json!({"capabilities":["completion"],"remote_model":"cloud"}).to_string(),
                ),
            ]
        } else {
            vec![(200, tags(&["other:small"]))]
        };
        let fixture = Fixture::new(fixtures, false);
        let mut session = buddy_chat::ChatSession::default();
        let (request, token) = session.start("test:small", "do not send", 0).unwrap();
        assert_eq!(
            fixture.provider.stream(&request, &token, &mut |_| Ok(())),
            Err(ChatError::InvalidModel)
        );
        assert!(
            fixture
                .requests
                .lock()
                .unwrap()
                .iter()
                .all(|(path, _)| path != "/api/chat")
        );
    }
}

#[test]
fn redirects_and_raw_error_bodies_never_become_diagnostics() {
    for (code, expected) in [
        (302, ChatError::Protocol),
        (401, ChatError::Authentication),
        (429, ChatError::RateLimited),
        (500, ChatError::ProviderFailure),
    ] {
        let fixture = Fixture::new(vec![(code, "private-provider-error".into())], false);
        let error = fixture
            .provider
            .discover(&CancellationToken::default())
            .err()
            .unwrap();
        assert_eq!(error, expected);
        assert!(!error.to_string().contains("private-provider-error"));
        assert_eq!(fixture.requests.lock().unwrap().len(), 1);
    }
}

#[test]
fn decoder_accepts_split_utf8_ignores_reasoning_and_preserves_finish_usage() {
    let line=json!({"model":"test:small","done":true,"done_reason":"length","message":{"role":"assistant","content":"😀".repeat(600),"thinking":"private-thinking"},"eval_count":512}).to_string();
    let mut decoder = Decoder::new("test:small");
    let mut events = Vec::new();
    for byte in line.bytes() {
        decoder
            .push(&[byte], &mut |event| {
                events.push(event);
                Ok(())
            })
            .unwrap();
    }
    decoder
        .finish(&mut |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();
    let text = events
        .iter()
        .filter_map(|e| {
            if let StreamEvent::Delta(t) = e {
                assert!(t.len() <= MAX_DELTA_BYTES);
                Some(t.as_str())
            } else {
                None
            }
        })
        .collect::<String>();
    assert_eq!(text, "😀".repeat(600));
    assert!(!text.contains("private-thinking"));
    assert!(matches!(
        events.last().unwrap(),
        StreamEvent::Finished {
            reason: FinishReason::Length,
            usage: Usage {
                output_tokens: Some(512),
                ..
            }
        }
    ));
}

#[test]
fn malformed_incomplete_tool_remote_or_oversized_streams_fail_closed() {
    for line in ["invalid".into(), "{}".into(), json!({"model":"test:small","done":true,"message":{"role":"tool","content":"bad"}}).to_string(),
        json!({"model":"test:small","done":true,"message":{"role":"assistant","content":"","tool_calls":[{}]}}).to_string(),
        json!({"model":"test:small","remote_host":"https://remote.test","done":true}).to_string(),
        json!({"model":"test:small","done":true,"error":"private-key-error"}).to_string(),
        text(&"a".repeat(MAX_RESPONSE_BYTES+1),true), "a".repeat(MAX_LINE_BYTES+1)] {
        let mut decoder=Decoder::new("test:small");
        assert!(decoder.push(format!("{line}\n").as_bytes(),&mut |_| Ok(())).is_err());
    }
    let mut decoder = Decoder::new("test:small");
    decoder
        .push(text("incomplete", false).as_bytes(), &mut |_| Ok(()))
        .unwrap();
    assert_eq!(decoder.finish(&mut |_| Ok(())), Err(ChatError::Protocol));
    let mut decoder = Decoder::new("test:small");
    assert_eq!(
        decoder.push(&vec![b' '; MAX_WIRE_BYTES + 1], &mut |_| Ok(())),
        Err(ChatError::Protocol)
    );
    let mut decoder = Decoder::new("test:small");
    assert_eq!(
        decoder.push(&[0xff, b'\n'], &mut |_| Ok(())),
        Err(ChatError::Protocol)
    );
}

#[test]
fn bounded_metadata_and_already_cancelled_requests_are_rejected() {
    let fixture = Fixture::new(vec![(200, "a".repeat(MAX_METADATA_BYTES + 1))], false);
    assert_eq!(
        fixture
            .provider
            .discover(&CancellationToken::default())
            .err(),
        Some(ChatError::Protocol)
    );
    let token = CancellationToken::default();
    token.cancel();
    assert_eq!(
        Ollama::default().discover(&token).err(),
        Some(ChatError::Cancelled)
    );
}

#[test]
#[ignore = "Gezielte lokale Ollama-Abnahme; kein Modellaufruf in der CI"]
fn real_local_ollama_answers_without_downloading_a_model() {
    let provider = Ollama::default();
    let models = provider.discover(&CancellationToken::default()).unwrap();
    let model = models
        .into_iter()
        .next()
        .expect("vorhandenes lokales Chatmodell");
    let mut session = buddy_chat::ChatSession::default();
    let (request, token) = session
        .start(&model.name, "Sag kurz Hallo als Beni, in einem Satz.", 0)
        .unwrap();
    let started = Instant::now();
    provider
        .with_model(model)
        .stream(&request, &token, &mut |event| {
            session.receive(request.id(), event, started.elapsed().as_millis() as u64);
            Ok(())
        })
        .unwrap();
    assert!(matches!(
        session.phase(),
        buddy_chat::Phase::Completed { .. }
    ));
    assert!(!session.response().trim().is_empty());
    assert_eq!(session.history_turns(), 1);
}
