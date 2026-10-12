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
