use serde::{Deserialize, Serialize};

use super::contract::ResumeErrorV1;

pub const NORMALIZATION_SCHEMA_VERSION: &str = "career.resume_normalization.v1";
pub const NORMALIZATION_POLICY_VERSION: &str = "resume_normalization_v1";
pub const ENRICHMENT_PROPOSAL_SCHEMA_VERSION: &str = "career.resume_enrichment_proposal.v1";

pub const MAX_NORMALIZED_SUMMARY_CHARACTERS: usize = 2_000;
pub const MAX_NORMALIZED_EXPERIENCE_ENTRIES: usize = 15;
pub const MAX_NORMALIZED_EDUCATION_ENTRIES: usize = 10;
pub const MAX_NORMALIZED_SKILLS: usize = 50;
pub const MAX_NORMALIZED_PROJECTS: usize = 15;
pub const MAX_NORMALIZED_CERTIFICATIONS: usize = 15;
pub const MAX_NORMALIZED_RAW_TEXT_CHARACTERS: usize = 4_000;
pub const MAX_NORMALIZED_FIELD_CHARACTERS: usize = 300;
pub const MAX_NORMALIZED_BULLETS_PER_EXPERIENCE: usize = 20;
pub const MAX_NORMALIZED_BULLET_CHARACTERS: usize = 1_000;
pub const MAX_NORMALIZED_LINKS: usize = 10;
pub const MAX_NORMALIZED_DESCRIPTION_CHARACTERS: usize = 2_000;
pub const MAX_SOURCE_EXCERPT_CHARACTERS: usize = 160;

pub type ResumeNormalizationErrorV1 = ResumeErrorV1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeNormalizationSectionV1 {
    Summary,
    Experience,
    Education,
    Skills,
    Projects,
    Certifications,
}

impl ResumeNormalizationSectionV1 {
    pub const ALL: [Self; 6] = [
        Self::Summary,
        Self::Experience,
        Self::Education,
        Self::Skills,
        Self::Projects,
        Self::Certifications,
    ];

    pub const EXPECTED: [Self; 4] = [
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
            Self::Projects => "Projects",
            Self::Certifications => "Certifications",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Summary => 0,
            Self::Experience => 1,
            Self::Education => 2,
            Self::Skills => 3,
            Self::Projects => 4,
            Self::Certifications => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeSourceTransformationV1 {
    Verbatim,
    WhitespaceJoined,
    DelimiterSplit,
    ConservativeFallback,
    ExternalSourceGroundedProposal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeSourceSpanV1 {
    pub start_line: usize,
    pub end_line: usize,
    pub excerpt: String,
    pub transformation: ResumeSourceTransformationV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeGroundedTextV1 {
    pub value: String,
    pub source: ResumeSourceSpanV1,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeContactV1 {
    pub name: Option<ResumeGroundedTextV1>,
    pub email: Option<ResumeGroundedTextV1>,
    pub phone: Option<ResumeGroundedTextV1>,
    pub links: Vec<ResumeGroundedTextV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeExperienceEntryV1 {
    pub raw_text: ResumeGroundedTextV1,
    pub job_title: Option<ResumeGroundedTextV1>,
    pub company: Option<ResumeGroundedTextV1>,
    pub date_range: Option<ResumeGroundedTextV1>,
    pub bullets: Vec<ResumeGroundedTextV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEducationEntryV1 {
    pub raw_text: ResumeGroundedTextV1,
    pub institution: Option<ResumeGroundedTextV1>,
    pub degree: Option<ResumeGroundedTextV1>,
    pub date_range: Option<ResumeGroundedTextV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeProjectEntryV1 {
    pub raw_text: ResumeGroundedTextV1,
    pub name: ResumeGroundedTextV1,
    pub description: Option<ResumeGroundedTextV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeCertificationEntryV1 {
    pub raw_text: ResumeGroundedTextV1,
    pub name: ResumeGroundedTextV1,
    pub issuer: Option<ResumeGroundedTextV1>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeNormalizedDocumentV1 {
    pub contact: ResumeContactV1,
    pub summary: Option<ResumeGroundedTextV1>,
    pub experience: Vec<ResumeExperienceEntryV1>,
    pub education: Vec<ResumeEducationEntryV1>,
    pub skills: Vec<ResumeGroundedTextV1>,
    pub projects: Vec<ResumeProjectEntryV1>,
    pub certifications: Vec<ResumeCertificationEntryV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeSectionContentStatusV1 {
    DetectedWithContent,
    DetectedWithoutContent,
    NotDetected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeNormalizationSectionMetadataV1 {
    pub section: ResumeNormalizationSectionV1,
    pub status: ResumeSectionContentStatusV1,
    pub header_source: Option<ResumeSourceSpanV1>,
    pub content_line_count: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeDeterministicFallbackV1 {
    ExperienceFromDateRanges,
    SkillsFromCommaSeparatedLines,
    EducationFromDegreePhrases,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeNormalizationMetadataV1 {
    pub sections: Vec<ResumeNormalizationSectionMetadataV1>,
    pub detected_sections: Vec<ResumeNormalizationSectionV1>,
    pub missing_expected_sections: Vec<ResumeNormalizationSectionV1>,
    pub fallbacks_applied: Vec<ResumeDeterministicFallbackV1>,
    pub output_truncated: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeParseConfidenceLabelV1 {
    Unknown,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeConfidenceSignalKindV1 {
    ExtractionQuality,
    SectionCoverage,
    ContactCompleteness,
    ExperienceCompleteness,
    SkillsCompleteness,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeConfidenceSignalV1 {
    pub signal: ResumeConfidenceSignalKindV1,
    pub score: u8,
    pub max_score: u8,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeParseConfidenceV1 {
    pub label: ResumeParseConfidenceLabelV1,
    pub score: u8,
    pub max_score: u8,
    pub signals: Vec<ResumeConfidenceSignalV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeNormalizedFieldV1 {
    Contact,
    Summary,
    Experience,
    Education,
    Skills,
    Projects,
    Certifications,
}

impl ResumeNormalizedFieldV1 {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Contact => "Contact",
            Self::Summary => "Summary",
            Self::Experience => "Experience",
            Self::Education => "Education",
            Self::Skills => "Skills",
            Self::Projects => "Projects",
            Self::Certifications => "Certifications",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeFieldDetectionStatusV1 {
    Detected,
    LikelyMissing,
    NotDetected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeFieldStatusV1 {
    pub field: ResumeNormalizedFieldV1,
    pub status: ResumeFieldDetectionStatusV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeNormalizationWarningCodeV1 {
    NoRecognizedSectionHeaders,
    ExpectedSectionHeadersNotDetected,
    DetectedSectionHeadersWithoutContent,
    ConservativeFallbackApplied,
    ParseConfidenceProvisional,
    OutputTruncated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeNormalizationWarningV1 {
    pub code: ResumeNormalizationWarningCodeV1,
    pub message: String,
    pub related_fields: Vec<ResumeNormalizedFieldV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEnrichmentSectionV1 {
    Summary,
    Experience,
    Education,
    Skills,
}

impl ResumeEnrichmentSectionV1 {
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
    pub const fn field(self) -> ResumeNormalizedFieldV1 {
        match self {
            Self::Summary => ResumeNormalizedFieldV1::Summary,
            Self::Experience => ResumeNormalizedFieldV1::Experience,
            Self::Education => ResumeNormalizedFieldV1::Education,
            Self::Skills => ResumeNormalizedFieldV1::Skills,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEnrichmentRequestStatusV1 {
    Eligible,
    NotEligible,
}

impl ResumeEnrichmentRequestStatusV1 {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::NotEligible => "not eligible",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEnrichmentRequestReasonV1 {
    LowConfidenceWithMissingCoreFields,
    DeterministicParseConfidenceNotLow,
    NoMissingCoreFields,
}

impl ResumeEnrichmentRequestReasonV1 {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LowConfidenceWithMissingCoreFields => "low confidence with missing core fields",
            Self::DeterministicParseConfidenceNotLow => "deterministic parse confidence is not low",
            Self::NoMissingCoreFields => "no missing core fields",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEnrichmentRequestV1 {
    pub status: ResumeEnrichmentRequestStatusV1,
    pub reason: ResumeEnrichmentRequestReasonV1,
    pub target_sections: Vec<ResumeEnrichmentSectionV1>,
    pub proposal_schema_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeNormalizationV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub core_version: String,
    pub document_id: Option<String>,
    pub deterministic_document: ResumeNormalizedDocumentV1,
    pub confidence: ResumeParseConfidenceV1,
    pub field_statuses: Vec<ResumeFieldStatusV1>,
    pub metadata: ResumeNormalizationMetadataV1,
    pub warnings: Vec<ResumeNormalizationWarningV1>,
    pub enrichment_request: ResumeEnrichmentRequestV1,
}
