use std::collections::BTreeSet;
use std::ops::Range;

use crate::normalize_job;

use super::contract::{ResumeEvaluationErrorCodeV1, ResumeEvaluationErrorV1, ResumeInputV1};
use super::validation::validate_resume_input;
use super::variant_contract::{
    MAX_RESUME_VARIANT_CHANGE_TEXT_CHARACTERS, MAX_RESUME_VARIANT_CHANGES,
    MAX_RESUME_VARIANT_EVIDENCE_CHARACTERS, MAX_RESUME_VARIANT_EVIDENCE_ITEMS,
    MAX_RESUME_VARIANT_PROPOSAL_CHARACTERS, RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION,
    RESUME_VARIANT_POLICY_VERSION, RESUME_VARIANT_PROPOSAL_SCHEMA_VERSION,
    RESUME_VARIANT_REVIEW_INPUT_SCHEMA_VERSION, RESUME_VARIANT_REVIEW_SCHEMA_VERSION,
    RESUME_VARIANT_SCHEMA_VERSION, ResumeVariantAuthorityV1, ResumeVariantCanonicalChangeV1,
    ResumeVariantDiscardCodeV1, ResumeVariantDiscardedChangeV1,
    ResumeVariantMaterializationInputV1, ResumeVariantProposalV1, ResumeVariantProposedChangeV1,
    ResumeVariantReviewInputV1, ResumeVariantReviewV1, ResumeVariantV1, ResumeVariantWarningCodeV1,
    ResumeVariantWarningV1,
};

#[derive(Clone)]
struct ValidatedChange {
    input_index: usize,
    byte_range: Range<usize>,
    change: ResumeVariantProposedChangeV1,
}

pub fn review_resume_variant(
    input: &ResumeVariantReviewInputV1,
) -> Result<ResumeVariantReviewV1, ResumeEvaluationErrorV1> {
    validate_review_envelope(input)?;
    validate_resume_input(&input.resume).map_err(|error| prefix_error_path(error, "resume"))?;
    normalize_job(&input.vacancy).map_err(|error| prefix_error_path(error, "vacancy"))?;
    validate_proposal_bounds(&input.proposal)?;

    let line_ranges = source_line_ranges(&input.resume.text);
    let mut discarded_changes = Vec::new();
    let mut candidates = Vec::new();
    for (input_index, change) in input.proposal.changes.iter().enumerate() {
        match validate_change(
            input_index,
            change,
            &input.resume.text,
            &input.vacancy.text,
            &line_ranges,
        ) {
            Ok(validated) => candidates.push(validated),
            Err(code) => {
                discarded_changes.push(ResumeVariantDiscardedChangeV1 { input_index, code })
            }
        }
    }

    let overlapping = overlapping_candidate_indices(&candidates);
    let mut retained = Vec::new();
    for (candidate_index, candidate) in candidates.into_iter().enumerate() {
        if overlapping.contains(&candidate_index) {
            discarded_changes.push(ResumeVariantDiscardedChangeV1 {
                input_index: candidate.input_index,
                code: ResumeVariantDiscardCodeV1::DuplicateOrOverlappingTarget,
            });
        } else {
            retained.push(candidate);
        }
    }
    discarded_changes.sort_by_key(|discarded| discarded.input_index);
    retained.sort_by(|left, right| {
        left.byte_range
            .start
            .cmp(&right.byte_range.start)
            .then(left.byte_range.end.cmp(&right.byte_range.end))
            .then(left.input_index.cmp(&right.input_index))
    });

    let canonical = retained
        .iter()
        .enumerate()
        .map(|(index, retained)| canonical_change(index, &retained.change))
        .collect::<Vec<_>>();
    let preview = apply_changes(&input.resume.text, &retained);
    validate_candidate(&preview, &input.resume)?;

    Ok(ResumeVariantReviewV1 {
        schema_version: RESUME_VARIANT_REVIEW_SCHEMA_VERSION.to_owned(),
        policy_version: RESUME_VARIANT_POLICY_VERSION.to_owned(),
        core_version: env!("CARGO_PKG_VERSION").to_owned(),
        authority: ResumeVariantAuthorityV1::AssistedNonAuthoritative,
        baseline_resume: input.resume.clone(),
        proposed_preview_text: preview,
        changes: canonical,
        discarded_changes,
        warnings: variant_warnings(),
    })
}

pub fn materialize_resume_variant(
    input: &ResumeVariantMaterializationInputV1,
) -> Result<ResumeVariantV1, ResumeEvaluationErrorV1> {
    if input.schema_version != RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedVariantMaterializationInputSchemaVersion,
            format!(
                "schema_version must be {RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION}."
            ),
            "schema_version",
        ));
    }
    if input.expected_review_policy_version != RESUME_VARIANT_POLICY_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedVariantPolicyVersion,
            format!("expected_review_policy_version must be {RESUME_VARIANT_POLICY_VERSION}."),
            "expected_review_policy_version",
        ));
    }
    if input.selected_change_ids.is_empty() {
        return Err(error(
            ResumeEvaluationErrorCodeV1::VariantSelectionEmpty,
            "selected_change_ids must contain at least one canonical change identifier.",
            "selected_change_ids",
        ));
    }
    if input.selected_change_ids.len() > MAX_RESUME_VARIANT_CHANGES {
        return Err(error(
            ResumeEvaluationErrorCodeV1::VariantSelectionTooLong,
            format!("selected_change_ids must contain at most {MAX_RESUME_VARIANT_CHANGES} items."),
            "selected_change_ids",
        ));
    }

    let review = review_resume_variant(&input.review_input)
        .map_err(|error| prefix_error_path(error, "review_input"))?;
    let selected_ids = input.selected_change_ids.iter().collect::<BTreeSet<_>>();
    if selected_ids.len() != input.selected_change_ids.len()
        || selected_ids.iter().any(|identifier| {
            !review
                .changes
                .iter()
                .any(|change| &change.change_id == *identifier)
        })
    {
        return Err(error(
            ResumeEvaluationErrorCodeV1::VariantSelectionInvalid,
            "selected_change_ids must contain unique identifiers from the canonical review.",
            "selected_change_ids",
        ));
    }

    let selected_changes = review
        .changes
        .iter()
        .filter(|change| selected_ids.contains(&change.change_id))
        .cloned()
        .collect::<Vec<_>>();
    let line_ranges = source_line_ranges(&input.review_input.resume.text);
    let retained = selected_changes
        .iter()
        .enumerate()
        .map(|(index, change)| {
            let byte_range = byte_range_for_lines(change.start_line, change.end_line, &line_ranges)
                .ok_or_else(|| {
                    error(
                        ResumeEvaluationErrorCodeV1::VariantSelectionInvalid,
                        "A selected canonical change no longer matches the reviewed baseline.",
                        "selected_change_ids",
                    )
                })?;
            Ok(ValidatedChange {
                input_index: index,
                byte_range,
                change: ResumeVariantProposedChangeV1 {
                    section: change.section,
                    start_line: change.start_line,
                    end_line: change.end_line,
                    original_text: change.original_text.clone(),
                    proposed_text: change.proposed_text.clone(),
                    resume_evidence: change.resume_evidence.clone(),
                    vacancy_evidence: change.vacancy_evidence.clone(),
                },
            })
        })
        .collect::<Result<Vec<_>, ResumeEvaluationErrorV1>>()?;
    let assisted_resume_text = apply_changes(&input.review_input.resume.text, &retained);
    validate_candidate(&assisted_resume_text, &input.review_input.resume)?;

    Ok(ResumeVariantV1 {
        schema_version: RESUME_VARIANT_SCHEMA_VERSION.to_owned(),
        policy_version: RESUME_VARIANT_POLICY_VERSION.to_owned(),
        core_version: env!("CARGO_PKG_VERSION").to_owned(),
        authority: ResumeVariantAuthorityV1::AssistedNonAuthoritative,
        baseline_resume: input.review_input.resume.clone(),
        assisted_resume_text,
        selected_changes,
        warnings: variant_warnings(),
    })
}

fn validate_review_envelope(
    input: &ResumeVariantReviewInputV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    if input.schema_version != RESUME_VARIANT_REVIEW_INPUT_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedVariantReviewInputSchemaVersion,
            format!("schema_version must be {RESUME_VARIANT_REVIEW_INPUT_SCHEMA_VERSION}."),
            "schema_version",
        ));
    }
    if input.proposal.schema_version != RESUME_VARIANT_PROPOSAL_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedVariantProposalSchemaVersion,
            format!("proposal.schema_version must be {RESUME_VARIANT_PROPOSAL_SCHEMA_VERSION}."),
            "proposal.schema_version",
        ));
    }
    Ok(())
}

fn validate_proposal_bounds(
    proposal: &ResumeVariantProposalV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    if proposal.changes.len() > MAX_RESUME_VARIANT_CHANGES {
        return Err(error(
            ResumeEvaluationErrorCodeV1::VariantChangeListTooLong,
            format!("proposal.changes must contain at most {MAX_RESUME_VARIANT_CHANGES} items."),
            "proposal.changes",
        ));
    }
    let character_count = proposal_character_count(proposal);
    if character_count > MAX_RESUME_VARIANT_PROPOSAL_CHARACTERS {
        return Err(error(
            ResumeEvaluationErrorCodeV1::VariantProposalTooLarge,
            format!(
                "The variant proposal must contain at most {MAX_RESUME_VARIANT_PROPOSAL_CHARACTERS} string characters; received {character_count}."
            ),
            "proposal",
        ));
    }
    Ok(())
}

fn proposal_character_count(proposal: &ResumeVariantProposalV1) -> usize {
    let mut total = proposal.schema_version.chars().count();
    for change in &proposal.changes {
        total = total.saturating_add(change.original_text.chars().count());
        total = total.saturating_add(change.proposed_text.chars().count());
        for evidence in change
            .resume_evidence
            .iter()
            .chain(change.vacancy_evidence.iter())
        {
            total = total.saturating_add(evidence.chars().count());
        }
    }
    total
}

fn validate_change(
    input_index: usize,
    change: &ResumeVariantProposedChangeV1,
    resume_text: &str,
    vacancy_text: &str,
    line_ranges: &[Range<usize>],
) -> Result<ValidatedChange, ResumeVariantDiscardCodeV1> {
    let Some(byte_range) = byte_range_for_lines(change.start_line, change.end_line, line_ranges)
    else {
        return Err(ResumeVariantDiscardCodeV1::InvalidLineRange);
    };
    if change.original_text.is_empty()
        || resume_text.get(byte_range.clone()) != Some(change.original_text.as_str())
    {
        return Err(ResumeVariantDiscardCodeV1::TargetMismatch);
    }
    if change.original_text == change.proposed_text {
        return Err(ResumeVariantDiscardCodeV1::NoChange);
    }
    if change.original_text.chars().count() > MAX_RESUME_VARIANT_CHANGE_TEXT_CHARACTERS
        || change.proposed_text.chars().count() > MAX_RESUME_VARIANT_CHANGE_TEXT_CHARACTERS
    {
        return Err(ResumeVariantDiscardCodeV1::ChangeTextTooLong);
    }
    if contains_unsupported_control(&change.proposed_text) {
        return Err(ResumeVariantDiscardCodeV1::UnsupportedControlCharacter);
    }
    if !valid_evidence(&change.resume_evidence, resume_text) {
        return Err(ResumeVariantDiscardCodeV1::InvalidResumeEvidence);
    }
    if !valid_evidence(&change.vacancy_evidence, vacancy_text) {
        return Err(ResumeVariantDiscardCodeV1::InvalidVacancyEvidence);
    }

    Ok(ValidatedChange {
        input_index,
        byte_range,
        change: change.clone(),
    })
}

fn valid_evidence(values: &[String], source: &str) -> bool {
    if !(1..=MAX_RESUME_VARIANT_EVIDENCE_ITEMS).contains(&values.len()) {
        return false;
    }
    let mut seen = BTreeSet::new();
    values.iter().all(|value| {
        !value.trim().is_empty()
            && value.chars().count() <= MAX_RESUME_VARIANT_EVIDENCE_CHARACTERS
            && source.contains(value)
            && seen.insert(value)
    })
}

fn contains_unsupported_control(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
}

fn overlapping_candidate_indices(candidates: &[ValidatedChange]) -> BTreeSet<usize> {
    let mut overlapping = BTreeSet::new();
    for left in 0..candidates.len() {
        for right in (left + 1)..candidates.len() {
            if ranges_overlap(&candidates[left].byte_range, &candidates[right].byte_range) {
                overlapping.insert(left);
                overlapping.insert(right);
            }
        }
    }
    overlapping
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

fn canonical_change(
    index: usize,
    change: &ResumeVariantProposedChangeV1,
) -> ResumeVariantCanonicalChangeV1 {
    ResumeVariantCanonicalChangeV1 {
        change_id: format!("change-{:04}", index + 1),
        section: change.section,
        start_line: change.start_line,
        end_line: change.end_line,
        original_text: change.original_text.clone(),
        proposed_text: change.proposed_text.clone(),
        resume_evidence: change.resume_evidence.clone(),
        vacancy_evidence: change.vacancy_evidence.clone(),
    }
}

fn apply_changes(source: &str, changes: &[ValidatedChange]) -> String {
    let mut output = source.to_owned();
    let mut ordered = changes.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|change| std::cmp::Reverse(change.byte_range.start));
    for change in ordered {
        output.replace_range(change.byte_range.clone(), &change.change.proposed_text);
    }
    output
}

fn validate_candidate(
    candidate: &str,
    baseline: &ResumeInputV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    let candidate_input = ResumeInputV1 {
        schema_version: baseline.schema_version.clone(),
        text: candidate.to_owned(),
        metadata: baseline.metadata.clone(),
    };
    validate_resume_input(&candidate_input).map_err(|_| {
        error(
            ResumeEvaluationErrorCodeV1::VariantPreviewInvalid,
            "The retained variant changes do not produce a valid bounded resume candidate.",
            "proposal.changes",
        )
    })?;
    Ok(())
}

fn source_line_ranges(source: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for segment in source.split_inclusive('\n') {
        let segment_end = start + segment.len();
        let mut content_end = segment_end;
        if segment.ends_with('\n') {
            content_end = content_end.saturating_sub(1);
            if source.as_bytes().get(content_end.saturating_sub(1)) == Some(&b'\r') {
                content_end = content_end.saturating_sub(1);
            }
        }
        ranges.push(start..content_end);
        start = segment_end;
    }
    if source.is_empty() {
        Vec::new()
    } else {
        ranges
    }
}

fn byte_range_for_lines(
    start_line: usize,
    end_line: usize,
    line_ranges: &[Range<usize>],
) -> Option<Range<usize>> {
    if start_line == 0 || end_line < start_line {
        return None;
    }
    let start = line_ranges.get(start_line.checked_sub(1)?)?.start;
    let end = line_ranges.get(end_line.checked_sub(1)?)?.end;
    Some(start..end)
}

fn variant_warnings() -> Vec<ResumeVariantWarningV1> {
    vec![
        ResumeVariantWarningV1 {
            code: ResumeVariantWarningCodeV1::AssistedContentNonAuthoritative,
            message: "The proposed resume variant is externally assisted review content and is not authoritative for deterministic analysis or matching."
                .to_owned(),
        },
        ResumeVariantWarningV1 {
            code: ResumeVariantWarningCodeV1::EvidenceOccurrenceNotFactualCertification,
            message: "Exact evidence occurrence does not certify that generated wording is factually entailed; every selected change requires human verification."
                .to_owned(),
        },
        ResumeVariantWarningV1 {
            code: ResumeVariantWarningCodeV1::DeterministicBaselinePreserved,
            message: "The original resume remains the preserved deterministic baseline."
                .to_owned(),
        },
    ]
}

fn prefix_error_path(mut error: ResumeEvaluationErrorV1, prefix: &str) -> ResumeEvaluationErrorV1 {
    error.field_path = format!("{prefix}.{}", error.field_path);
    error
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
    use crate::{
        INPUT_SCHEMA_VERSION, JOB_INPUT_SCHEMA_VERSION, JobInputMetadataV1, ResumeInputMetadataV1,
        ResumeVariantProposalV1, ResumeVariantSectionV1,
    };

    const RESUME: &str = "Morgan Lee\nSUMMARY\nProduct engineer building native apps.\nEXPERIENCE\nEngineer at Northstar Labs\nBuilt macOS workflows with Swift.\nSKILLS\nSwift, macOS, testing";
    const VACANCY: &str = "We need a product engineer with SwiftUI, macOS, accessibility, and cross-functional delivery experience.";

    fn change(
        start_line: usize,
        end_line: usize,
        original_text: &str,
        proposed_text: &str,
    ) -> ResumeVariantProposedChangeV1 {
        ResumeVariantProposedChangeV1 {
            section: ResumeVariantSectionV1::Experience,
            start_line,
            end_line,
            original_text: original_text.to_owned(),
            proposed_text: proposed_text.to_owned(),
            resume_evidence: vec![original_text.to_owned()],
            vacancy_evidence: vec!["SwiftUI".to_owned()],
        }
    }

    fn review_input() -> ResumeVariantReviewInputV1 {
        ResumeVariantReviewInputV1 {
            schema_version: RESUME_VARIANT_REVIEW_INPUT_SCHEMA_VERSION.to_owned(),
            resume: ResumeInputV1 {
                schema_version: INPUT_SCHEMA_VERSION.to_owned(),
                text: RESUME.to_owned(),
                metadata: ResumeInputMetadataV1::default(),
            },
            vacancy: crate::JobInputV1 {
                schema_version: JOB_INPUT_SCHEMA_VERSION.to_owned(),
                text: VACANCY.to_owned(),
                metadata: JobInputMetadataV1::default(),
            },
            proposal: ResumeVariantProposalV1 {
                schema_version: RESUME_VARIANT_PROPOSAL_SCHEMA_VERSION.to_owned(),
                changes: vec![
                    change(
                        3,
                        3,
                        "Product engineer building native apps.",
                        "Product engineer building native Apple-platform apps with SwiftUI.",
                    ),
                    change(
                        6,
                        6,
                        "Built macOS workflows with Swift.",
                        "Built native macOS workflows with Swift and SwiftUI.",
                    ),
                ],
            },
        }
    }

    #[test]
    fn review_is_canonical_and_preserves_baseline() {
        let input = review_input();
        let review = review_resume_variant(&input).expect("proposal should review");

        assert_eq!(review.baseline_resume, input.resume);
        assert_eq!(review.changes.len(), 2);
        assert_eq!(review.changes[0].change_id, "change-0001");
        assert_eq!(review.changes[1].change_id, "change-0002");
        assert!(review.discarded_changes.is_empty());
        assert!(review.proposed_preview_text.contains("Apple-platform"));
        assert!(review.proposed_preview_text.contains("Swift and SwiftUI"));
        assert_eq!(review.warnings.len(), 3);
    }

    #[test]
    fn materialization_applies_only_selected_changes() {
        let review_input = review_input();
        let materialized = materialize_resume_variant(&ResumeVariantMaterializationInputV1 {
            schema_version: RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION.to_owned(),
            expected_review_policy_version: RESUME_VARIANT_POLICY_VERSION.to_owned(),
            review_input: review_input.clone(),
            selected_change_ids: vec!["change-0002".to_owned()],
        })
        .expect("selected change should materialize");

        assert_eq!(materialized.baseline_resume, review_input.resume);
        assert!(
            materialized
                .assisted_resume_text
                .contains("Product engineer building native apps.")
        );
        assert!(
            materialized
                .assisted_resume_text
                .contains("Built native macOS workflows with Swift and SwiftUI.")
        );
        assert_eq!(materialized.selected_changes.len(), 1);
        assert_eq!(materialized.selected_changes[0].change_id, "change-0002");
    }

    #[test]
    fn invalid_and_overlapping_changes_are_discarded_without_payload_echo() {
        let mut input = review_input();
        input
            .proposal
            .changes
            .push(change(3, 6, "private wrong target", "private replacement"));
        input.proposal.changes.push(change(
            6,
            6,
            "Built macOS workflows with Swift.",
            "Overlapping replacement",
        ));
        let review = review_resume_variant(&input).expect("invalid items should be discarded");

        assert_eq!(review.changes.len(), 1);
        assert_eq!(review.discarded_changes.len(), 3);
        assert_eq!(
            review.discarded_changes[0].code,
            ResumeVariantDiscardCodeV1::DuplicateOrOverlappingTarget
        );
        assert_eq!(
            review.discarded_changes[1].code,
            ResumeVariantDiscardCodeV1::TargetMismatch
        );
        assert_eq!(
            review.discarded_changes[2].code,
            ResumeVariantDiscardCodeV1::DuplicateOrOverlappingTarget
        );
        let encoded = serde_json::to_string(&review).expect("review should serialize");
        assert!(!encoded.contains("private wrong target"));
        assert!(!encoded.contains("private replacement"));
    }

    #[test]
    fn evidence_line_ranges_and_selection_fail_closed() {
        let mut invalid_evidence = review_input();
        invalid_evidence.proposal.changes[0].resume_evidence = vec!["invented evidence".to_owned()];
        let review = review_resume_variant(&invalid_evidence).expect("change should be discarded");
        assert_eq!(review.changes.len(), 1);
        assert_eq!(
            review.discarded_changes[0].code,
            ResumeVariantDiscardCodeV1::InvalidResumeEvidence
        );

        let mut invalid_line = review_input();
        invalid_line.proposal.changes[0].start_line = 0;
        let review = review_resume_variant(&invalid_line).expect("change should be discarded");
        assert_eq!(
            review.discarded_changes[0].code,
            ResumeVariantDiscardCodeV1::InvalidLineRange
        );

        let error = materialize_resume_variant(&ResumeVariantMaterializationInputV1 {
            schema_version: RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION.to_owned(),
            expected_review_policy_version: RESUME_VARIANT_POLICY_VERSION.to_owned(),
            review_input: review_input(),
            selected_change_ids: vec!["provider-id".to_owned()],
        })
        .expect_err("unknown selection should fail");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::VariantSelectionInvalid
        );
        assert!(!error.message.contains("provider-id"));
    }

    #[test]
    fn mixed_line_endings_and_unicode_materialize_exactly() {
        let resume = "Zoë Lee\r\nSUMMARY\nEngineer 🚀\r\nSKILLS\nSwift";
        let mut input = review_input();
        input.resume.text = resume.to_owned();
        input.proposal.changes = vec![change(3, 3, "Engineer 🚀", "Native engineer 🚀")];
        input.proposal.changes[0].resume_evidence = vec!["Engineer 🚀".to_owned()];
        let materialized = materialize_resume_variant(&ResumeVariantMaterializationInputV1 {
            schema_version: RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION.to_owned(),
            expected_review_policy_version: RESUME_VARIANT_POLICY_VERSION.to_owned(),
            review_input: input,
            selected_change_ids: vec!["change-0001".to_owned()],
        })
        .expect("mixed line endings should be preserved outside the change");

        assert_eq!(
            materialized.assisted_resume_text,
            "Zoë Lee\r\nSUMMARY\nNative engineer 🚀\r\nSKILLS\nSwift"
        );
    }

    #[test]
    fn control_characters_invalid_preview_and_policy_fail_conservatively() {
        let mut control = review_input();
        control.proposal.changes[0].proposed_text = "unsafe\u{0000}value".to_owned();
        let review = review_resume_variant(&control).expect("unsafe change should be discarded");
        assert_eq!(review.changes.len(), 1);
        assert_eq!(
            review.discarded_changes[0].code,
            ResumeVariantDiscardCodeV1::UnsupportedControlCharacter
        );

        let mut invalid_preview = review_input();
        invalid_preview.proposal.changes[0].proposed_text = "x".repeat(2_001);
        let error = review_resume_variant(&invalid_preview).expect_err("line limit should fail");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::VariantPreviewInvalid
        );
        assert!(!error.message.contains(&"x".repeat(100)));

        let error = materialize_resume_variant(&ResumeVariantMaterializationInputV1 {
            schema_version: RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION.to_owned(),
            expected_review_policy_version: "future-policy".to_owned(),
            review_input: review_input(),
            selected_change_ids: vec!["change-0001".to_owned()],
        })
        .expect_err("unsupported policy should fail");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::UnsupportedVariantPolicyVersion
        );
        assert!(!error.message.contains("future-policy"));
    }

    #[test]
    fn proposal_and_selection_limits_are_enforced() {
        let mut input = review_input();
        input.proposal.changes = (0..=MAX_RESUME_VARIANT_CHANGES)
            .map(|_| {
                change(
                    3,
                    3,
                    "Product engineer building native apps.",
                    "Replacement",
                )
            })
            .collect();
        let error = review_resume_variant(&input).expect_err("change list should be bounded");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::VariantChangeListTooLong
        );

        let error = materialize_resume_variant(&ResumeVariantMaterializationInputV1 {
            schema_version: RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION.to_owned(),
            expected_review_policy_version: RESUME_VARIANT_POLICY_VERSION.to_owned(),
            review_input: review_input(),
            selected_change_ids: Vec::new(),
        })
        .expect_err("empty selection should fail");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::VariantSelectionEmpty
        );
    }
}
