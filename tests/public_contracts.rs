use career_core::{
    JobInputV1, JobMatchCategoryV1, JobMatchInputV1, JobMatchItemKindV1, JobMatchItemStatusV1,
    JobMatchRecommendationLabelV1, JobMatchRecommendationStatusV1, JobMatchTypeV1, JobMatchV1,
    JobNormalizationV1, ResumeAnalysisReplacementReviewInputV1, ResumeAnalysisReplacementReviewV1,
    ResumeAnalysisSuggestionReviewInputV1, ResumeAnalysisSuggestionReviewV1, ResumeAnalysisV1,
    ResumeEnrichmentInputV1, ResumeEnrichmentResultV1, ResumeEvaluationV1, ResumeInputV1,
    ResumeNormalizationV1, ResumeVariantMaterializationInputV1, ResumeVariantReviewInputV1,
    ResumeVariantReviewV1, ResumeVariantV1, analyze_resume, apply_resume_enrichment,
    evaluate_resume, match_job, materialize_resume_variant, normalize_job, normalize_resume,
    review_resume_analysis_replacements, review_resume_analysis_suggestions, review_resume_variant,
};

const SCHEMAS: &[(&str, &str)] = &[
    (
        "capabilities-v1",
        include_str!("../schemas/capabilities-v1.schema.json"),
    ),
    (
        "schema-catalog-v1",
        include_str!("../schemas/schema-catalog-v1.schema.json"),
    ),
    (
        "operation-catalog-v1",
        include_str!("../schemas/operation-catalog-v1.schema.json"),
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
        "job-match-input-v1",
        include_str!("../schemas/job-match-input-v1.schema.json"),
    ),
    (
        "job-match-v1",
        include_str!("../schemas/job-match-v1.schema.json"),
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
        "resume-analysis-replacement-proposal-v1",
        include_str!("../schemas/resume-analysis-replacement-proposal-v1.schema.json"),
    ),
    (
        "resume-analysis-replacement-review-v1",
        include_str!("../schemas/resume-analysis-replacement-review-v1.schema.json"),
    ),
    (
        "resume-analysis-replacement-review-input-v1",
        include_str!("../schemas/resume-analysis-replacement-review-input-v1.schema.json"),
    ),
    (
        "resume-analysis-suggestion-proposal-v1",
        include_str!("../schemas/resume-analysis-suggestion-proposal-v1.schema.json"),
    ),
    (
        "resume-analysis-suggestion-review-v1",
        include_str!("../schemas/resume-analysis-suggestion-review-v1.schema.json"),
    ),
    (
        "resume-analysis-suggestion-review-input-v1",
        include_str!("../schemas/resume-analysis-suggestion-review-input-v1.schema.json"),
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
    (
        "resume-variant-proposal-v1",
        include_str!("../schemas/resume-variant-proposal-v1.schema.json"),
    ),
    (
        "resume-variant-review-input-v1",
        include_str!("../schemas/resume-variant-review-input-v1.schema.json"),
    ),
    (
        "resume-variant-review-v1",
        include_str!("../schemas/resume-variant-review-v1.schema.json"),
    ),
    (
        "resume-variant-materialization-input-v1",
        include_str!("../schemas/resume-variant-materialization-input-v1.schema.json"),
    ),
    (
        "resume-variant-v1",
        include_str!("../schemas/resume-variant-v1.schema.json"),
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
fn reviewed_job_match_fixtures_match_the_typed_contract() {
    for (name, input_json, expected_json) in [
        (
            "complete-match",
            include_str!("../fixtures/job/phase4b/complete-match.input.json"),
            include_str!("../fixtures/job/phase4b/complete-match.expected.json"),
        ),
        (
            "vague-job",
            include_str!("../fixtures/job/phase4b/vague-job.input.json"),
            include_str!("../fixtures/job/phase4b/vague-job.expected.json"),
        ),
    ] {
        let input: JobMatchInputV1 = serde_json::from_str(input_json)
            .unwrap_or_else(|error| panic!("invalid {name} input fixture: {error}"));
        let expected: JobMatchV1 = serde_json::from_str(expected_json)
            .unwrap_or_else(|error| panic!("invalid {name} expected fixture: {error}"));
        let actual =
            match_job(&input).unwrap_or_else(|error| panic!("{name} matching failed: {error}"));

        assert_eq!(actual, expected, "typed fixture changed for {name}");
    }
}

#[test]
fn job_match_input_rejects_assisted_or_caller_supplied_normalized_documents() {
    let mut input: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/complete-match.input.json"
    ))
    .expect("match input should be JSON");
    input["resume"]["assisted_document"] = serde_json::json!({"skills": ["invented"]});
    assert!(serde_json::from_value::<JobMatchInputV1>(input).is_err());

    let mut input: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/complete-match.input.json"
    ))
    .expect("match input should be JSON");
    input["job"]["deterministic_document"] = serde_json::json!({"required_skills": []});
    assert!(serde_json::from_value::<JobMatchInputV1>(input).is_err());
}

#[test]
fn selected_job_match_scores_match_the_executed_reference_projection() {
    let input: JobMatchInputV1 = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/complete-match.input.json"
    ))
    .expect("comparison input should be typed JSON");
    let actual = match_job(&input).expect("comparison input should match");
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/job-match-deterministic-v2-reference.json"
    ))
    .expect("job-match reference projection should be JSON");

    assert_eq!(
        reference["reference_policy_version"],
        "job_match_deterministic_v2"
    );
    assert_eq!(
        reference["comparison_basis"],
        "equivalent_normalized_facts_from_identical_source_text"
    );
    assert_eq!(
        serde_json::to_value(&actual.scoring_weights).expect("weights should serialize"),
        reference["scoring_weights"]
    );
    assert_eq!(actual.overall_score, reference["overall_score"]);
    assert_eq!(
        serde_json::to_value(&actual.category_scores).expect("scores should serialize"),
        reference["category_scores"]
    );
    for result in &actual.category_results {
        assert_eq!(
            result.raw_score,
            reference["raw_scores"][result.category.as_str()]
        );
    }

    let skills = actual
        .category_results
        .iter()
        .find(|result| result.category == JobMatchCategoryV1::SkillsMatch)
        .expect("skills category should exist");
    let required = skills
        .items
        .iter()
        .filter(|item| item.kind == JobMatchItemKindV1::RequiredSkill)
        .map(|item| {
            serde_json::json!({
                "job_skill": item.item,
                "resume_skill": item.resume_item,
                "match_type": item.match_type,
            })
        })
        .collect::<Vec<_>>();
    let preferred = skills
        .items
        .iter()
        .filter(|item| item.kind == JobMatchItemKindV1::PreferredSkill)
        .map(|item| {
            serde_json::json!({
                "job_skill": item.item,
                "resume_skill": item.resume_item,
                "match_type": item.match_type,
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::Value::Array(required),
        reference["skill_matches"]["required"]["matched"]
    );
    assert_eq!(
        serde_json::Value::Array(preferred),
        reference["skill_matches"]["preferred"]["matched"]
    );

    let metric_value = |category: JobMatchCategoryV1, metric_id: &str| {
        let result = actual
            .category_results
            .iter()
            .find(|result| result.category == category)
            .expect("comparison category should exist");
        result
            .metrics
            .iter()
            .find(|metric| {
                serde_json::to_value(metric.metric_id).expect("metric ID should serialize")
                    == metric_id
            })
            .map(|metric| metric.value.clone())
            .expect("comparison metric should exist")
    };
    assert_eq!(
        metric_value(JobMatchCategoryV1::ExperienceMatch, "required_years"),
        reference["experience"]["required_years"].to_string()
    );
    assert_eq!(
        metric_value(
            JobMatchCategoryV1::ExperienceMatch,
            "estimated_resume_years"
        ),
        reference["experience"]["estimated_resume_years"].to_string()
    );
    assert_eq!(
        metric_value(
            JobMatchCategoryV1::ExperienceMatch,
            "dated_experience_entry_count"
        ),
        reference["experience"]["dated_experience_entries"].to_string()
    );
    assert_eq!(
        metric_value(JobMatchCategoryV1::SeniorityFit, "job_seniority_signals"),
        "senior"
    );
    assert_eq!(
        metric_value(JobMatchCategoryV1::SeniorityFit, "resume_seniority_signals"),
        "senior"
    );

    let domain = actual
        .category_results
        .iter()
        .find(|result| result.category == JobMatchCategoryV1::DomainFit)
        .expect("domain category should exist");
    let mut matched_domains = domain
        .items
        .iter()
        .filter(|item| item.status == JobMatchItemStatusV1::ConfirmedMatch)
        .map(|item| serde_json::Value::String(item.item.clone()))
        .collect::<Vec<_>>();
    matched_domains.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    let reference_domains = reference["domain"]["matched_domains"]
        .as_array()
        .expect("reference domains should be an array")
        .iter()
        .filter(|value| value.as_str() != Some("frontend"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(matched_domains, reference_domains);
    assert!(
        reference["domain"]["matched_domains"]
            .as_array()
            .expect("reference domains should be an array")
            .iter()
            .any(|value| value == "frontend"),
        "the recorded reference difference should remain explicit"
    );

    let keywords = actual
        .category_results
        .iter()
        .find(|result| result.category == JobMatchCategoryV1::KeywordAlignment)
        .expect("keyword category should exist");
    let matched_keywords = keywords
        .items
        .iter()
        .filter(|item| item.status == JobMatchItemStatusV1::ConfirmedMatch)
        .map(|item| serde_json::Value::String(item.item.clone()))
        .collect::<Vec<_>>();
    let missing_keywords = keywords
        .items
        .iter()
        .filter(|item| item.status == JobMatchItemStatusV1::LikelyMissing)
        .map(|item| serde_json::Value::String(item.item.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::Value::Array(matched_keywords),
        reference["keywords"]["matched_keywords"]
    );
    assert_eq!(
        serde_json::Value::Array(missing_keywords),
        reference["keywords"]["missing_keywords"]
    );
    assert_eq!(
        metric_value(JobMatchCategoryV1::EducationFit, "resume_degrees"),
        "bachelor's, degree"
    );
    assert_eq!(
        metric_value(JobMatchCategoryV1::EducationFit, "required_degrees"),
        "bachelor's, degree"
    );
    assert_eq!(
        metric_value(JobMatchCategoryV1::EducationFit, "resume_certifications"),
        "aws certified"
    );
    assert_eq!(
        metric_value(JobMatchCategoryV1::EducationFit, "required_certifications"),
        ""
    );
}

#[test]
fn risky_job_match_fixtures_keep_conservative_boundaries() {
    let expectations: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/risky-expectations.json"
    ))
    .expect("risky expectations should be JSON");

    let alias_input: JobMatchInputV1 = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/alias-match.input.json"
    ))
    .expect("alias input should be typed JSON");
    let alias = match_job(&alias_input).expect("alias input should match");
    assert_eq!(
        alias.overall_score,
        expectations["alias_match"]["overall_score"]
    );
    assert_eq!(
        serde_json::to_value(&alias.category_scores).expect("scores should serialize"),
        expectations["alias_match"]["category_scores"]
    );
    let alias_skills = alias
        .category_results
        .iter()
        .find(|result| result.category == JobMatchCategoryV1::SkillsMatch)
        .expect("skills category should exist");
    let alias_required = alias_skills
        .items
        .iter()
        .filter(|item| item.kind == JobMatchItemKindV1::RequiredSkill)
        .collect::<Vec<_>>();
    assert_eq!(
        alias_required.len(),
        expectations["alias_match"]["required_alias_matches"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .expect("alias count should fit usize")
    );
    assert!(alias_required.iter().all(|item| {
        item.status == JobMatchItemStatusV1::ConfirmedMatch
            && item.match_type == Some(JobMatchTypeV1::ConservativeAlias)
    }));
    assert_eq!(
        alias.recommendation.blocking_required_skill_gaps.len(),
        expectations["alias_match"]["required_skill_gaps"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .expect("gap count should fit usize")
    );
    assert_eq!(
        serde_json::to_value(alias.recommendation.label).expect("recommendation should serialize"),
        expectations["alias_match"]["recommendation"]
    );

    let adjacent_input: JobMatchInputV1 = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/close-non-equivalent.input.json"
    ))
    .expect("adjacent input should be typed JSON");
    let adjacent = match_job(&adjacent_input).expect("adjacent input should match");
    assert_eq!(
        adjacent.overall_score,
        expectations["close_non_equivalent"]["overall_score"]
    );
    assert_eq!(
        serde_json::to_value(&adjacent.category_scores).expect("scores should serialize"),
        expectations["close_non_equivalent"]["category_scores"]
    );
    let adjacent_skills = adjacent
        .category_results
        .iter()
        .find(|result| result.category == JobMatchCategoryV1::SkillsMatch)
        .expect("skills category should exist");
    assert!(
        adjacent_skills
            .items
            .iter()
            .filter(|item| item.kind == JobMatchItemKindV1::RequiredSkill)
            .all(|item| item.status == JobMatchItemStatusV1::LikelyMissing)
    );
    assert_eq!(
        adjacent.recommendation.blocking_required_skill_gaps.len(),
        expectations["close_non_equivalent"]["required_skill_gaps"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .expect("gap count should fit usize")
    );
    assert_eq!(
        serde_json::to_value(adjacent.recommendation.label)
            .expect("recommendation should serialize"),
        expectations["close_non_equivalent"]["recommendation"]
    );

    let vague: JobMatchV1 = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/vague-job.expected.json"
    ))
    .expect("vague fixture should be typed JSON");
    assert!(vague.confidence_context.is_uncertain);
    assert!(
        vague
            .category_results
            .iter()
            .all(|category| (50..=75).contains(&category.score))
    );
    assert!(vague.top_gaps.is_empty());
    assert_eq!(
        vague.recommendation.status,
        JobMatchRecommendationStatusV1::Provisional
    );
    assert_ne!(
        vague.recommendation.label,
        JobMatchRecommendationLabelV1::ApplyNow
    );

    let weak_input: JobMatchInputV1 = serde_json::from_str(include_str!(
        "../fixtures/job/phase4b/weak-resume-narrow-job.input.json"
    ))
    .expect("weak input should be typed JSON");
    let weak = match_job(&weak_input).expect("weak input should match");
    assert_eq!(
        weak.overall_score,
        expectations["weak_resume_narrow_job"]["overall_score"]
    );
    assert_eq!(
        serde_json::to_value(&weak.category_scores).expect("scores should serialize"),
        expectations["weak_resume_narrow_job"]["category_scores"]
    );
    assert_eq!(
        weak.recommendation.blocking_required_skill_gaps.len(),
        expectations["weak_resume_narrow_job"]["required_skill_gaps"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .expect("gap count should fit usize")
    );
    assert_eq!(
        serde_json::to_value(weak.recommendation.label).expect("recommendation should serialize"),
        expectations["weak_resume_narrow_job"]["recommendation"]
    );
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
fn reviewed_resume_analysis_replacement_fixture_matches_the_typed_contract() {
    let input: ResumeAnalysisReplacementReviewInputV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase7/complete-analysis-replacement-review.input.json"
    ))
    .expect("analysis-replacement review input should be typed JSON");
    let expected: ResumeAnalysisReplacementReviewV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase7/complete-analysis-replacement-review.expected.json"
    ))
    .expect("analysis-replacement review output should be typed JSON");
    let actual = review_resume_analysis_replacements(&input)
        .expect("analysis-replacement review should succeed");

    assert_eq!(actual, expected);
    assert_eq!(
        actual.baseline_analysis,
        analyze_resume(&input.resume).expect("baseline analysis should repeat")
    );
}

#[test]
fn reviewed_resume_analysis_suggestion_fixture_matches_the_typed_contract() {
    let input: ResumeAnalysisSuggestionReviewInputV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase7/complete-analysis-suggestion-review.input.json"
    ))
    .expect("analysis-suggestion review input should be typed JSON");
    let expected: ResumeAnalysisSuggestionReviewV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase7/complete-analysis-suggestion-review.expected.json"
    ))
    .expect("analysis-suggestion review output should be typed JSON");
    let actual = review_resume_analysis_suggestions(&input)
        .expect("analysis-suggestion review should succeed");

    assert_eq!(actual, expected);
    assert_eq!(
        actual.baseline_analysis,
        analyze_resume(&input.resume).expect("baseline analysis should repeat")
    );
}

#[test]
fn reviewed_resume_variant_fixtures_match_the_typed_contract() {
    let review_input: ResumeVariantReviewInputV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase7/complete-variant-review.input.json"
    ))
    .expect("variant review input should be typed JSON");
    let expected_review: ResumeVariantReviewV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase7/complete-variant-review.expected.json"
    ))
    .expect("variant review output should be typed JSON");
    assert_eq!(
        review_resume_variant(&review_input).expect("variant should review"),
        expected_review
    );

    let materialization_input: ResumeVariantMaterializationInputV1 = serde_json::from_str(
        include_str!("../fixtures/resume/phase7/selected-variant-materialization.input.json"),
    )
    .expect("variant materialization input should be typed JSON");
    let expected_variant: ResumeVariantV1 = serde_json::from_str(include_str!(
        "../fixtures/resume/phase7/selected-variant-materialization.expected.json"
    ))
    .expect("variant output should be typed JSON");
    assert_eq!(
        materialize_resume_variant(&materialization_input).expect("variant should materialize"),
        expected_variant
    );
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
