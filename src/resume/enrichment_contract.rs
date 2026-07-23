use serde::{Deserialize, Serialize};

use super::contract::{ResumeErrorV1, ResumeInputV1};
use super::normalization_contract::{
    ResumeEnrichmentSectionV1, ResumeNormalizationV1, ResumeNormalizedDocumentV1,
    ResumeNormalizedFieldV1,
};

pub const ENRICHMENT_INPUT_SCHEMA_VERSION: &str = "career.resume_enrichment_input.v1";
pub const ENRICHMENT_RESULT_SCHEMA_VERSION: &str = "career.resume_enrichment_result.v1";
pub const ENRICHMENT_POLICY_VERSION: &str = "resume_normalization_enrichment_v1";
pub const MAX_ENRICHMENT_PROPOSAL_CHARACTERS: usize = 50_000;

pub type ResumeEnrichmentErrorV1 = ResumeErrorV1;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentExperienceProposalV1 {
    pub raw_text: String,
    pub job_title: String,
    pub company: String,
    pub date_range: String,
    pub bullets: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentEducationProposalV1 {
    pub raw_text: String,
    pub institution: String,
    pub degree: String,
    pub date_range: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentProposalV1 {
    pub schema_version: String,
    pub summary: String,
    pub experience: Vec<ResumeEnrichmentExperienceProposalV1>,
    pub education: Vec<ResumeEnrichmentEducationProposalV1>,
    pub skills: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentInputV1 {
    pub schema_version: String,
    pub resume: ResumeInputV1,
    pub proposal: ResumeEnrichmentProposalV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEnrichmentMergeStatusV1 {
    Applied,
    NotApplied,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEnrichmentFieldSourceV1 {
    Deterministic,
    ExternalSourceGroundedProposal,
    NotAvailable,
}

impl ResumeEnrichmentFieldSourceV1 {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::ExternalSourceGroundedProposal => "external source-grounded proposal",
            Self::NotAvailable => "not available",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentFieldProvenanceV1 {
    pub field: ResumeNormalizedFieldV1,
    pub source: ResumeEnrichmentFieldSourceV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentMergeV1 {
    pub status: ResumeEnrichmentMergeStatusV1,
    pub target_sections: Vec<ResumeEnrichmentSectionV1>,
    pub applied_fields: Vec<ResumeNormalizedFieldV1>,
    pub field_provenance: Vec<ResumeEnrichmentFieldProvenanceV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEnrichmentWarningCodeV1 {
    AssistedFieldsNonAuthoritative,
    DeterministicConfidencePreserved,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentWarningV1 {
    pub code: ResumeEnrichmentWarningCodeV1,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentResultV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub core_version: String,
    pub baseline: ResumeNormalizationV1,
    pub assisted_document: ResumeNormalizedDocumentV1,
    pub merge: ResumeEnrichmentMergeV1,
    pub warnings: Vec<ResumeEnrichmentWarningV1>,
}
