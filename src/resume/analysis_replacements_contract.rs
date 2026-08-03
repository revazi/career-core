use serde::{Deserialize, Serialize};

use super::analysis_contract::{
    ResumeAnalysisActionV1, ResumeAnalysisCategoryV1, ResumeAnalysisCheckIdV1,
    ResumeAnalysisFindingStatusV1, ResumeAnalysisV1,
};
use super::contract::{ResumeErrorV1, ResumeInputV1};

pub const RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_SCHEMA_VERSION: &str =
    "career.resume_analysis_replacement_proposal.v1";
pub const RESUME_ANALYSIS_REPLACEMENT_REVIEW_INPUT_SCHEMA_VERSION: &str =
    "career.resume_analysis_replacement_review_input.v1";
pub const RESUME_ANALYSIS_REPLACEMENT_REVIEW_SCHEMA_VERSION: &str =
    "career.resume_analysis_replacement_review.v1";
pub const RESUME_ANALYSIS_REPLACEMENT_REVIEW_POLICY_VERSION: &str =
    "resume_analysis_replacement_review_v1";

pub const MAX_RESUME_ANALYSIS_REPLACEMENTS: usize = 3;
pub const MAX_RESUME_ANALYSIS_REPLACEMENT_PROPOSAL_CHARACTERS: usize = 4_000;
pub const MAX_RESUME_ANALYSIS_REPLACEMENT_TARGET_CHARACTERS: usize = 500;
pub const MAX_RESUME_ANALYSIS_REPLACEMENT_CHARACTERS: usize = 600;
pub const MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_ITEMS: usize = 2;
pub const MAX_RESUME_ANALYSIS_REPLACEMENT_EVIDENCE_CHARACTERS: usize = 240;

pub type ResumeAnalysisReplacementErrorV1 = ResumeErrorV1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisProposedReplacementV1 {
    pub basis_check_id: ResumeAnalysisCheckIdV1,
    pub start_line: usize,
    pub end_line: usize,
    pub source_target: String,
    pub source_evidence: Vec<String>,
    pub proposed_replacement: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisReplacementProposalV1 {
    pub schema_version: String,
    pub replacements: Vec<ResumeAnalysisProposedReplacementV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisReplacementReviewInputV1 {
    pub schema_version: String,
    pub expected_analysis_policy_version: String,
    pub resume: ResumeInputV1,
    pub proposal: ResumeAnalysisReplacementProposalV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisReplacementDiscardCodeV1 {
    InvalidLineRange,
    TargetMismatch,
    InvalidTarget,
    NoChange,
    ReplacementTooLong,
    UnsupportedControlCharacter,
    InvalidSourceEvidence,
    BasisActionNotCurrent,
    DuplicateOrOverlappingTarget,
    DuplicateBasisAction,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisDiscardedReplacementV1 {
    pub input_index: usize,
    pub code: ResumeAnalysisReplacementDiscardCodeV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisCanonicalReplacementV1 {
    pub replacement_id: String,
    pub priority: u8,
    pub area: ResumeAnalysisCategoryV1,
    pub basis_check_id: ResumeAnalysisCheckIdV1,
    pub status: ResumeAnalysisFindingStatusV1,
    pub improvement_action: String,
    pub start_line: usize,
    pub end_line: usize,
    pub source_target: String,
    pub source_evidence: Vec<String>,
    pub proposed_replacement: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisReplacementAuthorityV1 {
    AssistedNonAuthoritative,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisReplacementWarningCodeV1 {
    AssistedReplacementsNonAuthoritative,
    EvidenceOccurrenceNotFactualOrRewriteCertification,
    DeterministicAnalysisPreserved,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisReplacementWarningV1 {
    pub code: ResumeAnalysisReplacementWarningCodeV1,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisReplacementReviewV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub analysis_policy_version: String,
    pub core_version: String,
    pub authority: ResumeAnalysisReplacementAuthorityV1,
    pub baseline_analysis: ResumeAnalysisV1,
    pub replacements: Vec<ResumeAnalysisCanonicalReplacementV1>,
    pub discarded_replacements: Vec<ResumeAnalysisDiscardedReplacementV1>,
    pub warnings: Vec<ResumeAnalysisReplacementWarningV1>,
}

pub(crate) struct ValidatedAnalysisReplacement {
    pub(crate) input_index: usize,
    pub(crate) byte_range: std::ops::Range<usize>,
    pub(crate) proposal: ResumeAnalysisProposedReplacementV1,
    pub(crate) action: ResumeAnalysisActionV1,
}
