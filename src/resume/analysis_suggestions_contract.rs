use serde::{Deserialize, Serialize};

use super::analysis_contract::{
    ResumeAnalysisActionV1, ResumeAnalysisCategoryV1, ResumeAnalysisCheckIdV1,
    ResumeAnalysisFindingStatusV1, ResumeAnalysisV1,
};
use super::contract::{ResumeErrorV1, ResumeInputV1};

pub const RESUME_ANALYSIS_SUGGESTION_PROPOSAL_SCHEMA_VERSION: &str =
    "career.resume_analysis_suggestion_proposal.v1";
pub const RESUME_ANALYSIS_SUGGESTION_REVIEW_INPUT_SCHEMA_VERSION: &str =
    "career.resume_analysis_suggestion_review_input.v1";
pub const RESUME_ANALYSIS_SUGGESTION_REVIEW_SCHEMA_VERSION: &str =
    "career.resume_analysis_suggestion_review.v1";
pub const RESUME_ANALYSIS_SUGGESTION_REVIEW_POLICY_VERSION: &str =
    "resume_analysis_suggestion_review_v1";

pub const MAX_RESUME_ANALYSIS_SUGGESTIONS: usize = 3;
pub const MAX_RESUME_ANALYSIS_SUGGESTION_PROPOSAL_CHARACTERS: usize = 4_000;
pub const MAX_RESUME_ANALYSIS_SUGGESTION_TARGET_CHARACTERS: usize = 500;
pub const MAX_RESUME_ANALYSIS_SUGGESTION_TEXT_CHARACTERS: usize = 600;
pub const MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_ITEMS: usize = 2;
pub const MAX_RESUME_ANALYSIS_SUGGESTION_EVIDENCE_CHARACTERS: usize = 240;

pub type ResumeAnalysisSuggestionErrorV1 = ResumeErrorV1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisProposedSuggestionV1 {
    pub basis_check_id: ResumeAnalysisCheckIdV1,
    pub start_line: usize,
    pub end_line: usize,
    pub source_target: String,
    pub source_evidence: Vec<String>,
    pub suggestion: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisSuggestionProposalV1 {
    pub schema_version: String,
    pub suggestions: Vec<ResumeAnalysisProposedSuggestionV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisSuggestionReviewInputV1 {
    pub schema_version: String,
    pub expected_analysis_policy_version: String,
    pub resume: ResumeInputV1,
    pub proposal: ResumeAnalysisSuggestionProposalV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisSuggestionDiscardCodeV1 {
    InvalidLineRange,
    TargetMismatch,
    InvalidTarget,
    InvalidSuggestion,
    UnsupportedControlCharacter,
    InvalidSourceEvidence,
    BasisActionNotCurrent,
    DuplicateOrOverlappingTarget,
    DuplicateBasisAction,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisDiscardedSuggestionV1 {
    pub input_index: usize,
    pub code: ResumeAnalysisSuggestionDiscardCodeV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisCanonicalSuggestionV1 {
    pub suggestion_id: String,
    pub priority: u8,
    pub area: ResumeAnalysisCategoryV1,
    pub basis_check_id: ResumeAnalysisCheckIdV1,
    pub status: ResumeAnalysisFindingStatusV1,
    pub improvement_action: String,
    pub start_line: usize,
    pub end_line: usize,
    pub source_target: String,
    pub source_evidence: Vec<String>,
    pub suggestion: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisSuggestionAuthorityV1 {
    AssistedNonAuthoritative,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisSuggestionWarningCodeV1 {
    AssistedSuggestionsNonAuthoritative,
    EvidenceOccurrenceNotFactualOrRewriteCertification,
    DeterministicAnalysisPreserved,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisSuggestionWarningV1 {
    pub code: ResumeAnalysisSuggestionWarningCodeV1,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisSuggestionReviewV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub analysis_policy_version: String,
    pub core_version: String,
    pub authority: ResumeAnalysisSuggestionAuthorityV1,
    pub baseline_analysis: ResumeAnalysisV1,
    pub suggestions: Vec<ResumeAnalysisCanonicalSuggestionV1>,
    pub discarded_suggestions: Vec<ResumeAnalysisDiscardedSuggestionV1>,
    pub warnings: Vec<ResumeAnalysisSuggestionWarningV1>,
}

pub(crate) struct ValidatedAnalysisSuggestion {
    pub(crate) input_index: usize,
    pub(crate) byte_range: std::ops::Range<usize>,
    pub(crate) proposal: ResumeAnalysisProposedSuggestionV1,
    pub(crate) action: ResumeAnalysisActionV1,
}
