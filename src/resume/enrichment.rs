use std::collections::BTreeSet;

use super::contract::{ResumeEvaluationErrorCodeV1, ResumeEvaluationErrorV1};
use super::enrichment_contract::{
    ENRICHMENT_INPUT_SCHEMA_VERSION, ENRICHMENT_POLICY_VERSION, ENRICHMENT_RESULT_SCHEMA_VERSION,
    MAX_ENRICHMENT_PROPOSAL_CHARACTERS, ResumeEnrichmentEducationProposalV1,
    ResumeEnrichmentExperienceProposalV1, ResumeEnrichmentFieldProvenanceV1,
    ResumeEnrichmentFieldSourceV1, ResumeEnrichmentInputV1, ResumeEnrichmentMergeStatusV1,
    ResumeEnrichmentMergeV1, ResumeEnrichmentProposalV1, ResumeEnrichmentResultV1,
    ResumeEnrichmentWarningCodeV1, ResumeEnrichmentWarningV1,
};
use super::normalization::{locate_source_span, normalize_grounding};
use super::normalization_contract::{
    ENRICHMENT_PROPOSAL_SCHEMA_VERSION, MAX_NORMALIZED_BULLET_CHARACTERS,
    MAX_NORMALIZED_BULLETS_PER_EXPERIENCE, MAX_NORMALIZED_EDUCATION_ENTRIES,
    MAX_NORMALIZED_EXPERIENCE_ENTRIES, MAX_NORMALIZED_FIELD_CHARACTERS,
    MAX_NORMALIZED_RAW_TEXT_CHARACTERS, MAX_NORMALIZED_SKILLS, MAX_NORMALIZED_SUMMARY_CHARACTERS,
    ResumeEducationEntryV1, ResumeEnrichmentRequestStatusV1, ResumeEnrichmentSectionV1,
    ResumeExperienceEntryV1, ResumeGroundedTextV1, ResumeNormalizedFieldV1,
    ResumeSourceTransformationV1,
};
use super::normalize_resume;

struct ValidatedProposal {
    summary: Option<ResumeGroundedTextV1>,
    experience: Vec<ResumeExperienceEntryV1>,
    education: Vec<ResumeEducationEntryV1>,
    skills: Vec<ResumeGroundedTextV1>,
}

pub fn apply_resume_enrichment(
    input: &ResumeEnrichmentInputV1,
) -> Result<ResumeEnrichmentResultV1, ResumeEvaluationErrorV1> {
    if input.schema_version != ENRICHMENT_INPUT_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedEnrichmentInputSchemaVersion,
            format!("schema_version must be {ENRICHMENT_INPUT_SCHEMA_VERSION}."),
            "schema_version",
        ));
    }
    if input.proposal.schema_version != ENRICHMENT_PROPOSAL_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedEnrichmentProposalSchemaVersion,
            format!("proposal.schema_version must be {ENRICHMENT_PROPOSAL_SCHEMA_VERSION}."),
            "proposal.schema_version",
        ));
    }

    let baseline = normalize_resume(&input.resume)?;
    if baseline.enrichment_request.status != ResumeEnrichmentRequestStatusV1::Eligible {
        return Err(error(
            ResumeEvaluationErrorCodeV1::EnrichmentNotEligible,
            "The deterministic normalization is not eligible for external enrichment.",
            "proposal",
        ));
    }

    validate_total_size(&input.proposal)?;
    enforce_target_sections(
        &input.proposal,
        &baseline.enrichment_request.target_sections,
    )?;
    let validated = validate_proposal(&input.resume.text, &input.proposal)?;
    let mut assisted_document = baseline.deterministic_document.clone();
    let mut applied_fields = Vec::new();

    if let (true, Some(summary)) = (
        baseline
            .enrichment_request
            .target_sections
            .contains(&ResumeEnrichmentSectionV1::Summary),
        validated.summary,
    ) {
        assisted_document.summary = Some(summary);
        applied_fields.push(ResumeNormalizedFieldV1::Summary);
    }
    if baseline
        .enrichment_request
        .target_sections
        .contains(&ResumeEnrichmentSectionV1::Experience)
        && !validated.experience.is_empty()
    {
        assisted_document.experience = validated.experience;
        applied_fields.push(ResumeNormalizedFieldV1::Experience);
    }
    if baseline
        .enrichment_request
        .target_sections
        .contains(&ResumeEnrichmentSectionV1::Education)
        && !validated.education.is_empty()
    {
        assisted_document.education = validated.education;
        applied_fields.push(ResumeNormalizedFieldV1::Education);
    }
    if baseline
        .enrichment_request
        .target_sections
        .contains(&ResumeEnrichmentSectionV1::Skills)
        && !validated.skills.is_empty()
    {
        assisted_document.skills = validated.skills;
        applied_fields.push(ResumeNormalizedFieldV1::Skills);
    }

    let field_provenance = ResumeEnrichmentSectionV1::ALL
        .into_iter()
        .map(|section| {
            let field = section.field();
            let source = if applied_fields.contains(&field) {
                ResumeEnrichmentFieldSourceV1::ExternalSourceGroundedProposal
            } else if deterministic_field_has_value(&baseline, field) {
                ResumeEnrichmentFieldSourceV1::Deterministic
            } else {
                ResumeEnrichmentFieldSourceV1::NotAvailable
            };
            ResumeEnrichmentFieldProvenanceV1 { field, source }
        })
        .collect();

    Ok(ResumeEnrichmentResultV1 {
        schema_version: ENRICHMENT_RESULT_SCHEMA_VERSION.to_owned(),
        policy_version: ENRICHMENT_POLICY_VERSION.to_owned(),
        core_version: env!("CARGO_PKG_VERSION").to_owned(),
        merge: ResumeEnrichmentMergeV1 {
            status: if applied_fields.is_empty() {
                ResumeEnrichmentMergeStatusV1::NotApplied
            } else {
                ResumeEnrichmentMergeStatusV1::Applied
            },
            target_sections: baseline.enrichment_request.target_sections.clone(),
            applied_fields,
            field_provenance,
        },
        assisted_document,
        baseline,
        warnings: vec![
            ResumeEnrichmentWarningV1 {
                code: ResumeEnrichmentWarningCodeV1::AssistedFieldsNonAuthoritative,
                message: "Externally proposed fields are source-grounded assistance and must not replace the deterministic baseline used for authoritative scoring."
                    .to_owned(),
            },
            ResumeEnrichmentWarningV1 {
                code: ResumeEnrichmentWarningCodeV1::DeterministicConfidencePreserved,
                message: "Parser confidence and field-status evidence remain those of the deterministic normalization."
                    .to_owned(),
            },
        ],
    })
}

fn validate_total_size(
    proposal: &ResumeEnrichmentProposalV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    let character_count = proposal_character_count(proposal);
    if character_count > MAX_ENRICHMENT_PROPOSAL_CHARACTERS {
        return Err(error(
            ResumeEvaluationErrorCodeV1::EnrichmentProposalTooLarge,
            format!(
                "The enrichment proposal must contain at most {MAX_ENRICHMENT_PROPOSAL_CHARACTERS} string characters; received {character_count}."
            ),
            "proposal",
        ));
    }
    Ok(())
}

fn proposal_character_count(proposal: &ResumeEnrichmentProposalV1) -> usize {
    let mut total = character_count(0, &proposal.schema_version);
    total = character_count(total, &proposal.summary);
    for entry in &proposal.experience {
        total = character_count(total, &entry.raw_text);
        total = character_count(total, &entry.job_title);
        total = character_count(total, &entry.company);
        total = character_count(total, &entry.date_range);
        for bullet in &entry.bullets {
            total = character_count(total, bullet);
        }
    }
    for entry in &proposal.education {
        total = character_count(total, &entry.raw_text);
        total = character_count(total, &entry.institution);
        total = character_count(total, &entry.degree);
        total = character_count(total, &entry.date_range);
    }
    for skill in &proposal.skills {
        total = character_count(total, skill);
    }
    total
}

fn character_count(total: usize, value: &str) -> usize {
    total.saturating_add(value.chars().count())
}

fn enforce_target_sections(
    proposal: &ResumeEnrichmentProposalV1,
    targets: &[ResumeEnrichmentSectionV1],
) -> Result<(), ResumeEvaluationErrorV1> {
    for (section, populated, path) in [
        (
            ResumeEnrichmentSectionV1::Summary,
            !proposal.summary.trim().is_empty(),
            "proposal.summary",
        ),
        (
            ResumeEnrichmentSectionV1::Experience,
            !proposal.experience.is_empty(),
            "proposal.experience",
        ),
        (
            ResumeEnrichmentSectionV1::Education,
            !proposal.education.is_empty(),
            "proposal.education",
        ),
        (
            ResumeEnrichmentSectionV1::Skills,
            !proposal.skills.is_empty(),
            "proposal.skills",
        ),
    ] {
        if populated && !targets.contains(&section) {
            return Err(error(
                ResumeEvaluationErrorCodeV1::EnrichmentNonTargetPopulated,
                format!(
                    "{path} must remain empty because it is not an eligible enrichment target."
                ),
                path,
            ));
        }
    }
    Ok(())
}

fn validate_proposal(
    source_text: &str,
    proposal: &ResumeEnrichmentProposalV1,
) -> Result<ValidatedProposal, ResumeEvaluationErrorV1> {
    if proposal.experience.len() > MAX_NORMALIZED_EXPERIENCE_ENTRIES {
        return Err(list_too_long(
            "proposal.experience",
            MAX_NORMALIZED_EXPERIENCE_ENTRIES,
            proposal.experience.len(),
        ));
    }
    if proposal.education.len() > MAX_NORMALIZED_EDUCATION_ENTRIES {
        return Err(list_too_long(
            "proposal.education",
            MAX_NORMALIZED_EDUCATION_ENTRIES,
            proposal.education.len(),
        ));
    }
    if proposal.skills.len() > MAX_NORMALIZED_SKILLS {
        return Err(list_too_long(
            "proposal.skills",
            MAX_NORMALIZED_SKILLS,
            proposal.skills.len(),
        ));
    }

    let summary = validate_grounded(
        &proposal.summary,
        "proposal.summary",
        source_text,
        MAX_NORMALIZED_SUMMARY_CHARACTERS,
        true,
    )?;
    let experience = proposal
        .experience
        .iter()
        .enumerate()
        .map(|(index, entry)| validate_experience_entry(entry, index, source_text))
        .collect::<Result<Vec<_>, _>>()?;
    let education = proposal
        .education
        .iter()
        .enumerate()
        .map(|(index, entry)| validate_education_entry(entry, index, source_text))
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen = BTreeSet::new();
    let mut skills = Vec::new();
    for (index, skill) in proposal.skills.iter().enumerate() {
        let path = format!("proposal.skills[{index}]");
        let Some(value) = validate_grounded(
            skill,
            &path,
            source_text,
            MAX_NORMALIZED_FIELD_CHARACTERS,
            false,
        )?
        else {
            return Err(field_empty(path));
        };
        if seen.insert(value.value.to_lowercase()) {
            skills.push(value);
        }
    }

    Ok(ValidatedProposal {
        summary,
        experience,
        education,
        skills,
    })
}

fn validate_experience_entry(
    entry: &ResumeEnrichmentExperienceProposalV1,
    index: usize,
    source_text: &str,
) -> Result<ResumeExperienceEntryV1, ResumeEvaluationErrorV1> {
    let prefix = format!("proposal.experience[{index}]");
    let raw_path = format!("{prefix}.raw_text");
    let Some(raw_text) = validate_grounded(
        &entry.raw_text,
        &raw_path,
        source_text,
        MAX_NORMALIZED_RAW_TEXT_CHARACTERS,
        false,
    )?
    else {
        return Err(field_empty(raw_path));
    };
    let title_path = format!("{prefix}.job_title");
    let Some(job_title) = validate_grounded(
        &entry.job_title,
        &title_path,
        &entry.raw_text,
        MAX_NORMALIZED_FIELD_CHARACTERS,
        false,
    )?
    else {
        return Err(field_empty(title_path));
    };
    let company = validate_nested_grounded(
        &entry.company,
        &format!("{prefix}.company"),
        &entry.raw_text,
        source_text,
        MAX_NORMALIZED_FIELD_CHARACTERS,
        true,
    )?;
    let date_range = validate_nested_grounded(
        &entry.date_range,
        &format!("{prefix}.date_range"),
        &entry.raw_text,
        source_text,
        MAX_NORMALIZED_FIELD_CHARACTERS,
        true,
    )?;
    if company.is_none() && date_range.is_none() {
        return Err(error(
            ResumeEvaluationErrorCodeV1::EnrichmentEntryInvalid,
            "An enrichment experience entry must contain a source-grounded company or date range.",
            prefix,
        ));
    }
    if entry.bullets.len() > MAX_NORMALIZED_BULLETS_PER_EXPERIENCE {
        return Err(list_too_long(
            &format!("{prefix}.bullets"),
            MAX_NORMALIZED_BULLETS_PER_EXPERIENCE,
            entry.bullets.len(),
        ));
    }
    let bullets = entry
        .bullets
        .iter()
        .enumerate()
        .map(|(bullet_index, bullet)| {
            let path = format!("{prefix}.bullets[{bullet_index}]");
            validate_nested_grounded(
                bullet,
                &path,
                &entry.raw_text,
                source_text,
                MAX_NORMALIZED_BULLET_CHARACTERS,
                false,
            )?
            .ok_or_else(|| field_empty(path))
        })
        .collect::<Result<Vec<_>, ResumeEvaluationErrorV1>>()?;

    Ok(ResumeExperienceEntryV1 {
        raw_text,
        job_title: Some(remap_span(job_title, source_text)),
        company,
        date_range,
        bullets,
    })
}

fn validate_education_entry(
    entry: &ResumeEnrichmentEducationProposalV1,
    index: usize,
    source_text: &str,
) -> Result<ResumeEducationEntryV1, ResumeEvaluationErrorV1> {
    let prefix = format!("proposal.education[{index}]");
    let raw_path = format!("{prefix}.raw_text");
    let Some(raw_text) = validate_grounded(
        &entry.raw_text,
        &raw_path,
        source_text,
        MAX_NORMALIZED_RAW_TEXT_CHARACTERS,
        false,
    )?
    else {
        return Err(field_empty(raw_path));
    };
    let institution = validate_nested_grounded(
        &entry.institution,
        &format!("{prefix}.institution"),
        &entry.raw_text,
        source_text,
        MAX_NORMALIZED_FIELD_CHARACTERS,
        true,
    )?;
    let degree = validate_nested_grounded(
        &entry.degree,
        &format!("{prefix}.degree"),
        &entry.raw_text,
        source_text,
        MAX_NORMALIZED_FIELD_CHARACTERS,
        true,
    )?;
    if institution.is_none() && degree.is_none() {
        return Err(error(
            ResumeEvaluationErrorCodeV1::EnrichmentEntryInvalid,
            "An enrichment education entry must contain a source-grounded institution or degree.",
            prefix,
        ));
    }
    let date_range = validate_nested_grounded(
        &entry.date_range,
        &format!("{prefix}.date_range"),
        &entry.raw_text,
        source_text,
        MAX_NORMALIZED_FIELD_CHARACTERS,
        true,
    )?;

    Ok(ResumeEducationEntryV1 {
        raw_text,
        institution,
        degree,
        date_range,
    })
}

fn validate_nested_grounded(
    value: &str,
    path: &str,
    raw_text: &str,
    source_text: &str,
    maximum: usize,
    allow_empty: bool,
) -> Result<Option<ResumeGroundedTextV1>, ResumeEvaluationErrorV1> {
    let nested = validate_grounded(value, path, raw_text, maximum, allow_empty)?;
    Ok(nested.map(|value| remap_span(value, source_text)))
}

fn remap_span(mut value: ResumeGroundedTextV1, source_text: &str) -> ResumeGroundedTextV1 {
    if let Some(span) = locate_source_span(
        source_text,
        &value.value,
        ResumeSourceTransformationV1::ExternalSourceGroundedProposal,
    ) {
        value.source = span;
    }
    value
}

fn validate_grounded(
    value: &str,
    path: &str,
    source_text: &str,
    maximum: usize,
    allow_empty: bool,
) -> Result<Option<ResumeGroundedTextV1>, ResumeEvaluationErrorV1> {
    let cleaned = value.trim();
    if cleaned.is_empty() {
        return if allow_empty {
            Ok(None)
        } else {
            Err(field_empty(path))
        };
    }
    let character_count = cleaned.chars().count();
    if character_count > maximum {
        return Err(error(
            ResumeEvaluationErrorCodeV1::EnrichmentFieldTooLong,
            format!(
                "{path} must contain at most {maximum} characters; received {character_count}."
            ),
            path,
        ));
    }
    if !normalize_grounding(source_text).contains(&normalize_grounding(cleaned)) {
        return Err(error(
            ResumeEvaluationErrorCodeV1::EnrichmentValueNotGrounded,
            format!("{path} must be copied from the supplied resume text."),
            path,
        ));
    }
    let Some(source) = locate_source_span(
        source_text,
        cleaned,
        ResumeSourceTransformationV1::ExternalSourceGroundedProposal,
    ) else {
        return Err(error(
            ResumeEvaluationErrorCodeV1::EnrichmentValueNotGrounded,
            format!("{path} must be traceable to the supplied resume text."),
            path,
        ));
    };
    Ok(Some(ResumeGroundedTextV1 {
        value: cleaned.to_owned(),
        source,
    }))
}

fn deterministic_field_has_value(
    baseline: &super::normalization_contract::ResumeNormalizationV1,
    field: ResumeNormalizedFieldV1,
) -> bool {
    match field {
        ResumeNormalizedFieldV1::Summary => baseline.deterministic_document.summary.is_some(),
        ResumeNormalizedFieldV1::Experience => {
            !baseline.deterministic_document.experience.is_empty()
        }
        ResumeNormalizedFieldV1::Education => !baseline.deterministic_document.education.is_empty(),
        ResumeNormalizedFieldV1::Skills => !baseline.deterministic_document.skills.is_empty(),
        ResumeNormalizedFieldV1::Contact => {
            let contact = &baseline.deterministic_document.contact;
            contact.name.is_some()
                || contact.email.is_some()
                || contact.phone.is_some()
                || !contact.links.is_empty()
        }
        ResumeNormalizedFieldV1::Projects => !baseline.deterministic_document.projects.is_empty(),
        ResumeNormalizedFieldV1::Certifications => {
            !baseline.deterministic_document.certifications.is_empty()
        }
    }
}

fn field_empty(path: impl Into<String>) -> ResumeEvaluationErrorV1 {
    let path = path.into();
    error(
        ResumeEvaluationErrorCodeV1::EnrichmentFieldEmpty,
        format!("{path} must contain a non-empty source-grounded string."),
        path,
    )
}

fn list_too_long(path: &str, maximum: usize, actual: usize) -> ResumeEvaluationErrorV1 {
    error(
        ResumeEvaluationErrorCodeV1::EnrichmentListTooLong,
        format!("{path} must contain at most {maximum} items; received {actual}."),
        path,
    )
}

fn error(
    code: ResumeEvaluationErrorCodeV1,
    message: impl Into<String>,
    field_path: impl Into<String>,
) -> ResumeEvaluationErrorV1 {
    ResumeEvaluationErrorV1::new(code, message, field_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ENRICHMENT_PROPOSAL_SCHEMA_VERSION, INPUT_SCHEMA_VERSION, ResumeInputMetadataV1};

    const SOURCE: &str = "Morgan Lee\nmorgan.lee@example.com\nBackend engineer focused on reliable APIs and platform stability.\nPlatform Engineer at Acme Corp since 2021\nDelivered internal APIs used by support and operations teams.\nReduced deployment errors by 30 percent through release automation.\nComfortable with Python and Django for backend services\nBachelor of Science in Computer Science\nExample University\n2016 - 2020";

    fn valid_input() -> ResumeEnrichmentInputV1 {
        ResumeEnrichmentInputV1 {
            schema_version: ENRICHMENT_INPUT_SCHEMA_VERSION.to_owned(),
            resume: crate::ResumeInputV1 {
                schema_version: INPUT_SCHEMA_VERSION.to_owned(),
                text: SOURCE.to_owned(),
                metadata: ResumeInputMetadataV1::default(),
            },
            proposal: ResumeEnrichmentProposalV1 {
                schema_version: ENRICHMENT_PROPOSAL_SCHEMA_VERSION.to_owned(),
                summary: "Backend engineer focused on reliable APIs and platform stability."
                    .to_owned(),
                experience: vec![ResumeEnrichmentExperienceProposalV1 {
                    raw_text: "Platform Engineer at Acme Corp since 2021\nDelivered internal APIs used by support and operations teams.\nReduced deployment errors by 30 percent through release automation."
                        .to_owned(),
                    job_title: "Platform Engineer".to_owned(),
                    company: "Acme Corp".to_owned(),
                    date_range: String::new(),
                    bullets: vec![
                        "Delivered internal APIs used by support and operations teams.".to_owned(),
                        "Reduced deployment errors by 30 percent through release automation."
                            .to_owned(),
                    ],
                }],
                education: Vec::new(),
                skills: vec!["Python".to_owned(), "Django".to_owned()],
            },
        }
    }

    #[test]
    fn grounded_proposal_fills_only_eligible_fields_and_preserves_baseline() {
        let input = valid_input();
        let result = apply_resume_enrichment(&input).expect("proposal should be accepted");

        assert_eq!(result.merge.status, ResumeEnrichmentMergeStatusV1::Applied);
        assert_eq!(
            result.merge.applied_fields,
            vec![
                ResumeNormalizedFieldV1::Summary,
                ResumeNormalizedFieldV1::Experience,
                ResumeNormalizedFieldV1::Skills,
            ]
        );
        assert!(result.baseline.deterministic_document.summary.is_none());
        assert!(result.baseline.deterministic_document.experience.is_empty());
        assert!(result.baseline.deterministic_document.skills.is_empty());
        assert_eq!(
            result
                .assisted_document
                .summary
                .as_ref()
                .map(|summary| summary.value.as_str()),
            Some("Backend engineer focused on reliable APIs and platform stability.")
        );
        assert_eq!(result.assisted_document.experience.len(), 1);
        assert_eq!(result.assisted_document.skills.len(), 2);
        assert!(!result.baseline.deterministic_document.education.is_empty());
        assert_eq!(
            result.assisted_document.education,
            result.baseline.deterministic_document.education
        );
        assert_eq!(
            result.baseline.confidence.label,
            crate::ResumeParseConfidenceLabelV1::Low
        );
    }

    #[test]
    fn unsupported_or_non_target_values_are_rejected() {
        let mut unsupported = valid_input();
        unsupported.proposal.skills = vec!["Kubernetes".to_owned()];
        let unsupported_error =
            apply_resume_enrichment(&unsupported).expect_err("unsupported skill should fail");
        assert_eq!(
            unsupported_error.code,
            ResumeEvaluationErrorCodeV1::EnrichmentValueNotGrounded
        );
        assert!(!unsupported_error.message.contains("Kubernetes"));

        let mut non_target = valid_input();
        non_target.proposal.education = vec![ResumeEnrichmentEducationProposalV1 {
            raw_text: "Bachelor of Science in Computer Science\nExample University\n2016 - 2020"
                .to_owned(),
            institution: "Example University".to_owned(),
            degree: "Bachelor of Science in Computer Science".to_owned(),
            date_range: "2016 - 2020".to_owned(),
        }];
        let non_target_error =
            apply_resume_enrichment(&non_target).expect_err("non-target should fail");
        assert_eq!(
            non_target_error.code,
            ResumeEvaluationErrorCodeV1::EnrichmentNonTargetPopulated
        );
    }

    #[test]
    fn deterministic_output_is_unchanged_by_different_proposals() {
        let first = apply_resume_enrichment(&valid_input()).expect("proposal should apply");
        let mut second_input = valid_input();
        second_input.proposal.summary.clear();
        second_input.proposal.skills.clear();
        let second =
            apply_resume_enrichment(&second_input).expect("empty target values are allowed");

        assert_eq!(first.baseline, second.baseline);
        assert_ne!(first.assisted_document, second.assisted_document);
        assert_eq!(second.merge.status, ResumeEnrichmentMergeStatusV1::Applied);
    }

    #[test]
    fn proposal_limits_and_entry_requirements_are_enforced_without_payload_echo() {
        let mut oversized = valid_input();
        oversized.proposal.summary = "private".repeat(8_000);
        let oversized_error =
            apply_resume_enrichment(&oversized).expect_err("oversized proposal should fail");
        assert_eq!(
            oversized_error.code,
            ResumeEvaluationErrorCodeV1::EnrichmentProposalTooLarge
        );
        assert!(!oversized_error.message.contains("private"));

        let mut incomplete = valid_input();
        incomplete.proposal.experience[0].company.clear();
        incomplete.proposal.experience[0].date_range.clear();
        let incomplete_error =
            apply_resume_enrichment(&incomplete).expect_err("incomplete entry should fail");
        assert_eq!(
            incomplete_error.code,
            ResumeEvaluationErrorCodeV1::EnrichmentEntryInvalid
        );
    }

    #[test]
    fn duplicate_grounded_skills_are_deduplicated_stably() {
        let mut input = valid_input();
        input.proposal.skills.push("python".to_owned());
        let result = apply_resume_enrichment(&input).expect("proposal should apply");
        let skills = result
            .assisted_document
            .skills
            .iter()
            .map(|skill| skill.value.as_str())
            .collect::<Vec<_>>();

        assert_eq!(skills, vec!["Python", "Django"]);
    }

    #[test]
    fn high_confidence_normalization_is_not_eligible() {
        let mut input = valid_input();
        input.resume.text = "Morgan Lee\nmorgan@example.com\n+1 555 010 0200\nSUMMARY\nBackend engineer building reliable APIs.\nEXPERIENCE\nPlatform Engineer at Acme Corp\n2021 - Present\n- Built reliable APIs for customers.\nEDUCATION\nBSc Computer Science, Example University\n2017 - 2021\nSKILLS\nRust, SQL, Docker, Kubernetes, AWS"
            .to_owned();
        input.proposal.summary.clear();
        input.proposal.experience.clear();
        input.proposal.skills.clear();

        let error = apply_resume_enrichment(&input).expect_err("high confidence should not enrich");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::EnrichmentNotEligible
        );
    }
}
