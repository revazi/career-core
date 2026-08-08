use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn schema_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas")
        .join(name)
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/resume/phase1")
        .join(name)
}

fn phase2_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/resume/phase2")
        .join(name)
}

fn phase3_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/resume/phase3")
        .join(name)
}

fn phase7_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/resume/phase7")
        .join(name)
}

fn phase4a_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/job/phase4a")
        .join(name)
}

fn phase4b_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/job/phase4b")
        .join(name)
}

fn phase8_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/managed-adapter/phase8")
        .join(name)
}

fn collect_schema_references<'a>(value: &'a serde_json::Value, references: &mut Vec<&'a str>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(reference) = object.get("$ref") {
                references.push(reference.as_str().expect("$ref should be a string"));
            }
            for child in object.values() {
                collect_schema_references(child, references);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                collect_schema_references(child, references);
            }
        }
        _ => {}
    }
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
    assert_eq!(value["capabilities"][2]["id"], "resume.analyze");
    assert_eq!(value["capabilities"][2]["status"], "available");
    assert_eq!(value["capabilities"][3]["id"], "resume.normalize");
    assert_eq!(value["capabilities"][3]["status"], "available");
    assert_eq!(value["capabilities"][4]["id"], "resume.enrich");
    assert_eq!(value["capabilities"][4]["status"], "available");
    assert_eq!(
        value["capabilities"][5]["id"],
        "resume.analysis-suggestions.review"
    );
    assert_eq!(value["capabilities"][5]["status"], "available");
    assert_eq!(
        value["capabilities"][6]["id"],
        "resume.analysis-replacements.review"
    );
    assert_eq!(value["capabilities"][6]["status"], "available");
    assert_eq!(value["capabilities"][7]["id"], "resume.variant.review");
    assert_eq!(value["capabilities"][7]["status"], "available");
    assert_eq!(value["capabilities"][8]["id"], "resume.variant.materialize");
    assert_eq!(value["capabilities"][8]["status"], "available");
    assert_eq!(value["capabilities"][9]["id"], "job.normalize");
    assert_eq!(value["capabilities"][9]["status"], "available");
    assert_eq!(value["capabilities"][10]["id"], "job.match");
    assert_eq!(value["capabilities"][10]["status"], "available");
    assert_eq!(
        output.stdout,
        fs::read(phase8_fixture_path("capabilities.pre-phase8.expected.json"))
            .expect("pre-Phase 8 capabilities golden should load")
    );
}

#[test]
fn operation_catalog_matches_golden_and_maps_available_capabilities_once() {
    let output = Command::new(env!("CARGO_BIN_EXE_career"))
        .arg("operations")
        .output()
        .expect("career binary should run");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        fs::read(phase8_fixture_path("operation-catalog.expected.json"))
            .expect("operation catalog golden should load")
    );

    let catalog: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("operation catalog should be JSON");
    assert_eq!(catalog["schema_version"], "career.operation_catalog.v1");
    assert_eq!(catalog["core_version"], env!("CARGO_PKG_VERSION"));
    let operations = catalog["operations"]
        .as_array()
        .expect("operations should be an array");
    let operation_ids = operations
        .iter()
        .map(|operation| {
            operation["operation_id"]
                .as_str()
                .expect("operation ID should be a string")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(operation_ids.len(), operations.len());
    assert!(operation_ids.contains("core.operations"));
    assert!(operation_ids.contains("schema.list"));
    assert!(operation_ids.contains("schema.export"));
    assert!(operation_ids.contains("schema.bundle"));

    let capabilities_output = Command::new(env!("CARGO_BIN_EXE_career"))
        .arg("capabilities")
        .output()
        .expect("career binary should run");
    let capabilities: serde_json::Value =
        serde_json::from_slice(&capabilities_output.stdout).expect("capabilities should be JSON");
    let available_capability_ids = capabilities["capabilities"]
        .as_array()
        .expect("capabilities should be an array")
        .iter()
        .filter(|capability| capability["status"] == "available")
        .map(|capability| {
            capability["id"]
                .as_str()
                .expect("capability ID should be a string")
        })
        .collect::<Vec<_>>();
    let mapped_capability_ids = operations
        .iter()
        .filter_map(|operation| operation["capability_id"].as_str())
        .collect::<Vec<_>>();

    assert_eq!(mapped_capability_ids, available_capability_ids);
    for operation in operations {
        assert_eq!(operation["availability"], "available");
        assert_eq!(
            operation["maximum_successful_machine_output_bytes"],
            33_554_432
        );
        if operation["input_transport"] == "json_file_or_stdin" {
            assert!(operation["input_schema_id"].is_string());
            assert!(operation["maximum_input_bytes"].is_u64());
        }
    }
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
    assert!(stdout.contains("resume.analyze [available]"));
    assert!(stdout.contains("resume.normalize [available]"));
    assert!(stdout.contains("resume.enrich [available]"));
    assert!(stdout.contains("resume.analysis-suggestions.review [available]"));
    assert!(stdout.contains("resume.analysis-replacements.review [available]"));
    assert!(stdout.contains("resume.variant.review [available]"));
    assert!(stdout.contains("resume.variant.materialize [available]"));
    assert!(stdout.contains("job.normalize [available]"));
    assert!(stdout.contains("job.match [available]"));
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

    let invalid_schema = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["schema", "export", "--id", "career.unsupported.v1"])
        .output()
        .expect("career binary should run");
    assert_eq!(invalid_schema.status.code(), Some(2));
    assert!(invalid_schema.stdout.is_empty());
    let schema_error: serde_json::Value =
        serde_json::from_slice(&invalid_schema.stderr).expect("stderr should contain JSON");
    assert_eq!(schema_error["code"], "invalid_arguments");

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
fn resume_analysis_matches_reviewed_golden_json() {
    for fixture in ["complete-analysis", "messy-analysis"] {
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args([
                "resume",
                "analyze",
                "--input",
                phase3_fixture_path(&format!("{fixture}.input.json"))
                    .to_str()
                    .expect("fixture path should be UTF-8"),
            ])
            .output()
            .expect("career binary should run");

        assert!(output.status.success(), "fixture {fixture} should analyze");
        assert!(output.stderr.is_empty());
        let expected = fs::read(phase3_fixture_path(&format!("{fixture}.expected.json")))
            .expect("expected fixture should be readable");
        assert_eq!(output.stdout, expected, "fixture {fixture} changed");
    }
}

#[test]
fn resume_analysis_accepts_stdin_and_text_output() {
    let input = fs::read(phase3_fixture_path("messy-analysis.input.json"))
        .expect("input fixture should be readable");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "analyze", "--input", "-", "--format", "text"])
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
    assert!(stdout.contains("Deterministic resume analysis: 62/100"));
    assert!(stdout.contains("Parser confidence: low (32/100)"));
    assert!(stdout.contains("Experience impact could not be verified [provisional]"));
    assert!(stdout.contains("not a reproduction of a proprietary ATS ranking"));
}

#[test]
fn job_normalization_matches_reviewed_golden_json() {
    for fixture in ["complete-normalization", "prose-heavy"] {
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args([
                "job",
                "normalize",
                "--input",
                phase4a_fixture_path(&format!("{fixture}.input.json"))
                    .to_str()
                    .expect("fixture path should be UTF-8"),
            ])
            .output()
            .expect("career binary should run");

        assert!(
            output.status.success(),
            "fixture {fixture} should normalize"
        );
        assert!(output.stderr.is_empty());
        let expected = fs::read(phase4a_fixture_path(&format!("{fixture}.expected.json")))
            .expect("expected fixture should be readable");
        assert_eq!(output.stdout, expected, "fixture {fixture} changed");
    }
}

#[test]
fn job_normalization_accepts_stdin_and_text_output() {
    let input = fs::read(phase4a_fixture_path("prose-heavy.input.json"))
        .expect("input fixture should be readable");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["job", "normalize", "--input", "-", "--format", "text"])
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
    assert!(stdout.contains("Job-description normalization: low confidence (14/100)"));
    assert!(stdout.contains("Required skills: 0; preferred skills: 0; responsibilities: 0"));
    assert!(stdout.contains("downstream matching must not turn unverified fields"));
}

#[test]
fn job_matching_matches_reviewed_golden_json() {
    for fixture in ["complete-match", "vague-job"] {
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args([
                "job",
                "match",
                "--input",
                phase4b_fixture_path(&format!("{fixture}.input.json"))
                    .to_str()
                    .expect("fixture path should be UTF-8"),
            ])
            .output()
            .expect("career binary should run");

        assert!(output.status.success(), "fixture {fixture} should match");
        assert!(output.stderr.is_empty());
        let expected = fs::read(phase4b_fixture_path(&format!("{fixture}.expected.json")))
            .expect("expected fixture should be readable");
        assert_eq!(output.stdout, expected, "fixture {fixture} changed");
    }
}

#[test]
fn job_matching_accepts_stdin_and_marks_uncertain_text_output_provisional() {
    let input = fs::read(phase4b_fixture_path("vague-job.input.json"))
        .expect("input fixture should be readable");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["job", "match", "--input", "-", "--format", "text"])
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
    assert!(stdout.contains("Deterministic resume-to-job match: 61/100"));
    assert!(stdout.contains("Normalization confidence: resume high; job low; uncertain: true"));
    assert!(stdout.contains("Recommendation: apply after small edits [provisional]"));
    assert!(stdout.contains("not a hiring prediction"));
    assert!(!stdout.contains("Top gaps:"));
}

#[test]
fn invalid_job_match_input_prefixes_nested_error_without_payload_echo() {
    let secret = "Secret unsupported resume payload";
    let mut input: serde_json::Value = serde_json::from_slice(
        &fs::read(phase4b_fixture_path("complete-match.input.json"))
            .expect("input fixture should be readable"),
    )
    .expect("fixture should be JSON");
    input["resume"]["text"] = serde_json::Value::String(secret.repeat(2_000));
    let input = serde_json::to_vec(&input).expect("input should serialize");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["job", "match", "--input", "-"])
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
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "source_text_too_large");
    assert_eq!(error["field_path"], "resume.text");
    assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
}

#[test]
fn invalid_job_input_returns_typed_core_error_without_payload_echo() {
    let secret = "Secret unsupported vacancy payload";
    let input = serde_json::json!({
        "schema_version": "career.job_input.v1",
        "text": secret.repeat(2_000),
    })
    .to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["job", "normalize", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "source_text_too_large");
    assert_eq!(error["field_path"], "text");
    assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
}

#[test]
fn resume_normalization_matches_reviewed_golden_json() {
    for fixture in ["complete-normalization", "messy-unlabeled"] {
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args([
                "resume",
                "normalize",
                "--input",
                phase2_fixture_path(&format!("{fixture}.input.json"))
                    .to_str()
                    .expect("fixture path should be UTF-8"),
            ])
            .output()
            .expect("career binary should run");

        assert!(
            output.status.success(),
            "fixture {fixture} should normalize"
        );
        assert!(output.stderr.is_empty());
        let expected = fs::read(phase2_fixture_path(&format!("{fixture}.expected.json")))
            .expect("expected fixture should be readable");
        assert_eq!(output.stdout, expected, "fixture {fixture} changed");
    }
}

#[test]
fn resume_normalization_accepts_stdin_and_text_output() {
    let input = fs::read(phase2_fixture_path("messy-unlabeled.input.json"))
        .expect("input fixture should be readable");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "normalize", "--input", "-", "--format", "text"])
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
    assert!(stdout.contains("Resume normalization: low confidence (32/100)"));
    assert!(stdout.contains("External enrichment: eligible"));
    assert!(stdout.contains("targets: Summary, Experience, Skills"));
}

#[test]
fn resume_enrichment_matches_reviewed_golden_and_text_output() {
    let input_path = phase2_fixture_path("messy-unlabeled.enrichment-input.json");
    let output = Command::new(env!("CARGO_BIN_EXE_career"))
        .args([
            "resume",
            "enrich",
            "--input",
            input_path.to_str().expect("fixture path should be UTF-8"),
        ])
        .output()
        .expect("career binary should run");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let expected = fs::read(phase2_fixture_path(
        "messy-unlabeled.enrichment-expected.json",
    ))
    .expect("expected fixture should be readable");
    assert_eq!(output.stdout, expected);

    let text = Command::new(env!("CARGO_BIN_EXE_career"))
        .args([
            "resume",
            "enrich",
            "--input",
            input_path.to_str().expect("fixture path should be UTF-8"),
            "--format",
            "text",
        ])
        .output()
        .expect("career binary should run");
    assert!(text.status.success());
    let stdout = String::from_utf8(text.stdout).expect("stdout should contain UTF-8");
    assert!(stdout.contains("Resume external enrichment: applied"));
    assert!(stdout.contains("Summary: external source-grounded proposal"));
    assert!(stdout.contains("Deterministic confidence preserved: low (32/100)"));
}

#[test]
fn resume_analysis_suggestion_and_variant_reviews_match_goldens_and_text_output() {
    for (command, input_name, expected_name, text_marker) in [
        (
            "analysis-suggestions-review",
            "complete-analysis-suggestion-review.input.json",
            "complete-analysis-suggestion-review.expected.json",
            "Assisted resume analysis suggestion review: 1 retained; 0 discarded",
        ),
        (
            "analysis-replacements-review",
            "complete-analysis-replacement-review.input.json",
            "complete-analysis-replacement-review.expected.json",
            "Assisted resume analysis replacement review: 1 retained; 0 discarded",
        ),
        (
            "variant-review",
            "complete-variant-review.input.json",
            "complete-variant-review.expected.json",
            "Assisted resume variant review: 2 retained; 0 discarded",
        ),
        (
            "variant-materialize",
            "selected-variant-materialization.input.json",
            "selected-variant-materialization.expected.json",
            "Assisted resume variant materialized: 1 selected changes",
        ),
    ] {
        let input_path = phase7_fixture_path(input_name);
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args([
                "resume",
                command,
                "--input",
                input_path.to_str().expect("fixture path should be UTF-8"),
            ])
            .output()
            .expect("career binary should run");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let expected =
            fs::read(phase7_fixture_path(expected_name)).expect("expected fixture should load");
        assert_eq!(output.stdout, expected);

        let text = Command::new(env!("CARGO_BIN_EXE_career"))
            .args([
                "resume",
                command,
                "--input",
                input_path.to_str().expect("fixture path should be UTF-8"),
                "--format",
                "text",
            ])
            .output()
            .expect("career binary should run");
        assert!(text.status.success());
        let stdout = String::from_utf8(text.stdout).expect("stdout should be UTF-8");
        assert!(stdout.contains(text_marker));
        assert!(stdout.contains("Authority: assisted, non-authoritative"));
        assert!(
            stdout.contains("does not certify") || stdout.contains("does not establish"),
            "text warning should retain the factuality boundary"
        );
    }
}

#[test]
fn discarded_analysis_suggestion_does_not_echo_untrusted_payload() {
    let input = fs::read_to_string(phase7_fixture_path(
        "complete-analysis-suggestion-review.input.json",
    ))
    .expect("fixture should load")
    .replace(
        "Review this bullet and add a specific, source-supported outcome if one can be verified.",
        "Private discarded analysis suggestion.",
    )
    .replace(
        "Built reliable APIs for internal teams.\"\n        ],",
        "private unmatched evidence\"\n        ],",
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "analysis-suggestions-review", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(result["suggestions"], serde_json::json!([]));
    assert_eq!(
        result["discarded_suggestions"][0]["code"],
        "invalid_source_evidence"
    );
    let encoded = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(!encoded.contains("Private discarded analysis suggestion."));
    assert!(!encoded.contains("private unmatched evidence"));
}

#[test]
fn discarded_analysis_replacement_does_not_echo_untrusted_payload_and_unknown_fields_fail() {
    let input = fs::read_to_string(phase7_fixture_path(
        "complete-analysis-replacement-review.input.json",
    ))
    .expect("fixture should load")
    .replace(
        "- Built reliable APIs for internal teams, improving deployment reliability by 30 percent.",
        "Private discarded analysis replacement.",
    )
    .replace(
        "Reduced deployment failures by 30 percent.\"\n        ],",
        "private unmatched evidence\"\n        ],",
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "analysis-replacements-review", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(result["replacements"], serde_json::json!([]));
    assert_eq!(
        result["discarded_replacements"][0]["code"],
        "invalid_source_evidence"
    );
    let encoded = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(!encoded.contains("Private discarded analysis replacement."));
    assert!(!encoded.contains("private unmatched evidence"));

    let unknown_field_input = fs::read_to_string(phase7_fixture_path(
        "complete-analysis-replacement-review.input.json",
    ))
    .expect("fixture should load")
    .replace(
        "\"proposed_replacement\":",
        "\"provider_score\": 100, \"proposed_replacement\":",
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "analysis-replacements-review", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(unknown_field_input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should be JSON");
    assert_eq!(error["code"], "invalid_json");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("career.resume_analysis_replacement_review_input.v1")
    );
}

#[test]
fn invalid_variant_selection_returns_bounded_core_error() {
    let input = fs::read_to_string(phase7_fixture_path(
        "selected-variant-materialization.input.json",
    ))
    .expect("fixture should load")
    .replace("change-0002", "private-provider-change-id");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "variant-materialize", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "variant_selection_invalid");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private-provider-change-id"));
}

#[test]
fn enrichment_proposal_rejects_unknown_json_fields() {
    let input = fs::read_to_string(phase2_fixture_path("messy-unlabeled.enrichment-input.json"))
        .expect("input fixture should be readable")
        .replace("\"summary\":", "\"unexpected\": true, \"summary\":");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "enrich", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "invalid_json");
    assert!(String::from_utf8_lossy(&output.stderr).contains("career.resume_enrichment_input.v1"));
}

#[test]
fn unsupported_enrichment_value_returns_bounded_core_error() {
    let input = fs::read_to_string(phase2_fixture_path("messy-unlabeled.enrichment-input.json"))
        .expect("input fixture should be readable")
        .replace("\"Python\"", "\"Unsupported Secret Skill\"");
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "enrich", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "enrichment_value_not_grounded");
    assert_eq!(error["field_path"], "proposal.skills[0]");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Unsupported Secret Skill"));
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
fn analysis_replacement_cli_input_bytes_are_bounded_before_json_parsing() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["resume", "analysis-replacements-review", "--input", "-"])
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
        serde_json::from_slice(&output.stderr).expect("stderr should be JSON");
    assert_eq!(error["code"], "cli_input_too_large");
}

#[test]
fn job_match_cli_input_bytes_use_a_bounded_composite_limit() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["job", "match", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("career binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(&vec![b'x'; 1_048_577])
        .expect("oversized input should be written");
    let output = child
        .wait_with_output()
        .expect("career binary should finish");

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should contain JSON");
    assert_eq!(error["code"], "cli_input_too_large");
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|message| message.contains("1048576 bytes"))
    );
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

#[test]
fn embedded_schema_catalog_lists_and_exports_every_public_schema() {
    let catalog_output = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["schema", "list"])
        .output()
        .expect("career binary should run");
    assert!(catalog_output.status.success());
    assert!(catalog_output.stderr.is_empty());

    let catalog: serde_json::Value =
        serde_json::from_slice(&catalog_output.stdout).expect("catalog should be JSON");
    assert_eq!(catalog["schema_version"], "career.schema_catalog.v1");
    let schemas = catalog["schemas"]
        .as_array()
        .expect("catalog schemas should be an array");
    let schema_ids = schemas
        .iter()
        .map(|entry| entry["id"].as_str().expect("schema ID should be a string"))
        .collect::<Vec<_>>();
    assert_eq!(
        schema_ids,
        vec![
            "career.capabilities.v1",
            "career.error.v1",
            "career.job_input.v1",
            "career.job_match.v1",
            "career.job_match_input.v1",
            "career.job_normalization.v1",
            "career.resume_analysis.v1",
            "career.resume_analysis_replacement_proposal.v1",
            "career.resume_analysis_replacement_review.v1",
            "career.resume_analysis_replacement_review_input.v1",
            "career.resume_analysis_suggestion_proposal.v1",
            "career.resume_analysis_suggestion_review.v1",
            "career.resume_analysis_suggestion_review_input.v1",
            "career.resume_enrichment_input.v1",
            "career.resume_enrichment_proposal.v1",
            "career.resume_enrichment_result.v1",
            "career.resume_evaluation.v1",
            "career.resume_input.v1",
            "career.resume_normalization.v1",
            "career.resume_variant_materialization_input.v1",
            "career.resume_variant_proposal.v1",
            "career.resume_variant_review.v1",
            "career.resume_variant_review_input.v1",
            "career.resume_variant.v1",
            "career.schema_catalog.v1",
            "career.operation_catalog.v1",
        ]
    );

    for entry in schemas {
        let id = entry["id"].as_str().expect("schema ID should be a string");
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args(["schema", "export", "--id", id])
            .output()
            .expect("career binary should run");
        assert!(output.status.success(), "schema {id} should export");
        assert!(output.stderr.is_empty());
        let file_name = entry["file_name"]
            .as_str()
            .expect("schema file name should be a string");
        assert_eq!(
            output.stdout,
            fs::read(schema_path(file_name)).expect("reviewed schema should be readable"),
            "schema export should preserve reviewed bytes for {id}"
        );
        let schema: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("export should be JSON");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["title"], entry["title"]);
        assert_eq!(schema["properties"]["schema_version"]["const"], id);

        let first_bundle = Command::new(env!("CARGO_BIN_EXE_career"))
            .args(["schema", "bundle", "--id", id, "--format", "json-compact"])
            .output()
            .expect("career binary should run");
        let second_bundle = Command::new(env!("CARGO_BIN_EXE_career"))
            .args(["schema", "bundle", "--id", id, "--format", "json-compact"])
            .output()
            .expect("career binary should run");
        assert!(first_bundle.status.success(), "schema {id} should bundle");
        assert!(first_bundle.stderr.is_empty());
        assert_eq!(first_bundle.stdout, second_bundle.stdout);
        let bundle: serde_json::Value =
            serde_json::from_slice(&first_bundle.stdout).expect("schema bundle should be JSON");
        assert_eq!(
            bundle["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        let mut references = Vec::new();
        collect_schema_references(&bundle, &mut references);
        for reference in references {
            assert!(
                reference.starts_with('#'),
                "non-local bundle ref: {reference}"
            );
            assert!(
                bundle
                    .pointer(
                        reference
                            .strip_prefix('#')
                            .expect("local ref should start #")
                    )
                    .is_some(),
                "unresolved bundle ref: {reference}"
            );
        }
    }

    let text = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["schema", "list", "--format", "text"])
        .output()
        .expect("career binary should run");
    assert!(text.status.success());
    assert!(String::from_utf8_lossy(&text.stdout).contains("career.job_match.v1"));
}

#[test]
fn explicit_pretty_json_preserves_canonical_default_output() {
    let default_output = Command::new(env!("CARGO_BIN_EXE_career"))
        .arg("capabilities")
        .output()
        .expect("career binary should run");
    let explicit_output = Command::new(env!("CARGO_BIN_EXE_career"))
        .args(["capabilities", "--format", "json-pretty"])
        .output()
        .expect("career binary should run");

    assert!(default_output.status.success());
    assert!(explicit_output.status.success());
    assert_eq!(explicit_output.stdout, default_output.stdout);
    assert_eq!(explicit_output.stderr, default_output.stderr);
}

#[test]
fn every_machine_operation_supports_one_line_compact_json() {
    let commands = vec![
        vec!["capabilities".to_owned()],
        vec!["operations".to_owned()],
        vec!["schema".to_owned(), "list".to_owned()],
        vec![
            "schema".to_owned(),
            "export".to_owned(),
            "--id".to_owned(),
            "career.job_match.v1".to_owned(),
        ],
        vec![
            "schema".to_owned(),
            "bundle".to_owned(),
            "--id".to_owned(),
            "career.job_match.v1".to_owned(),
        ],
        vec![
            "resume".to_owned(),
            "evaluate".to_owned(),
            "--input".to_owned(),
            fixture_path("complete-sections.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "resume".to_owned(),
            "analyze".to_owned(),
            "--input".to_owned(),
            phase3_fixture_path("complete-analysis.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "resume".to_owned(),
            "analysis-replacements-review".to_owned(),
            "--input".to_owned(),
            phase7_fixture_path("complete-analysis-replacement-review.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "resume".to_owned(),
            "analysis-suggestions-review".to_owned(),
            "--input".to_owned(),
            phase7_fixture_path("complete-analysis-suggestion-review.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "resume".to_owned(),
            "normalize".to_owned(),
            "--input".to_owned(),
            phase2_fixture_path("complete-normalization.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "resume".to_owned(),
            "enrich".to_owned(),
            "--input".to_owned(),
            phase2_fixture_path("messy-unlabeled.enrichment-input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "resume".to_owned(),
            "variant-review".to_owned(),
            "--input".to_owned(),
            phase7_fixture_path("complete-variant-review.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "resume".to_owned(),
            "variant-materialize".to_owned(),
            "--input".to_owned(),
            phase7_fixture_path("selected-variant-materialization.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "job".to_owned(),
            "normalize".to_owned(),
            "--input".to_owned(),
            phase4a_fixture_path("complete-normalization.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "job".to_owned(),
            "match".to_owned(),
            "--input".to_owned(),
            phase4b_fixture_path("complete-match.input.json")
                .to_string_lossy()
                .into_owned(),
        ],
    ];

    for mut arguments in commands {
        let command_name = arguments.join(" ");
        arguments.extend(["--format".to_owned(), "json-compact".to_owned()]);
        let output = Command::new(env!("CARGO_BIN_EXE_career"))
            .args(&arguments)
            .output()
            .expect("career binary should run");
        assert!(
            output.status.success(),
            "compact command failed: {command_name}"
        );
        assert!(output.stderr.is_empty());
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap_or_else(|error| {
            panic!("compact output was not JSON for {command_name}: {error}")
        });
        assert_eq!(
            output.stdout.iter().filter(|byte| **byte == b'\n').count(),
            1,
            "compact output should be one line for {command_name}"
        );
    }
}

#[test]
fn compact_json_errors_remain_machine_clean() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_career"))
        .args([
            "resume",
            "evaluate",
            "--input",
            "-",
            "--format",
            "json-compact",
        ])
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
    assert_eq!(error["code"], "invalid_json");
    assert_eq!(
        output.stderr.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
}
