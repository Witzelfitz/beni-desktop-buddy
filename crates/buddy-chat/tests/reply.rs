use buddy_chat::*;
use serde_json::json;

#[test]
fn structured_reply_preserves_emotions_and_normalizes_swiss_spelling() {
    let reply = Reply::decode(
        &json!({"v":1,"segments":[
            {"text":"Hallo! Das ist eine schöne Straße.","emotion":"friendly"},
            {"text":"Eine großartige Idee!","emotion":"happy"}
        ]})
        .to_string(),
    )
    .unwrap();
    assert_eq!(
        reply.segments()[0].text(),
        "Hallo! Das ist eine schöne Strasse."
    );
    assert_eq!(reply.segments()[1].emotion(), Emotion::Happy);
    assert_eq!(Reply::decode(&reply.encode()).unwrap().text(), reply.text());
    assert!(!format!("{reply:?}").contains("Strasse"));
}

#[test]
fn invalid_json_actions_versions_emotions_and_dialect_never_become_a_reply() {
    for value in [
        json!({"v":2,"segments":[{"text":"Hallo","emotion":"neutral"}]}),
        json!({"v":1,"segments":[]}),
        json!({"v":1,"segments":[{"text":"Hallo","emotion":"execute"}]}),
        json!({"v":1,"segments":[{"text":"Hallo","emotion":"neutral","animation":"run-shell"}]}),
        json!({"v":1,"segments":[{"text":" ","emotion":"neutral"}]}),
        json!({"v":1,"segments":[{"text":"Hallo\u{0}","emotion":"neutral"}]}),
        json!({"v":1,"segments":[{"text":"a".repeat(MAX_SEGMENT_CHARS+1),"emotion":"neutral"}]}),
        json!({"v":1,"segments":vec![json!({"text":"Hallo","emotion":"neutral"});MAX_SEGMENTS+1]}),
    ] {
        assert!(Reply::decode(&value.to_string()).is_err());
    }
    for text in [
        "Hoi!",
        "Grüezi!",
        "Das isch chli schwierig.",
        "Du chasch das.",
    ] {
        assert_eq!(
            Reply::decode(
                &json!({"v":1,"segments":[{"text":text,"emotion":"friendly"}]}).to_string()
            )
            .unwrap_err(),
            ChatError::Language
        );
    }
    assert!(Reply::decode("{\"v\":1,\"v\":1,\"segments\":[]}").is_err());
    assert!(Reply::decode("```json\n{}\n```").is_err());
    assert!(Reply::decode(&"x".repeat(MAX_REPLY_BYTES + 1)).is_err());
}

#[test]
fn structured_context_preview_is_exact_and_cancelled_emotions_cannot_enter_history() {
    let mut session = ChatSession::default();
    let (request, _) = session.start("test", "Eine Frage", 0).unwrap();
    let reply =
        Reply::decode(r#"{"v":1,"segments":[{"text":"Hallo!","emotion":"friendly"}]}"#).unwrap();
    session.receive(request.id(), StreamEvent::Reply(reply.clone()), 1);
    session.receive(
        request.id(),
        StreamEvent::Finished {
            reason: FinishReason::Stop,
            usage: Usage::default(),
        },
        2,
    );
    let preview = session.preview("Weiter").unwrap();
    assert_eq!(preview[2].text(), reply.encode());
    let (next, _) = session.start("test", "Weiter", 3).unwrap();
    assert_eq!(next.messages(), preview);
    session.cancel();
    assert!(!session.receive(next.id(), StreamEvent::Reply(reply), 4));
    assert_eq!(session.history_turns(), 1);
    assert!(session.reply().is_none());
}

#[test]
fn structured_progress_keeps_deadlines_without_exposing_json_and_events_stay_bounded() {
    let mut session = ChatSession::default();
    let (request, _) = session.start("test", "Frage", 0).unwrap();
    session.receive(request.id(), StreamEvent::Progress, 10_000);
    assert_eq!(session.phase(), Phase::Waiting);
    assert!(session.response().is_empty());
    session.advance(39_999);
    assert!(session.busy());
    session.advance(40_000);
    assert_eq!(session.phase(), Phase::Failed(ChatError::Timeout));
    assert!(PERSONA.contains("niemals Schweizerdeutsch oder Dialekt"));
    assert!(PERSONA.contains("Hochdeutsch mit Schweizer Rechtschreibung"));
}

#[test]
fn dialect_split_across_legacy_chunks_never_commits_and_example_is_valid() {
    let example = Reply::decode(include_str!("../../../docs/examples/chat-reply.json")).unwrap();
    assert_eq!(example.segments().len(), 4);
    let mut session = ChatSession::default();
    let (request, _) = session.start("test", "Hoi Beni", 0).unwrap();
    for text in ["Ho", "i!"] {
        session.receive(request.id(), StreamEvent::Delta(text.into()), 1);
    }
    session.receive(
        request.id(),
        StreamEvent::Finished {
            reason: FinishReason::Stop,
            usage: Usage::default(),
        },
        2,
    );
    assert_eq!(session.phase(), Phase::Failed(ChatError::Language));
    assert!(session.response().is_empty());
    assert_eq!(session.history_turns(), 0);
}
