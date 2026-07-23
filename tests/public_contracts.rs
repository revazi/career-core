use career_core::{ResumeEvaluationV1, ResumeInputV1, evaluate_resume};

const SCHEMAS: &[(&str, &str)] = &[
    (
        "capabilities-v1",
        include_str!("../schemas/capabilities-v1.schema.json"),
    ),
    (
        "resume-input-v1",
        include_str!("../schemas/resume-input-v1.schema.json"),
    ),
    (
        "resume-evaluation-v1",
        include_str!("../schemas/resume-evaluation-v1.schema.json"),
    ),
    ("error-v1", include_str!("../schemas/error-v1.schema.json")),
];

#[test]
fn public_schema_documents_are_valid_json_schema_objects() {
    for (name, schema) in SCHEMAS {
        let value: serde_json::Value =
            serde_json::from_str(schema).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            value["$schema"], "https://json-schema.org/draft/2020-12/schema",
            "schema marker changed for {name}"
        );
        assert!(value["$id"].is_string(), "schema ID missing for {name}");
        assert_eq!(value["type"], "object", "root type changed for {name}");
    }
}

#[test]
fn reviewed_resume_fixtures_match_the_typed_contract() {
    for (name, input_json, expected_json) in [
        (
            "complete-sections",
            include_str!("../fixtures/resume/phase1/complete-sections.input.json"),
            include_str!("../fixtures/resume/phase1/complete-sections.expected.json"),
        ),
        (
            "prompt-like-sparse",
            include_str!("../fixtures/resume/phase1/prompt-like-sparse.input.json"),
            include_str!("../fixtures/resume/phase1/prompt-like-sparse.expected.json"),
        ),
    ] {
        let input: ResumeInputV1 = serde_json::from_str(input_json)
            .unwrap_or_else(|error| panic!("invalid {name} input fixture: {error}"));
        let expected: ResumeEvaluationV1 = serde_json::from_str(expected_json)
            .unwrap_or_else(|error| panic!("invalid {name} expected fixture: {error}"));
        let actual = evaluate_resume(&input)
            .unwrap_or_else(|error| panic!("{name} evaluation failed: {error}"));

        assert_eq!(actual, expected, "typed fixture changed for {name}");
    }
}
