use std::process::Command;

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
    assert!(stdout.contains("core.capabilities [available]"));
}
