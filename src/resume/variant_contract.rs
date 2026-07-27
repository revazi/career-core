use serde::{Deserialize, Serialize};

use crate::JobInputV1;

use super::contract::{ResumeErrorV1, ResumeInputV1};

pub const RESUME_VARIANT_PROPOSAL_SCHEMA_VERSION: &str = "career.resume_variant_proposal.v1";
pub const RESUME_VARIANT_REVIEW_INPUT_SCHEMA_VERSION: &str =
    "career.resume_variant_review_input.v1";
pub const RESUME_VARIANT_REVIEW_SCHEMA_VERSION: &str = "career.resume_variant_review.v1";
pub const RESUME_VARIANT_MATERIALIZATION_INPUT_SCHEMA_VERSION: &str =
    "career.resume_variant_materialization_input.v1";
pub const RESUME_VARIANT_SCHEMA_VERSION: &str = "career.resume_variant.v1";
pub const RESUME_VARIANT_POLICY_VERSION: &str = "resume_variant_review_v1";

pub const MAX_RESUME_VARIANT_CHANGES: usize = 50;
pub const MAX_RESUME_VARIANT_PROPOSAL_CHARACTERS: usize = 100_000;
pub const MAX_RESUME_VARIANT_CHANGE_TEXT_CHARACTERS: usize = 10_000;
pub const MAX_RESUME_VARIANT_EVIDENCE_ITEMS: usize = 5;
pub const MAX_RESUME_VARIANT_EVIDENCE_CHARACTERS: usize = 300;

pub type ResumeVariantErrorV1 = ResumeErrorV1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeVariantSectionV1 {
    Contact,
    Summary,
    Experience,
    Education,
    Skills,
    Projects,
    Certifications,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantProposedChangeV1 {
    pub section: ResumeVariantSectionV1,
    pub start_line: usize,
    pub end_line: usize,
    pub original_text: String,
    pub proposed_text: String,
    pub resume_evidence: Vec<String>,
    pub vacancy_evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantProposalV1 {
    pub schema_version: String,
    pub changes: Vec<ResumeVariantProposedChangeV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantReviewInputV1 {
    pub schema_version: String,
    pub resume: ResumeInputV1,
    pub vacancy: JobInputV1,
    pub proposal: ResumeVariantProposalV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeVariantDiscardCodeV1 {
    InvalidLineRange,
    TargetMismatch,
    NoChange,
    ChangeTextTooLong,
    UnsupportedControlCharacter,
    InvalidResumeEvidence,
    InvalidVacancyEvidence,
    DuplicateOrOverlappingTarget,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantDiscardedChangeV1 {
    pub input_index: usize,
    pub code: ResumeVariantDiscardCodeV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantCanonicalChangeV1 {
    pub change_id: String,
    pub section: ResumeVariantSectionV1,
    pub start_line: usize,
    pub end_line: usize,
    pub original_text: String,
    pub proposed_text: String,
    pub resume_evidence: Vec<String>,
    pub vacancy_evidence: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeVariantAuthorityV1 {
    AssistedNonAuthoritative,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeVariantWarningCodeV1 {
    AssistedContentNonAuthoritative,
    EvidenceOccurrenceNotFactualCertification,
    DeterministicBaselinePreserved,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantWarningV1 {
    pub code: ResumeVariantWarningCodeV1,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantReviewV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub core_version: String,
    pub authority: ResumeVariantAuthorityV1,
    pub baseline_resume: ResumeInputV1,
    pub proposed_preview_text: String,
    pub changes: Vec<ResumeVariantCanonicalChangeV1>,
    pub discarded_changes: Vec<ResumeVariantDiscardedChangeV1>,
    pub warnings: Vec<ResumeVariantWarningV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantMaterializationInputV1 {
    pub schema_version: String,
    pub expected_review_policy_version: String,
    pub review_input: ResumeVariantReviewInputV1,
    pub selected_change_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeVariantV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub core_version: String,
    pub authority: ResumeVariantAuthorityV1,
    pub baseline_resume: ResumeInputV1,
    pub assisted_resume_text: String,
    pub selected_changes: Vec<ResumeVariantCanonicalChangeV1>,
    pub warnings: Vec<ResumeVariantWarningV1>,
}
