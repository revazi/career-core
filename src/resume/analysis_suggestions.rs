use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use super::analysis::analyze_resume;
use super::analysis_contract::{ANALYSIS_POLICY_VERSION, ResumeAnalysisV1};
use super::analysis_suggestions_contract::{
    MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_CHARACTERS,
    MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_ITEMS,
    MAX_RESUME_ANALYSIS_SUGGESTION_PROPOSAL_CHARACTERS,
    MAX_RESUME_ANALYSIS_SUGGESTION_TARGET_CHARACTERS,
    MAX_RESUME_ANALYSIS_SUGGESTION_TEXT_CHARACTERS, MAX_RESUME_ANALYSIS_SUGGESTIONS,
    RESUME_ANALYSIS_SUGGESTION_PROPOSAL_SCHEMA_VERSION,
    RESUME_ANALYSIS_SUGGESTION_REVIEW_INPUT_SCHEMA_VERSION,
    RESUME_ANALYSIS_SUGGESTION_REVIEW_POLICY_VERSION,
    RESUME_ANALYSIS_SUGGESTION_REVIEW_SCHEMA_VERSION, ResumeAnalysisCanonicalSuggestionV1,
    ResumeAnalysisDiscardedSuggestionV1, ResumeAnalysisProposedSuggestionV1,
    ResumeAnalysisSuggestionAuthorityV1, ResumeAnalysisSuggestionDiscardCodeV1,
    ResumeAnalysisSuggestionProposalV1, ResumeAnalysisSuggestionReviewInputV1,
    ResumeAnalysisSuggestionReviewV1, ResumeAnalysisSuggestionWarningCodeV1,
    ResumeAnalysisSuggestionWarningV1, ValidatedAnalysisSuggestion,
};
use super::contract::{ResumeEvaluationErrorCodeV1, ResumeEvaluationErrorV1};

/// Reviews bounded external analysis suggestions without changing the
/// deterministic analysis baseline or producing a resume candidate.
pub fn review_resume_analysis_suggestions(
    input: &ResumeAnalysisSuggestionReviewInputV1,
) -> Result<ResumeAnalysisSuggestionReviewV1, ResumeEvaluationErrorV1> {
    validate_review_envelope(input)?;
    validate_proposal_bounds(&input.proposal)?;
    let baseline_analysis =
        analyze_resume(&input.resume).map_err(|error| prefix_error_path(error, "resume"))?;
    let line_ranges = source_line_ranges(&input.resume.text);

    let mut discarded_suggestions = Vec::new();
    let mut candidates = Vec::new();
    for (input_index, suggestion) in input.proposal.suggestions.iter().enumerate() {
        match validate_suggestion(
            input_index,
            suggestion,
            &input.resume.text,
            &line_ranges,
            &baseline_analysis,
        ) {
            Ok(validated) => candidates.push(validated),
            Err(code) => discarded_suggestions
                .push(ResumeAnalysisDiscardedSuggestionV1 { input_index, code }),
        }
    }

    let overlapping = overlapping_candidate_indices(&candidates);
    let duplicate_actions = duplicate_action_candidate_indices(&candidates);
    let mut retained = Vec::new();
    for (candidate_index, candidate) in candidates.into_iter().enumerate() {
        let discard_code = if overlapping.contains(&candidate_index) {
            Some(ResumeAnalysisSuggestionDiscardCodeV1::DuplicateOrOverlappingTarget)
        } else if duplicate_actions.contains(&candidate_index) {
            Some(ResumeAnalysisSuggestionDiscardCodeV1::DuplicateBasisAction)
        } else {
            None
        };
        if let Some(code) = discard_code {
            discarded_suggestions.push(ResumeAnalysisDiscardedSuggestionV1 {
                input_index: candidate.input_index,
                code,
            });
        } else {
            retained.push(candidate);
        }
    }

    discarded_suggestions.sort_by_key(|discarded| discarded.input_index);
    retained.sort_by(|left, right| {
        left.action
            .priority
            .cmp(&right.action.priority)
            .then(left.byte_range.start.cmp(&right.byte_range.start))
            .then(left.byte_range.end.cmp(&right.byte_range.end))
            .then(left.input_index.cmp(&right.input_index))
    });
    let suggestions = retained
        .iter()
        .enumerate()
        .map(|(index, suggestion)| canonical_suggestion(index, suggestion))
        .collect();

    Ok(ResumeAnalysisSuggestionReviewV1 {
        schema_version: RESUME_ANALYSIS_SUGGESTION_REVIEW_SCHEMA_VERSION.to_owned(),
        policy_version: RESUME_ANALYSIS_SUGGESTION_REVIEW_POLICY_VERSION.to_owned(),
        analysis_policy_version: baseline_analysis.policy_version.clone(),
        core_version: env!("CARGO_PKG_VERSION").to_owned(),
        authority: ResumeAnalysisSuggestionAuthorityV1::AssistedNonAuthoritative,
        baseline_analysis,
        suggestions,
        discarded_suggestions,
        warnings: suggestion_warnings(),
    })
}

fn validate_review_envelope(
    input: &ResumeAnalysisSuggestionReviewInputV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    if input.schema_version != RESUME_ANALYSIS_SUGGESTION_REVIEW_INPUT_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisSuggestionReviewInputSchemaVersion,
            format!(
                "schema_version must be {RESUME_ANALYSIS_SUGGESTION_REVIEW_INPUT_SCHEMA_VERSION}."
            ),
            "schema_version",
        ));
    }
    if input.proposal.schema_version != RESUME_ANALYSIS_SUGGESTION_PROPOSAL_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisSuggestionProposalSchemaVersion,
            format!(
                "proposal.schema_version must be {RESUME_ANALYSIS_SUGGESTION_PROPOSAL_SCHEMA_VERSION}."
            ),
            "proposal.schema_version",
        ));
    }
    if input.expected_analysis_policy_version != ANALYSIS_POLICY_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisSuggestionAnalysisPolicyVersion,
            format!("expected_analysis_policy_version must be {ANALYSIS_POLICY_VERSION}."),
            "expected_analysis_policy_version",
        ));
    }
    Ok(())
}

fn validate_proposal_bounds(
    proposal: &ResumeAnalysisSuggestionProposalV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    if proposal.suggestions.len() > MAX_RESUME_ANALYSIS_SUGGESTIONS {
        return Err(error(
            ResumeEvaluationErrorCodeV1::AnalysisSuggestionListTooLong,
            format!(
                "proposal.suggestions must contain at most {MAX_RESUME_ANALYSIS_SUGGESTIONS} items."
            ),
            "proposal.suggestions",
        ));
    }
    let character_count = proposal_character_count(proposal);
    if character_count > MAX_RESUME_ANALYSIS_SUGGESTION_PROPOSAL_CHARACTERS {
        return Err(error(
            ResumeEvaluationErrorCodeV1::AnalysisSuggestionProposalTooLarge,
            format!(
                "The analysis-suggestion proposal must contain at most {MAX_RESUME_ANALYSIS_SUGGESTION_PROPOSAL_CHARACTERS} string characters; received {character_count}."
            ),
            "proposal",
        ));
    }
    Ok(())
}

fn proposal_character_count(proposal: &ResumeAnalysisSuggestionProposalV1) -> usize {
    let mut total = proposal.schema_version.chars().count();
    for suggestion in &proposal.suggestions {
        total = total.saturating_add(suggestion.source_target.chars().count());
        total = total.saturating_add(suggestion.suggestion.chars().count());
        for evidence in &suggestion.source_evidence {
            total = total.saturating_add(evidence.chars().count());
        }
    }
    total
}

fn validate_suggestion(
    input_index: usize,
    suggestion: &ResumeAnalysisProposedSuggestionV1,
    resume_text: &str,
    line_ranges: &[Range<usize>],
    analysis: &ResumeAnalysisV1,
) -> Result<ValidatedAnalysisSuggestion, ResumeAnalysisSuggestionDiscardCodeV1> {
    let Some(byte_range) =
        byte_range_for_lines(suggestion.start_line, suggestion.end_line, line_ranges)
    else {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::InvalidLineRange);
    };
    if suggestion.source_target.trim().is_empty()
        || suggestion.source_target.chars().count()
            > MAX_RESUME_ANALYSIS_SUGGESTION_TARGET_CHARACTERS
    {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::InvalidTarget);
    }
    if resume_text.get(byte_range.clone()) != Some(suggestion.source_target.as_str()) {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::TargetMismatch);
    }
    if suggestion.suggestion.trim().is_empty()
        || suggestion.suggestion.chars().count() > MAX_RESUME_ANALYSIS_SUGGESTION_TEXT_CHARACTERS
    {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::InvalidSuggestion);
    }
    if contains_unsupported_control(&suggestion.suggestion) {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::UnsupportedControlCharacter);
    }
    if !valid_source_evidence(&suggestion.source_evidence, resume_text) {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::InvalidSourceEvidence);
    }
    let Some(action) = analysis
        .improvement_actions
        .iter()
        .find(|action| action.basis_check_id == suggestion.basis_check_id)
    else {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::BasisActionNotCurrent);
    };
    let Some(check) = analysis
        .checks
        .iter()
        .find(|check| check.check_id == suggestion.basis_check_id)
    else {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::BasisActionNotCurrent);
    };
    if check.passed {
        return Err(ResumeAnalysisSuggestionDiscardCodeV1::BasisActionNotCurrent);
    }

    Ok(ValidatedAnalysisSuggestion {
        input_index,
        byte_range,
        proposal: suggestion.clone(),
        action: action.clone(),
    })
}

fn valid_source_evidence(values: &[String], source: &str) -> bool {
    if !(1..=MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_ITEMS).contains(&values.len()) {
        return false;
    }
    let mut seen = BTreeSet::new();
    values.iter().all(|value| {
        !value.trim().is_empty()
            && value.chars().count() <= MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_CHARACTERS
            && source.contains(value)
            && seen.insert(value)
    })
}

fn contains_unsupported_control(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
}

fn overlapping_candidate_indices(candidates: &[ValidatedAnalysisSuggestion]) -> BTreeSet<usize> {
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

fn duplicate_action_candidate_indices(
    candidates: &[ValidatedAnalysisSuggestion],
) -> BTreeSet<usize> {
    let mut action_candidates = BTreeMap::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        action_candidates
            .entry(candidate.proposal.basis_check_id)
            .or_insert_with(Vec::new)
            .push(candidate_index);
    }
    action_candidates
        .into_values()
        .filter(|indices| indices.len() > 1)
        .flatten()
        .collect()
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

fn canonical_suggestion(
    index: usize,
    suggestion: &ValidatedAnalysisSuggestion,
) -> ResumeAnalysisCanonicalSuggestionV1 {
    ResumeAnalysisCanonicalSuggestionV1 {
        suggestion_id: format!("suggestion-{:04}", index + 1),
        priority: suggestion.action.priority,
        area: suggestion.action.area,
        basis_check_id: suggestion.action.basis_check_id,
        status: suggestion.action.status,
        improvement_action: suggestion.action.action.clone(),
        start_line: suggestion.proposal.start_line,
        end_line: suggestion.proposal.end_line,
        source_target: suggestion.proposal.source_target.clone(),
        source_evidence: suggestion.proposal.source_evidence.clone(),
        suggestion: suggestion.proposal.suggestion.clone(),
    }
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

fn suggestion_warnings() -> Vec<ResumeAnalysisSuggestionWarningV1> {
    vec![
        ResumeAnalysisSuggestionWarningV1 {
            code: ResumeAnalysisSuggestionWarningCodeV1::AssistedSuggestionsNonAuthoritative,
            message: "Retained suggestions are externally assisted review content and are not authoritative for deterministic analysis, scoring, evidence, or matching."
                .to_owned(),
        },
        ResumeAnalysisSuggestionWarningV1 {
            code: ResumeAnalysisSuggestionWarningCodeV1::EvidenceOccurrenceNotFactualOrRewriteCertification,
            message: "Exact target and evidence occurrence does not establish factual entailment or certify any rewrite; human review is required."
                .to_owned(),
        },
        ResumeAnalysisSuggestionWarningV1 {
            code: ResumeAnalysisSuggestionWarningCodeV1::DeterministicAnalysisPreserved,
            message: "The current deterministic analysis baseline is preserved unchanged and remains authoritative."
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
        INPUT_SCHEMA_VERSION, ResumeAnalysisFindingStatusV1, ResumeInputMetadataV1, ResumeInputV1,
    };

    const RESUME: &str = "Morgan Lee\nmorgan.lee@example.com\n+1 555 010 0200\nhttps://example.com/morgan\nSUMMARY\nBackend engineer building reliable local services.\nEXPERIENCE\nPlatform Engineer at Acme Corp\n2021 - Present\n- Built reliable APIs for internal teams.\n- Reduced deployment failures by 30 percent.\nEDUCATION\nBachelor of Science in Computer Science, Example University\n2016 - 2020\nSKILLS\nRust, PostgreSQL, Docker, Testing, Git";

    fn proposed(
        basis_check_id: super::super::analysis_contract::ResumeAnalysisCheckIdV1,
        start_line: usize,
        end_line: usize,
        source_target: &str,
        suggestion: &str,
    ) -> ResumeAnalysisProposedSuggestionV1 {
        ResumeAnalysisProposedSuggestionV1 {
            basis_check_id,
            start_line,
            end_line,
            source_target: source_target.to_owned(),
            source_evidence: vec![source_target.to_owned()],
            suggestion: suggestion.to_owned(),
        }
    }

    fn review_input() -> ResumeAnalysisSuggestionReviewInputV1 {
        ResumeAnalysisSuggestionReviewInputV1 {
            schema_version: RESUME_ANALYSIS_SUGGESTION_REVIEW_INPUT_SCHEMA_VERSION.to_owned(),
            expected_analysis_policy_version: ANALYSIS_POLICY_VERSION.to_owned(),
            resume: ResumeInputV1 {
                schema_version: INPUT_SCHEMA_VERSION.to_owned(),
                text: RESUME.to_owned(),
                metadata: ResumeInputMetadataV1 {
                    document_id: Some("synthetic-analysis-suggestions".to_owned()),
                },
            },
            proposal: ResumeAnalysisSuggestionProposalV1 {
                schema_version: RESUME_ANALYSIS_SUGGESTION_PROPOSAL_SCHEMA_VERSION.to_owned(),
                suggestions: vec![proposed(
                    super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                    10,
                    10,
                    "- Built reliable APIs for internal teams.",
                    "Review this bullet and add a specific, source-supported outcome if one can be verified.",
                )],
            },
        }
    }

    #[test]
    fn review_preserves_the_rerun_analysis_and_current_action_status() {
        let input = review_input();
        let review = review_resume_analysis_suggestions(&input).expect("suggestion should review");

        assert_eq!(
            review.baseline_analysis,
            analyze_resume(&input.resume).expect("analysis")
        );
        assert_eq!(review.suggestions.len(), 1);
        assert_eq!(review.suggestions[0].suggestion_id, "suggestion-0001");
        assert_eq!(review.suggestions[0].priority, 1);
        assert_eq!(
            review.suggestions[0].status,
            ResumeAnalysisFindingStatusV1::Confirmed
        );
        assert_eq!(review.discarded_suggestions, Vec::new());
        assert_eq!(review.warnings.len(), 3);
    }

    #[test]
    fn invalid_duplicate_and_overlapping_items_are_discarded_without_payload_echo() {
        let mut input = review_input();
        input.proposal.suggestions.extend([
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                10,
                10,
                "- Built reliable APIs for internal teams.",
                "Private duplicate suggestion.",
            ),
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                10,
                11,
                "- Built reliable APIs for internal teams.\n- Reduced deployment failures by 30 percent.",
                "Private overlapping suggestion.",
            ),
        ]);
        let review = review_resume_analysis_suggestions(&input).expect("items should discard");

        assert!(review.suggestions.is_empty());
        assert_eq!(review.discarded_suggestions.len(), 3);
        assert!(review.discarded_suggestions.iter().all(|discarded| {
            discarded.code == ResumeAnalysisSuggestionDiscardCodeV1::DuplicateOrOverlappingTarget
        }));
        let encoded = serde_json::to_string(&review).expect("review should serialize");
        assert!(!encoded.contains("Private duplicate suggestion."));
        assert!(!encoded.contains("Private overlapping suggestion."));
    }

    #[test]
    fn suggestions_require_a_current_action_and_exact_source_binding() {
        let mut no_current_action = review_input();
        no_current_action.proposal.suggestions[0].basis_check_id =
            super::super::analysis_contract::ResumeAnalysisCheckIdV1::ContactEmailPresent;
        let review = review_resume_analysis_suggestions(&no_current_action)
            .expect("passing basis check should discard");
        assert_eq!(review.suggestions.len(), 0);
        assert_eq!(
            review.discarded_suggestions[0].code,
            ResumeAnalysisSuggestionDiscardCodeV1::BasisActionNotCurrent
        );

        let mut target_mismatch = review_input();
        target_mismatch.proposal.suggestions[0].source_target =
            "private mismatched target".to_owned();
        let review = review_resume_analysis_suggestions(&target_mismatch)
            .expect("mismatched target should discard");
        assert_eq!(
            review.discarded_suggestions[0].code,
            ResumeAnalysisSuggestionDiscardCodeV1::TargetMismatch
        );
        let encoded = serde_json::to_string(&review).expect("review should serialize");
        assert!(!encoded.contains("private mismatched target"));

        let mut invalid_evidence = review_input();
        invalid_evidence.proposal.suggestions[0].source_evidence =
            vec!["private evidence".to_owned()];
        let review = review_resume_analysis_suggestions(&invalid_evidence)
            .expect("ungrounded evidence should discard");
        assert_eq!(
            review.discarded_suggestions[0].code,
            ResumeAnalysisSuggestionDiscardCodeV1::InvalidSourceEvidence
        );
        let encoded = serde_json::to_string(&review).expect("review should serialize");
        assert!(!encoded.contains("private evidence"));

        let mut stale = review_input();
        stale.expected_analysis_policy_version = "future-policy".to_owned();
        let error = review_resume_analysis_suggestions(&stale).expect_err("stale policy fails");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisSuggestionAnalysisPolicyVersion
        );
        assert!(!error.message.contains("future-policy"));
    }

    #[test]
    fn provisional_actions_remain_provisional_and_duplicate_actions_fail_closed() {
        let mut input = review_input();
        input.resume.text = "Morgan Lee\nmorgan.lee@example.com\nBackend engineer focused on reliable APIs and platform stability.\nPlatform Engineer at Acme Corp since 2021\nDelivered internal APIs used by support and operations teams.\nReduced deployment errors by 30 percent through release automation.\nComfortable with Python and Django for backend services\nBachelor of Science in Computer Science\nExample University\n2016 - 2020".to_owned();
        input.proposal.suggestions = vec![proposed(
            super::super::analysis_contract::ResumeAnalysisCheckIdV1::ExperienceBulletsPresent,
            5,
            5,
            "Delivered internal APIs used by support and operations teams.",
            "Verify the extracted experience content before changing this resume text.",
        )];
        let review =
            review_resume_analysis_suggestions(&input).expect("provisional action reviews");
        assert_eq!(
            review.suggestions[0].status,
            ResumeAnalysisFindingStatusV1::Provisional
        );

        let mut duplicate = review_input();
        duplicate.proposal.suggestions = vec![
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                10,
                10,
                "- Built reliable APIs for internal teams.",
                "First private action suggestion.",
            ),
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                11,
                11,
                "- Reduced deployment failures by 30 percent.",
                "Second private action suggestion.",
            ),
        ];
        let review = review_resume_analysis_suggestions(&duplicate).expect("duplicates discard");
        assert!(review.suggestions.is_empty());
        assert!(review.discarded_suggestions.iter().all(|discarded| {
            discarded.code == ResumeAnalysisSuggestionDiscardCodeV1::DuplicateBasisAction
        }));
    }

    #[test]
    fn suggestion_limits_and_controls_are_enforced() {
        let mut input = review_input();
        input.proposal.suggestions = (0..=MAX_RESUME_ANALYSIS_SUGGESTIONS)
            .map(|_| input.proposal.suggestions[0].clone())
            .collect();
        let error = review_resume_analysis_suggestions(&input).expect_err("list limit fails");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::AnalysisSuggestionListTooLong
        );

        let mut oversized = review_input();
        oversized.proposal.suggestions = (0..MAX_RESUME_ANALYSIS_SUGGESTIONS)
            .map(|_| ResumeAnalysisProposedSuggestionV1 {
                basis_check_id:
                    super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                start_line: 1,
                end_line: 1,
                source_target: "a".repeat(MAX_RESUME_ANALYSIS_SUGGESTION_TARGET_CHARACTERS),
                source_evidence: vec![
                    "b".repeat(MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_CHARACTERS),
                    "c".repeat(MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_CHARACTERS),
                ],
                suggestion: "d".repeat(MAX_RESUME_ANALYSIS_SUGGESTION_TEXT_CHARACTERS),
            })
            .collect();
        let error = review_resume_analysis_suggestions(&oversized)
            .expect_err("proposal character limit fails");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::AnalysisSuggestionProposalTooLarge
        );

        let mut controls = review_input();
        controls.proposal.suggestions[0].suggestion = "unsafe\u{0000}suggestion".to_owned();
        let review = review_resume_analysis_suggestions(&controls).expect("control discards");
        assert_eq!(
            review.discarded_suggestions[0].code,
            ResumeAnalysisSuggestionDiscardCodeV1::UnsupportedControlCharacter
        );
    }
}
