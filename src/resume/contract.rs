use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

pub const INPUT_SCHEMA_VERSION: &str = "career.resume_input.v1";
pub const EVALUATION_SCHEMA_VERSION: &str = "career.resume_evaluation.v1";
pub const ERROR_SCHEMA_VERSION: &str = "career.error.v1";
pub const RESUME_SECTION_COVERAGE_POLICY_VERSION: &str = "resume_section_coverage_v1";

pub const MAX_RESUME_TEXT_CHARACTERS: usize = 50_000;
pub const MAX_RESUME_LINES: usize = 2_000;
pub const MAX_RESUME_LINE_CHARACTERS: usize = 2_000;
pub const MAX_DOCUMENT_ID_CHARACTERS: usize = 128;
pub const MAX_EVIDENCE_EXCERPT_CHARACTERS: usize = 120;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeInputMetadataV1 {
    pub document_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeInputV1 {
    pub schema_version: String,
    pub text: String,
    #[serde(default)]
    pub metadata: ResumeInputMetadataV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeSectionV1 {
    Summary,
    Experience,
    Education,
    Skills,
}

impl ResumeSectionV1 {
    pub const ALL: [Self; 4] = [
        Self::Summary,
        Self::Experience,
        Self::Education,
        Self::Skills,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Experience => "Experience",
            Self::Education => "Education",
            Self::Skills => "Skills",
        }
    }

    #[must_use]
    pub const fn check_id(self) -> &'static str {
        match self {
            Self::Summary => "resume.section.summary.detected",
            Self::Experience => "resume.section.experience.detected",
            Self::Education => "resume.section.education.detected",
            Self::Skills => "resume.section.skills.detected",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Summary => 0,
            Self::Experience => 1,
            Self::Education => 2,
            Self::Skills => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeDetectionStatusV1 {
    DetectedWithContent,
    DetectedWithoutContent,
    NotDetected,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeCheckCategoryV1 {
    SectionCoverage,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEvidenceKindV1 {
    RecognizedSectionHeader,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeEvidenceV1 {
    pub evidence_id: String,
    pub kind: ResumeEvidenceKindV1,
    pub section: ResumeSectionV1,
    pub line_number: usize,
    pub excerpt: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeSectionDetectionV1 {
    pub section: ResumeSectionV1,
    pub status: ResumeDetectionStatusV1,
    pub line_number: usize,
    pub header_excerpt: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeCheckV1 {
    pub check_id: String,
    pub category: ResumeCheckCategoryV1,
    pub section: ResumeSectionV1,
    pub status: ResumeDetectionStatusV1,
    pub score: u8,
    pub passed: bool,
    pub explanation: String,
    pub evidence: Vec<ResumeEvidenceV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeWarningCodeV1 {
    LimitedEvaluationScope,
    NoRecognizedSectionHeaders,
    ExpectedSectionHeadersNotDetected,
    DetectedSectionHeadersWithoutContent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeWarningV1 {
    pub code: ResumeWarningCodeV1,
    pub message: String,
    pub related_sections: Vec<ResumeSectionV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEvaluationScopeV1 {
    SectionCoverage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeEvaluationV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub core_version: String,
    pub document_id: Option<String>,
    pub scope: ResumeEvaluationScopeV1,
    pub score: u8,
    pub detected_sections: Vec<ResumeSectionDetectionV1>,
    pub checks: Vec<ResumeCheckV1>,
    pub warnings: Vec<ResumeWarningV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEvaluationErrorCodeV1 {
    UnsupportedSchemaVersion,
    SourceTextEmpty,
    SourceTextTooLarge,
    SourceLineCountExceeded,
    SourceLineTooLong,
    DocumentIdEmpty,
    DocumentIdTooLong,
    UnsupportedEnrichmentInputSchemaVersion,
    UnsupportedEnrichmentProposalSchemaVersion,
    EnrichmentNotEligible,
    EnrichmentProposalTooLarge,
    EnrichmentNonTargetPopulated,
    EnrichmentFieldEmpty,
    EnrichmentFieldTooLong,
    EnrichmentListTooLong,
    EnrichmentValueNotGrounded,
    EnrichmentEntryInvalid,
}

impl ResumeEvaluationErrorCodeV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion => "unsupported_schema_version",
            Self::SourceTextEmpty => "source_text_empty",
            Self::SourceTextTooLarge => "source_text_too_large",
            Self::SourceLineCountExceeded => "source_line_count_exceeded",
            Self::SourceLineTooLong => "source_line_too_long",
            Self::DocumentIdEmpty => "document_id_empty",
            Self::DocumentIdTooLong => "document_id_too_long",
            Self::UnsupportedEnrichmentInputSchemaVersion => {
                "unsupported_enrichment_input_schema_version"
            }
            Self::UnsupportedEnrichmentProposalSchemaVersion => {
                "unsupported_enrichment_proposal_schema_version"
            }
            Self::EnrichmentNotEligible => "enrichment_not_eligible",
            Self::EnrichmentProposalTooLarge => "enrichment_proposal_too_large",
            Self::EnrichmentNonTargetPopulated => "enrichment_non_target_populated",
            Self::EnrichmentFieldEmpty => "enrichment_field_empty",
            Self::EnrichmentFieldTooLong => "enrichment_field_too_long",
            Self::EnrichmentListTooLong => "enrichment_list_too_long",
            Self::EnrichmentValueNotGrounded => "enrichment_value_not_grounded",
            Self::EnrichmentEntryInvalid => "enrichment_entry_invalid",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeEvaluationErrorV1 {
    pub schema_version: String,
    pub code: ResumeEvaluationErrorCodeV1,
    pub message: String,
    pub field_path: String,
}

impl ResumeEvaluationErrorV1 {
    pub(crate) fn new(
        code: ResumeEvaluationErrorCodeV1,
        message: impl Into<String>,
        field_path: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: ERROR_SCHEMA_VERSION.to_owned(),
            code,
            message: message.into(),
            field_path: field_path.into(),
        }
    }
}

impl fmt::Display for ResumeEvaluationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for ResumeEvaluationErrorV1 {}

/// Domain-neutral alias retained alongside the original Phase 1 Rust type name.
pub type CareerErrorCodeV1 = ResumeEvaluationErrorCodeV1;

/// Domain-neutral alias for the shared `career.error.v1` failure contract.
pub type CareerErrorV1 = ResumeEvaluationErrorV1;

/// Resume-oriented compatibility alias for the shared error-code contract.
pub type ResumeErrorCodeV1 = CareerErrorCodeV1;

/// Resume-oriented compatibility alias for the shared error document.
pub type ResumeErrorV1 = CareerErrorV1;
