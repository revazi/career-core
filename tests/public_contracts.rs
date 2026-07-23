use career_core::{
    JobInputV1, JobNormalizationV1, ResumeAnalysisV1, ResumeEnrichmentInputV1,
    ResumeEnrichmentResultV1, ResumeEvaluationV1, ResumeInputV1, ResumeNormalizationV1,
    analyze_resume, apply_resume_enrichment, evaluate_resume, normalize_job, normalize_resume,
};

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
        "job-input-v1",
        include_str!("../schemas/job-input-v1.schema.json"),
    ),
    (
        "job-normalization-v1",
        include_str!("../schemas/job-normalization-v1.schema.json"),
    ),
    (
        "resume-evaluation-v1",
        include_str!("../schemas/resume-evaluation-v1.schema.json"),
    ),
    (
        "resume-analysis-v1",
        include_str!("../schemas/resume-analysis-v1.schema.json"),
    ),
    (
        "resume-normalization-v1",
        include_str!("../schemas/resume-normalization-v1.schema.json"),
    ),
    (
        "resume-enrichment-proposal-v1",
        include_str!("../schemas/resume-enrichment-proposal-v1.schema.json"),
    ),
    (
        "resume-enrichment-input-v1",
        include_str!("../schemas/resume-enrichment-input-v1.schema.json"),
    ),
    (
        "resume-enrichment-result-v1",
        include_str!("../schemas/resume-enrichment-result-v1.schema.json"),
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
fn reviewed_job_normalization_fixtures_match_the_typed_contract() {
    for (name, input_json, expected_json) in [
        (
            "complete-normalization",
            include_str!("../fixtures/job/phase4a/complete-normalization.input.json"),
            include_str!("../fixtures/job/phase4a/complete-normalization.expected.json"),
        ),
        (
            "prose-heavy",
            include_str!("../fixtures/job/phase4a/prose-heavy.input.json"),
            include_str!("../fixtures/job/phase4a/prose-heavy.expected.json"),
        ),
    ] {
        let input: JobInputV1 = serde_json::from_str(input_json)
            .unwrap_or_else(|error| panic!("invalid {name} input fixture: {error}"));
        let expected: JobNormalizationV1 = serde_json::from_str(expected_json)
            .unwrap_or_else(|error| panic!("invalid {name} expected fixture: {error}"));
        let actual = normalize_job(&input)
            .unwrap_or_else(|error| panic!("{name} normalization failed: {error}"));

        assert_eq!(actual, expected, "typed fixture changed for {name}");
    }
}

#[test]
fn selected_job_normalization_matches_the_independent_reference_fixture() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/job/phase4a/job-description-normalization-v6-reference.json"
    ))
    .expect("job reference projection should be JSON");
    assert_eq!(
        reference["reference_policy_version"],
        "job_description_normalization_v6"
    );
    assert_eq!(reference["comparison_basis"], "identical_source_text");

    for (index, input_json) in [
        include_str!("../fixtures/job/phase4a/complete-normalization.input.json"),
        include_str!("../fixtures/job/phase4a/prose-heavy.input.json"),
    ]
    .into_iter()
    .enumerate()
    {
        let input: JobInputV1 =
            serde_json::from_str(input_json).expect("comparison input should be typed JSON");
        let actual =
            serde_json::to_value(normalize_job(&input).expect("comparison input should normalize"))
                .expect("normalization should serialize");
        let expected = &reference["fixtures"][index];
        let actual_document = &actual["deterministic_document"];
        let expected_document = &expected["document"];

        for field in ["title", "company"] {
            let actual_value = actual_document[field]["value"].as_str().unwrap_or("");
            assert_eq!(
                serde_json::Value::String(actual_value.to_owned()),
                expected_document[field]
            );
        }
        for field in [
            "required_skills",
            "preferred_skills",
            "required_qualifications",
            "preferred_qualifications",
            "seniority_signals",
            "experience_requirements",
            "education_requirements",
            "certification_requirements",
            "responsibilities",
        ] {
            let values = actual_document[field]
                .as_array()
                .expect("actual field should be an array")
                .iter()
                .map(|item| item["value"].clone())
                .collect::<Vec<_>>();
            assert_eq!(serde_json::Value::Array(values), expected_document[field]);
        }

        let actual_sections = actual["metadata"]["matched_sections"]
            .as_array()
            .expect("matched sections should be an array");
        let expected_sections = expected["metadata"]["matched_sections"]
            .as_array()
            .expect("reference sections should be an array");
        assert_eq!(actual_sections.len(), expected_sections.len());
        for (actual_section, expected_section) in actual_sections.iter().zip(expected_sections) {
            assert_eq!(actual_section["section"], expected_section["section"]);
            assert_eq!(
                actual_section["header_source"]["excerpt"],
                expected_section["header"]
            );
            assert_eq!(
                actual_section["header_source"]["start_line"],
                expected_section["line_number"]
            );
        }
        let actual_unmatched = actual["metadata"]["unmatched_lines"]
            .as_array()
            .expect("unmatched lines should be an array");
        let expected_unmatched = expected["metadata"]["unmatched_lines"]
            .as_array()
            .expect("reference unmatched lines should be an array");
        assert_eq!(actual_unmatched.len(), expected_unmatched.len());
        for (actual_line, expected_line) in actual_unmatched.iter().zip(expected_unmatched) {
            assert_eq!(actual_line["line_number"], expected_line["line_number"]);
            assert_eq!(actual_line["excerpt"], expected_line["text"]);
        }
        assert_eq!(
            actual["metadata"]["unmatched_line_count"],
            expected["metadata"]["unmatched_line_count"]
        );
        assert_eq!(
            actual["metadata"]["unmatched_lines_truncated"],
            expected["metadata"]["unmatched_lines_truncated"]
        );
        assert_eq!(
            actual["confidence"]["label"],
            expected["confidence"]["label"]
        );
        assert_eq!(
            actual["confidence"]["score"],
            expected["confidence"]["score"]
        );
        for signal in actual["confidence"]["signals"]
            .as_array()
            .expect("signals should be an array")
        {
            let name = signal["signal"]
                .as_str()
                .expect("signal name should be text");
            assert_eq!(
                signal["score"],
                expected["confidence"]["signal_scores"][name]["score"]
            );
            assert_eq!(
                signal["max_score"],
                expected["confidence"]["signal_scores"][name]["max_score"]
            );
        }
    }
}

#[test]
fn reviewed_resume_evaluation_fixtures_match_the_typed_contract() {
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

#[test]
fn reviewed_resume_normalization_fixtures_match_the_typed_contract() {
    for (name, input_json, expected_json) in [
        (
            "complete-normalization",
            include_str!("../fixtures/resume/phase2/complete-normalization.input.json"),
            include_str!("../fixtures/resume/phase2/complete-normalization.expected.json"),
        ),
        (
            "messy-unlabeled",
            include_str!("../fixtures/resume/phase2/messy-unlabeled.input.json"),
            include_str!("../fixtures/resume/phase2/messy-unlabeled.expected.json"),
        ),
    ] {
        let input: ResumeInputV1 = serde_json::from_str(input_json)
            .unwrap_or_else(|error| panic!("invalid {name} input fixture: {error}"));
        let expected: ResumeNormalizationV1 = serde_json::from_str(expected_json)
            .unwrap_or_else(|error| panic!("invalid {name} expected fixture: {error}"));
        let actual = normalize_resume(&input)
            .unwrap_or_else(|error| panic!("{name} normalization failed: {error}"));

        assert_eq!(actual, expected, "typed fixture changed for {name}");
    }
}

#[test]
fn reviewed_resume_analysis_fixtures_match_the_typed_contract() {
    for (name, input_json, expected_json) in [
        (
            "complete-analysis",
            include_str!("../fixtures/resume/phase3/complete-analysis.input.json"),
            include_str!("../fixtures/resume/phase3/complete-analysis.expected.json"),
        ),
        (
            "messy-analysis",
            include_str!("../fixtures/resume/phase3/messy-analysis.input.json"),
            include_str!("../fixtures/resume/phase3/messy-analysis.expected.json"),
        ),
    ] {
        let input: ResumeInputV1 = serde_json::from_str(input_json)
            .unwrap_or_else(|error| panic!("invalid {name} input fixture: {error}"));
        let expected: ResumeAnalysisV1 = serde_json::from_str(expected_json)
            .unwrap_or_else(|error| panic!("invalid {name} expected fixture: {error}"));
        let actual = analyze_resume(&input)
            .unwrap_or_else(|error| panic!("{name} analysis failed: {error}"));

        assert_eq!(actual, expected, "typed fixture changed for {name}");
    }
}

#[test]
fn selected_resume_analysis_scores_match_the_independent_reference_fixture() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/resume/phase3/deterministic-v2-reference.json"
    ))
    .expect("reference comparison fixture should be JSON");
    assert_eq!(reference["reference_policy_version"], "deterministic_v2");
    assert_eq!(reference["comparison_basis"], "equivalent_normalized_facts");

    for (index, input_json) in [
        include_str!("../fixtures/resume/phase3/complete-analysis.input.json"),
        include_str!("../fixtures/resume/phase3/messy-analysis.input.json"),
    ]
    .into_iter()
    .enumerate()
    {
        let input: ResumeInputV1 =
            serde_json::from_str(input_json).expect("comparison input should be typed JSON");
        let actual =
            serde_json::to_value(analyze_resume(&input).expect("comparison input should analyze"))
                .expect("analysis should serialize");
        let expected = &reference["fixtures"][index];

        assert_eq!(actual["overall_score"], expected["overall_score"]);
        assert_eq!(actual["category_scores"], expected["category_scores"]);
        let actual_checks = actual["checks"]
            .as_array()
            .expect("actual checks should be an array");
        let expected_checks = expected["checks"]
            .as_array()
            .expect("reference checks should be an array");
        assert_eq!(actual_checks.len(), expected_checks.len());
        for (actual_check, expected_check) in actual_checks.iter().zip(expected_checks) {
            assert_eq!(actual_check["check_id"], expected_check["id"]);
            assert_eq!(actual_check["category"], expected_check["category"]);
            assert_eq!(actual_check["score"], expected_check["score"]);
            assert_eq!(actual_check["passed"], expected_check["passed"]);
            assert_eq!(
                actual_check["detection_status"],
                expected_check["detection_status"]
            );
            assert_eq!(actual_check["explanation"], expected_check["details"]);
        }
    }
}

#[test]
fn reviewed_resume_enrichment_fixture_matches_the_typed_contract() {
    let input: ResumeEnrichmentInputV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase2/messy-unlabeled.enrichment-input.json"
    ))
    .expect("enrichment input fixture should be typed JSON");
    let expected: ResumeEnrichmentResultV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase2/messy-unlabeled.enrichment-expected.json"
    ))
    .expect("enrichment expected fixture should be typed JSON");
    let actual = apply_resume_enrichment(&input).expect("enrichment should apply");

    assert_eq!(actual, expected);
    assert_eq!(
        actual.baseline,
        normalize_resume(&input.resume).expect("baseline normalization should repeat")
    );
}
