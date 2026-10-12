use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_beni-buddy-cli"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn validates_real_embedded_assets() {
    let output = run(&["validate-assets"]);
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("44 Posen, 20 Animationen")
    );
}
#[test]
fn sample_is_valid_json_and_supports_aliases() {
    let output = run(&["sample", "wave", "321"]);
    assert!(output.status.success());
    let pose: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(pose["frame"], "winken_o");
}
#[test]
fn invalid_commands_and_non_finite_time_fail_cleanly() {
    for args in [
        &["unknown"][..],
        &["sample", "unknown"][..],
        &["sample", "wave", "NaN"][..],
    ] {
        let output = run(args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[path = "../../buddy-ipc/tests/common/mod.rs"]
mod common;
#[test]
fn notify_has_distinct_exit_codes_and_delivers_through_real_transport() {
    let endpoint = common::TestEndpoint::new();
    let name = &endpoint.endpoint.0;
    let output = run(&["notify", "ok", "--endpoint", name]);
    assert_eq!(output.status.code(), Some(3));
    let output = run(&["notify", "ok", "--state", "invalid", "--endpoint", name]);
    assert_eq!(output.status.code(), Some(2));
    let server = buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
    let output = run(&[
        "notify",
        "Tests passed",
        "--state",
        "success",
        "--source",
        "test-cli",
        "--endpoint",
        name,
    ]);
    assert!(output.status.success());
    let got = server.pop().unwrap();
    assert_eq!(got.message.source, "test-cli");
    assert_eq!(got.message.state.as_deref(), Some("daumen"));
    for _ in 0..5 {
        assert!(run(&["notify", "ok", "--endpoint", name]).status.success());
    }
    assert_eq!(
        run(&["notify", "ok", "--endpoint", name]).status.code(),
        Some(4)
    );
    for args in [
        vec!["notify", "ok", "--ttl", "0"],
        vec!["notify", "ok", "--source", "a/b"],
        vec!["notify", "ok", "--state"],
        vec!["notify", "ok", "--wat", "value"],
        vec!["notify", "ok", "--state", "reden", "--state", "hallo"],
    ] {
        assert_eq!(run(&args).status.code(), Some(2));
    }
}
#[test]
fn run_preserves_failure_success_and_unavailable_buddy_does_not_block_commands() {
    let endpoint = common::TestEndpoint::new();
    let invoke = |command: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_beni-buddy-cli"))
            .args(["run", "--", env!("CARGO_BIN_EXE_beni-buddy-cli")])
            .args(command)
            .env("BENI_IPC_ENDPOINT", &endpoint.endpoint.0)
            .output()
            .unwrap()
    };
    assert!(invoke(&["validate-assets"]).status.success());
    let server = buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap();
    assert_eq!(invoke(&["sample", "unknown"]).status.code(), Some(2));
    assert_eq!(
        server.pop().unwrap().message.state.as_deref(),
        Some("nachdenken")
    );
    assert_eq!(
        server.pop().unwrap().message.state.as_deref(),
        Some("stopp")
    );
    assert!(invoke(&["validate-assets"]).status.success());
    server.pop();
    assert_eq!(
        server.pop().unwrap().message.state.as_deref(),
        Some("daumen")
    );
}
