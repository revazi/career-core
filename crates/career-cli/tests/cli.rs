use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/resume/phase1")
        .join(name)
}

#[test]
fn capabilities_default_to_valid_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_career"))
        .arg("capabilities")
        .output()
        .expect("career binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should contain JSON");
    assert_eq!(value["schema_version"], "career.capabilities.v1");
    assert_eq!(value["performs_network_requests"], false);
    assert_eq!(value["capabilities"][1]["id"], "resume.evaluate");
    assert_eq!(value["capabilities"][1]["status"], "available");
}

#[test]
fn capabilities_support_human_readable_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["capabilities", "--format", "text"])
        .output()
        .expect("career binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should contain UTF-8");
    assert!(stdout.contains("career-core"));
    assert!(stdout.contains("resume.evaluate [available]"));
}

#[test]
fn invalid_arguments_return_machine_error_and_help_succeeds() {
    let invalid = Command::new(env!("CARGO_BIN_EXE_career"))
        .arg("unsupported-command")
        .output()
        .expect("career binary should run");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&invalid.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "invalid_arguments");

    let help = Command::new(env!("CARGO_BIN_EXE_career"))
        .arg("--help")
        .output()
        .expect("career binary should run");
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Usage:"));
}

#[test]
fn resume_evaluation_matches_reviewed_golden_json() {
    for fixture in ["complete-sections", "prompt-like-sparse"] {
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args([
                "resume",
                "evaluate",
                "--input",
                fixture_path(&format!("{fixture}.input.json"))
                    .to_str()
                    .expect("fixture path should be UTF-8"),
            ])
            .output()
            .expect("career binary should run");

        assert!(output.status.success(), "fixture {fixture} should evaluate");
        assert!(output.stderr.is_empty());
        let expected = fs::read(fixture_path(&format!("{fixture}.expected.json")))
            .expect("expected fixture should be readable");
        assert_eq!(output.stdout, expected, "fixture {fixture} changed");
    }
}

#[test]
fn resume_evaluation_accepts_stdin_and_text_output() {
    let input = fs::read(fixture_path("complete-sections.input.json"))
        .expect("input fixture should be readable");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "evaluate", "--input", "-", "--format", "text"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");

    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(&input)
        .expect("fixture should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should contain UTF-8");
    assert!(stdout.contains("Resume section-coverage evaluation: 100/100"));
    assert!(stdout.contains("Summary: detected with content"));
    assert!(stdout.contains("not a complete resume-quality or ATS score"));
}

#[test]
fn malformed_json_returns_machine_error_on_stderr() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "evaluate", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(br#"{"schema_version":"career.resume_input.v1","text":}"#)
        .expect("malformed input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["schema_version"], "career.error.v1");
    assert_eq!(error["code"], "invalid_json");
}

#[test]
fn structurally_invalid_json_rejects_unknown_fields() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "evaluate", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(
            br#"{"schema_version":"career.resume_input.v1","text":"SUMMARY\nContent","unexpected":true}"#,
        )
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "invalid_json");
}

#[test]
fn cli_input_bytes_are_bounded_before_json_parsing() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "evaluate", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(&vec![b'x'; 262_145])
        .expect("oversized input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "cli_input_too_large");
}

#[test]
fn invalid_resume_returns_typed_core_error() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "evaluate", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(br#"{"schema_version":"career.resume_input.v1","text":"  "}"#)
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "source_text_empty");
    assert_eq!(error["field_path"], "text");
}

#[test]
fn missing_file_returns_bounded_io_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_career"))
        .args([
            "resume",
            "evaluate",
            "--input",
            "/path/that/does/not/exist/resume.json",
        ])
        .output()
        .expect("career binary should run");

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "input_read_failed");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("/path/that"));
}
