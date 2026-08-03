use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use super::analysis::analyze_resume;
use super::analysis_contract::{ANALYSIS_POLICY_VERSION, ResumeAnalysisV1};
use super::analysis_replacements_contract::{
    MAX_RESUME_ANALYSIS_REPLACEMENT_CHARACTERS,
    MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_CHARACTERS,
    MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_ITEMS,
    MAX_RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_CHARACTERS,
    MAX_RESUME_ANALYSIS_REPLACEMENT_TARGET_CHARACTERS, MAX_RESUME_ANALYSIS_REPLACEMENTS,
    RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_SCHEMA_VERSION,
    RESUME_ANALYSIS_REPLACEMENT_REVIEW_INPUT_SCHEMA_VERSION,
    RESUME_ANALYSIS_REPLACEMENT_REVIEW_POLICY_VERSION,
    RESUME_ANALYSIS_REPLACEMENT_REVIEW_SCHEMA_VERSION, ResumeAnalysisCanonicalReplacementV1,
    ResumeAnalysisDiscardedReplacementV1, ResumeAnalysisProposedReplacementV1,
    ResumeAnalysisReplacementAuthorityV1, ResumeAnalysisReplacementDiscardCodeV1,
    ResumeAnalysisReplacementProposalV1, ResumeAnalysisReplacementReviewInputV1,
    ResumeAnalysisReplacementReviewV1, ResumeAnalysisReplacementWarningCodeV1,
    ResumeAnalysisReplacementWarningV1, ValidatedAnalysisReplacement,
};
use super::contract::{ResumeEvaluationErrorCodeV1, ResumeEvaluationErrorV1};

/// Reviews bounded external exact replacements without changing the
/// deterministic analysis baseline or producing a resume candidate.
pub fn review_resume_analysis_replacements(
    input: &ResumeAnalysisReplacementReviewInputV1,
) -> Result<ResumeAnalysisReplacementReviewV1, ResumeEvaluationErrorV1> {
    validate_review_envelope(input)?;
    validate_proposal_bounds(&input.proposal)?;
    let baseline_analysis =
        analyze_resume(&input.resume).map_err(|error| prefix_error_path(error, "resume"))?;
    let line_ranges = source_line_ranges(&input.resume.text);

    let mut discarded_replacements = Vec::new();
    let mut candidates = Vec::new();
    for (input_index, replacement) in input.proposal.replacements.iter().enumerate() {
        match validate_replacement(
            input_index,
            replacement,
            &input.resume.text,
            &line_ranges,
            &baseline_analysis,
        ) {
            Ok(validated) => candidates.push(validated),
            Err(code) => discarded_replacements
                .push(ResumeAnalysisDiscardedReplacementV1 { input_index, code }),
        }
    }

    let overlapping = overlapping_candidate_indices(&candidates);
    let duplicate_actions = duplicate_action_candidate_indices(&candidates);
    let mut retained = Vec::new();
    for (candidate_index, candidate) in candidates.into_iter().enumerate() {
        let discard_code = if overlapping.contains(&candidate_index) {
            Some(ResumeAnalysisReplacementDiscardCodeV1::DuplicateOrOverlappingTarget)
        } else if duplicate_actions.contains(&candidate_index) {
            Some(ResumeAnalysisReplacementDiscardCodeV1::DuplicateBasisAction)
        } else {
            None
        };
        if let Some(code) = discard_code {
            discarded_replacements.push(ResumeAnalysisDiscardedReplacementV1 {
                input_index: candidate.input_index,
                code,
            });
        } else {
            retained.push(candidate);
        }
    }

    discarded_replacements.sort_by_key(|discarded| discarded.input_index);
    retained.sort_by(|left, right| {
        left.action
            .priority
            .cmp(&right.action.priority)
            .then(left.byte_range.start.cmp(&right.byte_range.start))
            .then(left.byte_range.end.cmp(&right.byte_range.end))
            .then(left.input_index.cmp(&right.input_index))
    });
    let replacements = retained
        .iter()
        .enumerate()
        .map(|(index, replacement)| canonical_replacement(index, replacement))
        .collect();

    Ok(ResumeAnalysisReplacementReviewV1 {
        schema_version: RESUME_ANALYSIS_REPLACEMENT_REVIEW_SCHEMA_VERSION.to_owned(),
        policy_version: RESUME_ANALYSIS_REPLACEMENT_REVIEW_POLICY_VERSION.to_owned(),
        analysis_policy_version: baseline_analysis.policy_version.clone(),
        core_version: env!("CARGO_PKG_VERSION").to_owned(),
        authority: ResumeAnalysisReplacementAuthorityV1::AssistedNonAuthoritative,
        baseline_analysis,
        replacements,
        discarded_replacements,
        warnings: replacement_warnings(),
    })
}

fn validate_review_envelope(
    input: &ResumeAnalysisReplacementReviewInputV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    if input.schema_version != RESUME_ANALYSIS_REPLACEMENT_REVIEW_INPUT_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisReplacementReviewInputSchemaVersion,
            format!(
                "schema_version must be {RESUME_ANALYSIS_REPLACEMENT_REVIEW_INPUT_SCHEMA_VERSION}."
            ),
            "schema_version",
        ));
    }
    if input.proposal.schema_version != RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_SCHEMA_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisReplacementProposalSchemaVersion,
            format!(
                "proposal.schema_version must be {RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_SCHEMA_VERSION}."
            ),
            "proposal.schema_version",
        ));
    }
    if input.expected_analysis_policy_version != ANALYSIS_POLICY_VERSION {
        return Err(error(
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisReplacementAnalysisPolicyVersion,
            format!("expected_analysis_policy_version must be {ANALYSIS_POLICY_VERSION}."),
            "expected_analysis_policy_version",
        ));
    }
    Ok(())
}

fn validate_proposal_bounds(
    proposal: &ResumeAnalysisReplacementProposalV1,
) -> Result<(), ResumeEvaluationErrorV1> {
    if proposal.replacements.len() > MAX_RESUME_ANALYSIS_REPLACEMENTS {
        return Err(error(
            ResumeEvaluationErrorCodeV1::AnalysisReplacementListTooLong,
            format!(
                "proposal.replacements must contain at most {MAX_RESUME_ANALYSIS_REPLACEMENTS} items."
            ),
            "proposal.replacements",
        ));
    }
    let character_count = proposal_character_count(proposal);
    if character_count > MAX_RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_CHARACTERS {
        return Err(error(
            ResumeEvaluationErrorCodeV1::AnalysisReplacementProposalTooLarge,
            format!(
                "The analysis-replacement proposal must contain at most {MAX_RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_CHARACTERS} string characters; received {character_count}."
            ),
            "proposal",
        ));
    }
    Ok(())
}

fn proposal_character_count(proposal: &ResumeAnalysisReplacementProposalV1) -> usize {
    let mut total = proposal.schema_version.chars().count();
    for replacement in &proposal.replacements {
        total = total.saturating_add(replacement.source_target.chars().count());
        total = total.saturating_add(replacement.proposed_replacement.chars().count());
        for evidence in &replacement.source_evidence {
            total = total.saturating_add(evidence.chars().count());
        }
    }
    total
}

fn validate_replacement(
    input_index: usize,
    replacement: &ResumeAnalysisProposedReplacementV1,
    resume_text: &str,
    line_ranges: &[Range<usize>],
    analysis: &ResumeAnalysisV1,
) -> Result<ValidatedAnalysisReplacement, ResumeAnalysisReplacementDiscardCodeV1> {
    let Some(byte_range) =
        byte_range_for_lines(replacement.start_line, replacement.end_line, line_ranges)
    else {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::InvalidLineRange);
    };
    if replacement.source_target.trim().is_empty()
        || replacement.source_target.chars().count()
            > MAX_RESUME_ANALYSIS_REPLACEMENT_TARGET_CHARACTERS
    {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::InvalidTarget);
    }
    if resume_text.get(byte_range.clone()) != Some(replacement.source_target.as_str()) {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::TargetMismatch);
    }
    if replacement.source_target == replacement.proposed_replacement {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::NoChange);
    }
    if replacement.proposed_replacement.chars().count() > MAX_RESUME_ANALYSIS_REPLACEMENT_CHARACTERS
    {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::ReplacementTooLong);
    }
    if contains_unsupported_control(&replacement.proposed_replacement) {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::UnsupportedControlCharacter);
    }
    if !valid_source_evidence(&replacement.source_evidence, resume_text) {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::InvalidSourceEvidence);
    }
    let Some(action) = analysis
        .improvement_actions
        .iter()
        .find(|action| action.basis_check_id == replacement.basis_check_id)
    else {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::BasisActionNotCurrent);
    };
    let Some(check) = analysis
        .checks
        .iter()
        .find(|check| check.check_id == replacement.basis_check_id)
    else {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::BasisActionNotCurrent);
    };
    if check.passed {
        return Err(ResumeAnalysisReplacementDiscardCodeV1::BasisActionNotCurrent);
    }

    Ok(ValidatedAnalysisReplacement {
        input_index,
        byte_range,
        proposal: replacement.clone(),
        action: action.clone(),
    })
}

fn valid_source_evidence(values: &[String], source: &str) -> bool {
    if !(1..=MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_ITEMS).contains(&values.len()) {
        return false;
    }
    let mut seen = BTreeSet::new();
    values.iter().all(|value| {
        !value.trim().is_empty()
            && value.chars().count() <= MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_CHARACTERS
            && source.contains(value)
            && seen.insert(value)
    })
}

fn contains_unsupported_control(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
}

fn overlapping_candidate_indices(candidates: &[ValidatedAnalysisReplacement]) -> BTreeSet<usize> {
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
    candidates: &[ValidatedAnalysisReplacement],
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

fn canonical_replacement(
    index: usize,
    replacement: &ValidatedAnalysisReplacement,
) -> ResumeAnalysisCanonicalReplacementV1 {
    ResumeAnalysisCanonicalReplacementV1 {
        replacement_id: format!("replacement-{:04}", index + 1),
        priority: replacement.action.priority,
        area: replacement.action.area,
        basis_check_id: replacement.action.basis_check_id,
        status: replacement.action.status,
        improvement_action: replacement.action.action.clone(),
        start_line: replacement.proposal.start_line,
        end_line: replacement.proposal.end_line,
        source_target: replacement.proposal.source_target.clone(),
        source_evidence: replacement.proposal.source_evidence.clone(),
        proposed_replacement: replacement.proposal.proposed_replacement.clone(),
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

fn replacement_warnings() -> Vec<ResumeAnalysisReplacementWarningV1> {
    vec![
        ResumeAnalysisReplacementWarningV1 {
            code: ResumeAnalysisReplacementWarningCodeV1::AssistedReplacementsNonAuthoritative,
            message: "Retained replacements are externally assisted review content and are not authoritative for deterministic analysis, scoring, evidence, or matching."
                .to_owned(),
        },
        ResumeAnalysisReplacementWarningV1 {
            code: ResumeAnalysisReplacementWarningCodeV1::EvidenceOccurrenceNotFactualOrRewriteCertification,
            message: "Exact target and evidence occurrence does not establish factual entailment or certify any rewrite; human review is required."
                .to_owned(),
        },
        ResumeAnalysisReplacementWarningV1 {
            code: ResumeAnalysisReplacementWarningCodeV1::DeterministicAnalysisPreserved,
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
        proposed_replacement: &str,
    ) -> ResumeAnalysisProposedReplacementV1 {
        ResumeAnalysisProposedReplacementV1 {
            basis_check_id,
            start_line,
            end_line,
            source_target: source_target.to_owned(),
            source_evidence: vec![source_target.to_owned()],
            proposed_replacement: proposed_replacement.to_owned(),
        }
    }

    fn review_input() -> ResumeAnalysisReplacementReviewInputV1 {
        ResumeAnalysisReplacementReviewInputV1 {
            schema_version: RESUME_ANALYSIS_REPLACEMENT_REVIEW_INPUT_SCHEMA_VERSION.to_owned(),
            expected_analysis_policy_version: ANALYSIS_POLICY_VERSION.to_owned(),
            resume: ResumeInputV1 {
                schema_version: INPUT_SCHEMA_VERSION.to_owned(),
                text: RESUME.to_owned(),
                metadata: ResumeInputMetadataV1 {
                    document_id: Some("synthetic-analysis-replacements".to_owned()),
                },
            },
            proposal: ResumeAnalysisReplacementProposalV1 {
                schema_version: RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_SCHEMA_VERSION.to_owned(),
                replacements: vec![proposed(
                    super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                    10,
                    10,
                    "- Built reliable APIs for internal teams.",
                    "- Built reliable APIs for internal teams, improving deployment reliability by 30 percent.",
                )],
            },
        }
    }

    #[test]
    fn review_preserves_the_rerun_analysis_and_exact_before_after() {
        let input = review_input();
        let review =
            review_resume_analysis_replacements(&input).expect("replacement should review");

        assert_eq!(
            review.baseline_analysis,
            analyze_resume(&input.resume).expect("analysis")
        );
        assert_eq!(review.replacements.len(), 1);
        assert_eq!(review.replacements[0].replacement_id, "replacement-0001");
        assert_eq!(review.replacements[0].priority, 1);
        assert_eq!(
            review.replacements[0].status,
            ResumeAnalysisFindingStatusV1::Confirmed
        );
        assert_eq!(
            review.replacements[0].source_target,
            "- Built reliable APIs for internal teams."
        );
        assert_eq!(
            review.replacements[0].proposed_replacement,
            "- Built reliable APIs for internal teams, improving deployment reliability by 30 percent."
        );
        assert_eq!(review.discarded_replacements, Vec::new());
        assert_eq!(review.warnings.len(), 3);
    }

    #[test]
    fn invalid_duplicate_and_overlapping_items_are_discarded_without_payload_echo() {
        let mut input = review_input();
        input.proposal.replacements.extend([
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                10,
                10,
                "- Built reliable APIs for internal teams.",
                "Private duplicate replacement.",
            ),
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                10,
                11,
                "- Built reliable APIs for internal teams.\n- Reduced deployment failures by 30 percent.",
                "Private overlapping replacement.",
            ),
        ]);
        let review = review_resume_analysis_replacements(&input)
            .expect("items should discard conservatively");

        assert!(review.replacements.is_empty());
        assert_eq!(review.discarded_replacements.len(), 3);
        assert!(review.discarded_replacements.iter().all(|discarded| {
            discarded.code == ResumeAnalysisReplacementDiscardCodeV1::DuplicateOrOverlappingTarget
        }));
        let encoded = serde_json::to_string(&review).expect("review should serialize");
        assert!(!encoded.contains("Private duplicate replacement."));
        assert!(!encoded.contains("Private overlapping replacement."));
    }

    #[test]
    fn replacements_require_current_actions_exact_targets_and_changes() {
        let mut no_current_action = review_input();
        no_current_action.proposal.replacements[0].basis_check_id =
            super::super::analysis_contract::ResumeAnalysisCheckIdV1::ContactEmailPresent;
        let review = review_resume_analysis_replacements(&no_current_action)
            .expect("passing basis check should discard");
        assert_eq!(
            review.discarded_replacements[0].code,
            ResumeAnalysisReplacementDiscardCodeV1::BasisActionNotCurrent
        );

        let mut target_mismatch = review_input();
        target_mismatch.proposal.replacements[0].source_target =
            "private mismatched target".to_owned();
        let review = review_resume_analysis_replacements(&target_mismatch)
            .expect("mismatched target should discard");
        assert_eq!(
            review.discarded_replacements[0].code,
            ResumeAnalysisReplacementDiscardCodeV1::TargetMismatch
        );
        let encoded = serde_json::to_string(&review).expect("review should serialize");
        assert!(!encoded.contains("private mismatched target"));

        let mut no_change = review_input();
        no_change.proposal.replacements[0].proposed_replacement =
            no_change.proposal.replacements[0].source_target.clone();
        let review =
            review_resume_analysis_replacements(&no_change).expect("unchanged target discards");
        assert_eq!(
            review.discarded_replacements[0].code,
            ResumeAnalysisReplacementDiscardCodeV1::NoChange
        );
    }

    #[test]
    fn provisional_actions_remain_provisional_and_duplicate_actions_fail_closed() {
        let mut input = review_input();
        input.resume.text = "Morgan Lee\nmorgan.lee@example.com\nBackend engineer focused on reliable APIs and platform stability.\nPlatform Engineer at Acme Corp since 2021\nDelivered internal APIs used by support and operations teams.\nReduced deployment errors by 30 percent through release automation.\nComfortable with Python and Django for backend services\nBachelor of Science in Computer Science\nExample University\n2016 - 2020".to_owned();
        input.proposal.replacements = vec![proposed(
            super::super::analysis_contract::ResumeAnalysisCheckIdV1::ExperienceBulletsPresent,
            5,
            5,
            "Delivered internal APIs used by support and operations teams.",
            "Delivered internal APIs used by support and operations teams with verified impact.",
        )];
        let review =
            review_resume_analysis_replacements(&input).expect("provisional action reviews");
        assert_eq!(
            review.replacements[0].status,
            ResumeAnalysisFindingStatusV1::Provisional
        );

        let mut duplicate = review_input();
        duplicate.proposal.replacements = vec![
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                10,
                10,
                "- Built reliable APIs for internal teams.",
                "First private action replacement.",
            ),
            proposed(
                super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                11,
                11,
                "- Reduced deployment failures by 30 percent.",
                "Second private action replacement.",
            ),
        ];
        let review = review_resume_analysis_replacements(&duplicate)
            .expect("duplicates discard conservatively");
        assert!(review.replacements.is_empty());
        assert!(review.discarded_replacements.iter().all(|discarded| {
            discarded.code == ResumeAnalysisReplacementDiscardCodeV1::DuplicateBasisAction
        }));
    }

    #[test]
    fn mixed_line_endings_and_unicode_remain_exactly_grounded() {
        let mut input = review_input();
        input.resume.text = RESUME
            .replace("Morgan Lee", "Morgån Lee")
            .replace('\n', "\r\n");
        input.proposal.replacements[0].proposed_replacement =
            "- Built reliable APIs for internal teams with verified impact 🚀.".to_owned();
        let review = review_resume_analysis_replacements(&input)
            .expect("mixed-line-ending replacement should review");

        assert_eq!(review.replacements.len(), 1);
        assert_eq!(
            review.replacements[0].source_target,
            "- Built reliable APIs for internal teams."
        );
        assert_eq!(
            review.replacements[0].proposed_replacement,
            "- Built reliable APIs for internal teams with verified impact 🚀."
        );
    }

    #[test]
    fn replacement_limits_controls_evidence_and_versions_are_enforced() {
        let mut input = review_input();
        input.proposal.replacements = (0..=MAX_RESUME_ANALYSIS_REPLACEMENTS)
            .map(|_| input.proposal.replacements[0].clone())
            .collect();
        let error =
            review_resume_analysis_replacements(&input).expect_err("list limit should fail");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::AnalysisReplacementListTooLong
        );

        let mut oversized = review_input();
        oversized.proposal.replacements = (0..MAX_RESUME_ANALYSIS_REPLACEMENTS)
            .map(|_| ResumeAnalysisProposedReplacementV1 {
                basis_check_id:
                    super::super::analysis_contract::ResumeAnalysisCheckIdV1::MeasurableImpactInBullets,
                start_line: 1,
                end_line: 1,
                source_target: "a".repeat(MAX_RESUME_ANALYSIS_REPLACEMENT_TARGET_CHARACTERS),
                source_evidence: vec![
                    "b".repeat(MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_CHARACTERS),
                    "c".repeat(MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_CHARACTERS),
                ],
                proposed_replacement: "d".repeat(MAX_RESUME_ANALYSIS_REPLACEMENT_CHARACTERS),
            })
            .collect();
        let error = review_resume_analysis_replacements(&oversized)
            .expect_err("proposal character limit should fail");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::AnalysisReplacementProposalTooLarge
        );

        let mut replacement_too_long = review_input();
        replacement_too_long.proposal.replacements[0].proposed_replacement =
            "x".repeat(MAX_RESUME_ANALYSIS_REPLACEMENT_CHARACTERS + 1);
        let review = review_resume_analysis_replacements(&replacement_too_long)
            .expect("too-long replacement discards");
        assert_eq!(
            review.discarded_replacements[0].code,
            ResumeAnalysisReplacementDiscardCodeV1::ReplacementTooLong
        );

        let mut controls = review_input();
        controls.proposal.replacements[0].proposed_replacement =
            "unsafe\u{0000}replacement".to_owned();
        let review =
            review_resume_analysis_replacements(&controls).expect("control replacement discards");
        assert_eq!(
            review.discarded_replacements[0].code,
            ResumeAnalysisReplacementDiscardCodeV1::UnsupportedControlCharacter
        );

        let mut invalid_evidence = review_input();
        invalid_evidence.proposal.replacements[0].source_evidence =
            vec!["private evidence".to_owned()];
        let review = review_resume_analysis_replacements(&invalid_evidence)
            .expect("ungrounded evidence discards");
        assert_eq!(
            review.discarded_replacements[0].code,
            ResumeAnalysisReplacementDiscardCodeV1::InvalidSourceEvidence
        );
        let encoded = serde_json::to_string(&review).expect("review should serialize");
        assert!(!encoded.contains("private evidence"));

        let mut stale = review_input();
        stale.expected_analysis_policy_version = "future-policy".to_owned();
        let error = review_resume_analysis_replacements(&stale).expect_err("stale policy fails");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisReplacementAnalysisPolicyVersion
        );
        assert!(!error.message.contains("future-policy"));

        let mut unsupported_input_schema = review_input();
        unsupported_input_schema.schema_version = "future-schema".to_owned();
        let error = review_resume_analysis_replacements(&unsupported_input_schema)
            .expect_err("unsupported input schema fails");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisReplacementReviewInputSchemaVersion
        );
        assert!(!error.message.contains("future-schema"));

        let mut unsupported_proposal_schema = review_input();
        unsupported_proposal_schema.proposal.schema_version = "future-schema".to_owned();
        let error = review_resume_analysis_replacements(&unsupported_proposal_schema)
            .expect_err("unsupported proposal schema fails");
        assert_eq!(
            error.code,
            ResumeEvaluationErrorCodeV1::UnsupportedAnalysisReplacementProposalSchemaVersion
        );
        assert!(!error.message.contains("future-schema"));
    }
}
