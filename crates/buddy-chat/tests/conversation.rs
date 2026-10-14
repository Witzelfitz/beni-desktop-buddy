use buddy_chat::*;

fn done() -> StreamEvent {
    StreamEvent::Finished {
        reason: FinishReason::Stop,
        usage: Usage {
            input_tokens: Some(3),
            output_tokens: Some(2),
        },
    }
}
fn complete(session: &mut ChatSession, input: &str, output: &str) {
    let (request, _) = session.start("local-model:small", input, 0).unwrap();
    for chunk in output.as_bytes().chunks(MAX_DELTA_BYTES) {
        session.receive(
            request.id(),
            StreamEvent::Delta(std::str::from_utf8(chunk).unwrap().into()),
            1,
        );
    }
    session.receive(request.id(), done(), 2);
    assert!(matches!(session.phase(), Phase::Completed { .. }));
}

#[test]
fn streamed_reply_preserves_unicode_and_commits_complete_pairs() {
    let mut session = ChatSession::default();
    let (request, token) = session
        .start("gemma:small", "  Hoi Beni 😀  ", 100)
        .unwrap();
    assert_eq!(session.phase(), Phase::Waiting);
    assert!(!token.is_cancelled());
    assert_eq!(request.messages()[0].role(), Role::System);
    assert_eq!(request.messages()[0].text(), PERSONA);
    assert_eq!(request.messages()[1].text(), "Hoi Beni 😀");
    assert_eq!(request.max_output_tokens(), 512);
    session.receive(request.id(), StreamEvent::Delta("Hoi! ".into()), 101);
    session.receive(
        request.id(),
        StreamEvent::Delta("Was läuft? 😀".into()),
        102,
    );
    assert_eq!(session.phase(), Phase::Speaking);
    assert_eq!(session.history_turns(), 0);
    session.receive(request.id(), done(), 103);
    assert_eq!(session.history_turns(), 1);
    assert_eq!(session.response(), "Hoi! Was läuft? 😀");
    assert_eq!(
        session.history().collect::<Vec<_>>(),
        [("Hoi Beni 😀", "Hoi! Was läuft? 😀")]
    );
    assert_eq!(
        session.phase(),
        Phase::Completed {
            reason: FinishReason::Stop,
            usage: Usage {
                input_tokens: Some(3),
                output_tokens: Some(2)
            }
        }
    );
    let (next, _) = session
        .start("gemma:small", "Noch eine Frage", 104)
        .unwrap();
    assert_eq!(
        next.messages()
            .iter()
            .map(Message::role)
            .collect::<Vec<_>>(),
        [Role::System, Role::User, Role::Assistant, Role::User]
    );
    assert_eq!(next.messages()[2].text(), "Hoi! Was läuft? 😀");
    assert!(!session.receive(request.id(), StreamEvent::Delta("late".into()), 105));
    assert!(session.response().is_empty());
}

#[test]
fn rejected_input_does_not_start_or_destroy_previous_reply() {
    let mut session = ChatSession::default();
    complete(&mut session, "Hallo", "Hoi");
    for (model, input, expected) in [
        ("", "Hallo".into(), ChatError::InvalidModel),
        ("bad\nmodel", "Hallo".into(), ChatError::InvalidModel),
        ("ok", " \n\t ".into(), ChatError::EmptyInput),
        (
            "ok",
            "😀".repeat(MAX_INPUT_BYTES / 4 + 1),
            ChatError::InputTooLong,
        ),
        (
            "ok",
            " ".repeat(MAX_INPUT_BYTES + 1),
            ChatError::InputTooLong,
        ),
        ("ok", "hello\0world".into(), ChatError::InvalidInput),
        ("ok", "hello\u{7f}".into(), ChatError::InvalidInput),
    ] {
        assert_eq!(session.start(model, &input, 3).unwrap_err(), expected);
        assert!(!session.busy());
        assert_eq!(session.response(), "Hoi");
        assert_eq!(session.history_turns(), 1);
    }
    assert_eq!(
        session.start(&"a".repeat(129), "Hallo", 3).unwrap_err(),
        ChatError::InvalidModel
    );
    let (_, _) = session
        .start("ok", &"ä".repeat(MAX_INPUT_BYTES / 2), 4)
        .unwrap();
    assert_eq!(
        session.start("ok", "Hallo", 5).unwrap_err(),
        ChatError::Busy
    );
}

#[test]
fn cancellation_is_immediate_shared_and_rejects_old_events_after_restart() {
    let mut session = ChatSession::default();
    let (old, token) = session.start("ok", "cancel me", 0).unwrap();
    session.receive(old.id(), StreamEvent::Delta("partial".into()), 1);
    session.cancel();
    assert!(token.is_cancelled());
    assert!(!session.busy());
    assert_eq!(session.phase(), Phase::Cancelled);
    assert_eq!(session.history_turns(), 0);
    let (new, fresh) = session.start("ok", "new request", 2).unwrap();
    assert!(!fresh.is_cancelled());
    assert_ne!(new.id(), old.id());
    assert!(!session.receive(old.id(), done(), 3));
    assert!(!session.receive(old.id(), StreamEvent::Failed(ChatError::Unavailable), 3));
    assert_eq!(session.phase(), Phase::Waiting);
    fresh.cancel();
    session.advance(4);
    assert_eq!(session.phase(), Phase::Cancelled);
}

#[test]
fn dropping_a_session_signals_its_background_worker() {
    let token = {
        let mut session = ChatSession::default();
        let (_, token) = session.start("ok", "question", 0).unwrap();
        token
    };
    assert!(token.is_cancelled());
}

#[test]
fn clearing_for_provider_change_cancels_and_does_not_reuse_ids_or_context() {
    let mut session = ChatSession::default();
    complete(&mut session, "private question", "private answer");
    let (old, token) = session.start("ok", "next", 0).unwrap();
    session.clear();
    assert!(token.is_cancelled());
    assert_eq!(session.phase(), Phase::Idle);
    assert_eq!(session.history_turns(), 0);
    assert!(session.response().is_empty());
    let (new, _) = session.start("other", "fresh", 1).unwrap();
    assert_eq!(new.messages().len(), 2);
    assert_ne!(old.id(), new.id());
    assert!(!session.receive(old.id(), StreamEvent::Delta("private answer".into()), 2));
}

#[test]
fn errors_empty_responses_and_incomplete_streams_never_enter_context() {
    for error in [
        ChatError::Unavailable,
        ChatError::Authentication,
        ChatError::RateLimited,
        ChatError::ProviderFailure,
        ChatError::Protocol,
    ] {
        let mut session = ChatSession::default();
        let (request, token) = session.start("ok", "question", 0).unwrap();
        session.receive(request.id(), StreamEvent::Delta("unfinished".into()), 1);
        session.receive(request.id(), StreamEvent::Failed(error), 2);
        assert_eq!(session.phase(), Phase::Failed(error));
        assert!(token.is_cancelled());
        assert_eq!(session.history_turns(), 0);
        let (next, _) = session.start("ok", "again", 3).unwrap();
        assert_eq!(next.messages().len(), 2);
    }
    for text in ["", " \n\t "] {
        let mut session = ChatSession::default();
        let (request, _) = session.start("ok", "question", 0).unwrap();
        session.receive(request.id(), StreamEvent::Delta(text.into()), 1);
        session.receive(request.id(), done(), 2);
        assert_eq!(session.phase(), Phase::Failed(ChatError::InvalidResponse));
        assert_eq!(session.history_turns(), 0);
    }
}

#[test]
fn history_drops_oldest_whole_turns_and_bounds_entire_request_context() {
    let mut session = ChatSession::default();
    for i in 0..15 {
        complete(
            &mut session,
            &format!("question {i}"),
            &format!("answer {i}"),
        );
    }
    assert_eq!(session.history_turns(), MAX_HISTORY_TURNS);
    let (request, _) = session.start("ok", "current", 0).unwrap();
    assert_eq!(request.messages().len(), 2 * MAX_HISTORY_TURNS + 2);
    assert_eq!(request.messages()[1].text(), "question 5");
    session.clear();
    for _ in 0..3 {
        complete(
            &mut session,
            &"i".repeat(MAX_INPUT_BYTES),
            &"o".repeat(MAX_RESPONSE_BYTES),
        );
    }
    assert_eq!(session.history_turns(), 1);
    let (request, _) = session
        .start("ok", &"i".repeat(MAX_INPUT_BYTES), 0)
        .unwrap();
    assert!(
        request
            .messages()
            .iter()
            .map(|m| m.text().len())
            .sum::<usize>()
            <= MAX_CONTEXT_BYTES
    );
    assert_eq!(request.messages().len(), 4);
    session.clear();
    for _ in 0..2 {
        complete(&mut session, "small", &"o".repeat(15 * 1024));
    }
    assert_eq!(session.history_turns(), 2);
    let (request, _) = session
        .start("ok", &"i".repeat(MAX_INPUT_BYTES), 0)
        .unwrap();
    assert_eq!(
        request.messages().len(),
        4,
        "request omits oldest pair to fit new input and persona"
    );
}

#[test]
fn response_and_delta_limits_fail_without_appending_or_committing_bad_chunk() {
    let mut session = ChatSession::default();
    let (request, token) = session.start("ok", "question", 0).unwrap();
    for _ in 0..MAX_RESPONSE_BYTES / MAX_DELTA_BYTES {
        session.receive(
            request.id(),
            StreamEvent::Delta("a".repeat(MAX_DELTA_BYTES)),
            1,
        );
    }
    assert_eq!(session.response().len(), MAX_RESPONSE_BYTES);
    session.receive(request.id(), StreamEvent::Delta("ä".into()), 2);
    assert_eq!(session.phase(), Phase::Failed(ChatError::ResponseLimit));
    assert_eq!(session.response().len(), MAX_RESPONSE_BYTES);
    assert_eq!(session.history_turns(), 0);
    assert!(token.is_cancelled());
    for chunk in [
        "a".repeat(MAX_DELTA_BYTES + 1),
        "bad\0".into(),
        "bad\u{1b}[31m".into(),
    ] {
        let (request, _) = session.start("ok", "question", 3).unwrap();
        session.receive(request.id(), StreamEvent::Delta(chunk), 4);
        assert_eq!(session.phase(), Phase::Failed(ChatError::InvalidResponse));
        assert!(session.response().is_empty());
    }
}

#[test]
fn event_flood_cannot_keep_an_unbounded_stream_open() {
    let mut session = ChatSession::default();
    let (request, token) = session.start("ok", "question", 0).unwrap();
    for _ in 0..MAX_STREAM_EVENTS {
        session.receive(request.id(), StreamEvent::Delta(String::new()), 1);
    }
    assert!(session.busy());
    session.receive(request.id(), StreamEvent::Delta(String::new()), 2);
    assert_eq!(session.phase(), Phase::Failed(ChatError::StreamLimit));
    assert!(token.is_cancelled());
    assert_eq!(session.history_turns(), 0);
}

#[test]
fn cold_model_has_total_timeout_then_text_stream_has_idle_timeout() {
    let mut session = ChatSession::default();
    let (request, token) = session.start("ok", "question", 100).unwrap();
    session.advance(100 + REQUEST_TIMEOUT_MS - 1);
    assert!(session.busy());
    assert!(!session.receive(request.id(), done(), 100 + REQUEST_TIMEOUT_MS));
    assert_eq!(session.phase(), Phase::Failed(ChatError::Timeout));
    assert!(token.is_cancelled());
    let (request, token) = session.start("ok", "question", 200_000).unwrap();
    session.receive(request.id(), StreamEvent::Delta("Hallo".into()), 200_001);
    session.receive(
        request.id(),
        StreamEvent::Delta(String::new()),
        200_002 + IDLE_TIMEOUT_MS / 2,
    );
    session.advance(200_001 + IDLE_TIMEOUT_MS);
    assert_eq!(session.phase(), Phase::Failed(ChatError::Timeout));
    assert!(token.is_cancelled());
    let (_, _) = session.start("ok", "question", 500_000).unwrap();
    session.advance(1);
    assert!(session.busy(), "backwards clock does not underflow");
}

#[test]
fn length_finish_is_an_explicit_success_and_duplicate_terminal_is_ignored() {
    let mut session = ChatSession::default();
    let (request, _) = session.start("ok", "question", 0).unwrap();
    session.receive(request.id(), StreamEvent::Delta("shortened".into()), 1);
    session.receive(
        request.id(),
        StreamEvent::Finished {
            reason: FinishReason::Length,
            usage: Usage::default(),
        },
        2,
    );
    assert_eq!(
        session.phase(),
        Phase::Completed {
            reason: FinishReason::Length,
            usage: Usage::default()
        }
    );
    assert_eq!(session.history_turns(), 1);
    assert!(!session.receive(request.id(), done(), 3));
    assert!(!session.receive(request.id(), StreamEvent::Failed(ChatError::Unavailable), 3));
}

#[test]
fn diagnostics_omit_all_user_model_and_generated_text() {
    let mut session = ChatSession::default();
    let (request, _) = session.start("secret-model", "secret-user", 0).unwrap();
    session.receive(request.id(), StreamEvent::Delta("secret-answer".into()), 1);
    for output in [
        format!("{session:?}"),
        format!("{request:?}"),
        format!("{:?}", StreamEvent::Delta("secret-delta".into())),
    ] {
        for secret in [
            "secret-model",
            "secret-user",
            "secret-answer",
            "secret-delta",
        ] {
            assert!(!output.contains(secret));
        }
    }
}

struct FakeProvider;
impl ChatProvider for FakeProvider {
    fn stream(
        &self,
        _request: &ChatRequest,
        cancellation: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ChatError>,
    ) -> Result<(), ChatError> {
        if cancellation.is_cancelled() {
            return Err(ChatError::Cancelled);
        }
        emit(StreamEvent::Delta("Hoi Beni!".into()))?;
        if cancellation.is_cancelled() {
            return Err(ChatError::Cancelled);
        }
        emit(done())
    }
}

#[test]
fn provider_contract_runs_on_worker_with_bounded_channel_and_shared_cancel() {
    let mut session = ChatSession::default();
    let (request, token) = session.start("fake", "Hallo", 0).unwrap();
    let id = request.id();
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        FakeProvider.stream(&request, &token, &mut |event| {
            send.send(event).map_err(|_| ChatError::Cancelled)
        })
    });
    for event in recv {
        session.receive(id, event, 1);
    }
    worker.join().unwrap().unwrap();
    assert_eq!(session.response(), "Hoi Beni!");
    assert_eq!(session.history_turns(), 1);
    let (request, token) = session.start("fake", "Hallo", 2).unwrap();
    session.cancel();
    let mut emitted = false;
    assert_eq!(
        FakeProvider.stream(&request, &token, &mut |_| {
            emitted = true;
            Ok(())
        }),
        Err(ChatError::Cancelled)
    );
    assert!(!emitted);
}
