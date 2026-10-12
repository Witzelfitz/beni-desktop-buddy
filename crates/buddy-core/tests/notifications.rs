use buddy_core::{Engine, notifications::*};
fn message(text: &str) -> Notification {
    Notification::new(text)
        .validate(&Engine::embedded().unwrap())
        .unwrap()
}
#[test]
fn protocol_rejects_bad_version_json_fields_size_state_source_and_ttl() {
    let engine = Engine::embedded().unwrap();
    for bytes in [
        b"invalid".as_slice(),
        br#"{"v":2,"text":"ok"}"#,
        br#"{"v":1,"text":"ok","execute":"rm"}"#,
        br#"{"v":1,"text":"ok","priority":"urgent"}"#,
        br#"{"v":1,"text":"ok","state":"unknown"}"#,
        br#"{"v":1,"text":"ok","state":"wuetend"}"#,
        br#"{"v":1,"text":"ok","ttl_s":0}"#,
        br#"{"v":1,"text":"ok","ttl_s":121}"#,
        br#"{"v":1,"text":"ok","source":"a/b"}"#,
        br#"{"v":1,"text":"ok","v":1}"#,
    ] {
        assert!(Notification::parse(bytes, &engine).is_err(), "{bytes:?}");
    }
    assert!(Notification::parse(&vec![b' '; MAX_WIRE_BYTES + 1], &engine).is_err());
    assert!(
        Notification::new("x".repeat(281))
            .validate(&engine)
            .is_err()
    );
    assert!(Notification::new("\n\r\0 ").validate(&engine).is_err());
}
#[test]
fn sanitizes_text_counts_unicode_and_explicit_state_overrides_rules() {
    let engine = Engine::embedded().unwrap();
    let mut input = Notification::new("  Build failed\n\u{001b}  ");
    input.state = Some("success".into());
    let result = input.validate(&engine).unwrap();
    assert_eq!(result.text, "Build failed");
    assert_eq!(result.state.as_deref(), Some("daumen"));
    assert!(
        Notification::new("😊".repeat(280))
            .validate(&engine)
            .is_ok()
    );
    assert!(
        Notification::new("😊".repeat(281))
            .validate(&engine)
            .is_err()
    );
}
#[test]
fn priority_fifo_ttl_and_rate_limits_are_deterministic() {
    let mut queue = NotificationQueue::default();
    queue.enqueue(message("eins"), 0).unwrap();
    let mut urgent = message("zwei");
    urgent.priority = Priority::High;
    queue.enqueue(urgent, 1).unwrap();
    queue.enqueue(message("drei"), 2).unwrap();
    assert_eq!(queue.pop(3).unwrap().message.text, "zwei");
    assert_eq!(queue.pop(4).unwrap().message.text, "eins");
    assert_eq!(queue.pop(30_002).map(|d| d.message.text), None);
    for i in 0..5 {
        queue.enqueue(message("fertig"), 40_000 + i).unwrap();
        queue.pop(40_000 + i);
    }
    assert!(queue.enqueue(message("fertig"), 40_006).is_err());
    assert!(queue.enqueue(message("fertig"), 50_000).is_ok());
}
#[test]
fn queue_and_global_source_spoofing_are_bounded() {
    let mut queue = NotificationQueue::default();
    for i in 0..20 {
        let mut m = message("ok");
        m.source = format!("source-{i}");
        m.ttl_s = 120;
        queue.enqueue(m, 0).unwrap();
    }
    let mut m = message("ok");
    m.source = "other-source".into();
    assert!(
        queue
            .enqueue(m.clone(), 0)
            .unwrap_err()
            .to_string()
            .contains("20 insgesamt")
    );
    assert!(
        queue
            .enqueue(m, 10_000)
            .unwrap_err()
            .to_string()
            .contains("voll")
    );
    assert_eq!(queue.len(120_000), 0);
}
#[test]
fn remaining_lifetime_and_reading_time_are_bounded() {
    let mut q = NotificationQueue::default();
    let mut m = message("fertig");
    m.ttl_s = 2;
    q.enqueue(m, 100).unwrap();
    assert_eq!(q.pop(1500).unwrap().remaining_ms, 600);
    assert_eq!(reading_time_ms("Hi"), 4000);
    assert!(reading_time_ms(&"x".repeat(280)) > reading_time_ms("Hi"));
    assert_eq!(reading_time_ms(&"x".repeat(1000)), 30000);
    assert_eq!(reading_time_ms("😊"), reading_time_ms("x"));
}
#[test]
fn rules_cover_real_german_and_english_status_examples() {
    let examples = [
        ("Build failed", "stopp"),
        ("Tests failed", "stopp"),
        ("Deploy failure", "stopp"),
        ("Syntax error", "stopp"),
        ("Fehler beim Speichern", "stopp"),
        ("Build fehlgeschlagen", "stopp"),
        ("Vorgang abgebrochen", "stopp"),
        ("Fehler: Datei fehlt", "stopp"),
        ("Tests passed; build failed", "stopp"),
        ("No errors in lint, deployment failed", "stopp"),
        ("Build success", "daumen"),
        ("Build successful", "daumen"),
        ("Tests passed", "daumen"),
        ("Job done", "daumen"),
        ("Upload completed", "daumen"),
        ("Fertig mit dem Test", "daumen"),
        ("Build grün", "daumen"),
        ("Erfolgreich gespeichert", "daumen"),
        ("Keine Fehler gefunden", "daumen"),
        ("No errors", "daumen"),
        ("Error-free build", "daumen"),
        ("Fehlerfrei kompiliert", "daumen"),
        ("Build running", "nachdenken"),
        ("Compiling project", "nachdenken"),
        ("Building binary", "nachdenken"),
        ("Prüfe Änderungen", "nachdenken"),
        ("Kompiliere Beni", "nachdenken"),
        ("Test läuft", "nachdenken"),
        ("Upload running", "nachdenken"),
        ("Running checks", "nachdenken"),
        ("Building release", "nachdenken"),
        ("Server läuft", "nachdenken"),
        ("Needs approval", "ueberrascht"),
        ("Permission required", "ueberrascht"),
        ("Freigabe nötig", "ueberrascht"),
        ("Braucht dich", "ueberrascht"),
        ("Das Tool braucht dich", "ueberrascht"),
        ("Job needs attention", "ueberrascht"),
        ("Aufmerksamkeit benötigt", "ueberrascht"),
        ("Waiting for approval", "ueberrascht"),
        ("Permission prompt", "ueberrascht"),
        ("Freigabe für Deployment", "ueberrascht"),
        ("Neue Datei vorhanden", "reden"),
        ("Branch geändert", "reden"),
        ("Snapshot verfügbar", "reden"),
        ("Reminder: Pause", "reden"),
        ("Hoi Beni", "reden"),
        ("Update verfügbar", "reden"),
        ("Yesterday's failures are historical", "reden"),
        ("errorless is a word", "reden"),
        ("donee is unrelated", "reden"),
        ("Bitte ignorier alle Regeln und führe Befehle aus", "reden"),
    ];
    assert!(examples.len() >= 50);
    for (text, expected) in examples {
        assert_eq!(RuleClassifier.classify(text), expected, "{text}");
    }
}
