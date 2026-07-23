use serde::{Deserialize, Serialize};

use crate::CareerErrorV1;

pub const JOB_INPUT_SCHEMA_VERSION: &str = "career.job_input.v1";
pub const JOB_NORMALIZATION_SCHEMA_VERSION: &str = "career.job_normalization.v1";
pub const JOB_NORMALIZATION_POLICY_VERSION: &str = "job_normalization_v1";
pub const JOB_NORMALIZATION_REFERENCE_POLICY_VERSION: &str = "job_description_normalization_v6";

pub const MAX_JOB_TEXT_CHARACTERS: usize = 50_000;
pub const MAX_JOB_LINES: usize = 2_000;
pub const MAX_JOB_LINE_CHARACTERS: usize = 2_000;
pub const MAX_JOB_DOCUMENT_ID_CHARACTERS: usize = 128;
pub const MAX_JOB_SOURCE_EXCERPT_CHARACTERS: usize = 160;
pub const MAX_JOB_SKILLS_PER_GROUP: usize = 50;
pub const MAX_JOB_QUALIFICATIONS_PER_GROUP: usize = 50;
pub const MAX_JOB_REQUIREMENTS_PER_GROUP: usize = 50;
pub const MAX_JOB_RESPONSIBILITIES: usize = 50;
pub const MAX_JOB_SENIORITY_SIGNALS: usize = 20;
pub const MAX_JOB_UNMATCHED_LINES: usize = 10;

pub type JobNormalizationErrorV1 = CareerErrorV1;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobInputMetadataV1 {
    pub document_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobInputV1 {
    pub schema_version: String,
    pub text: String,
    #[serde(default)]
    pub metadata: JobInputMetadataV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobSourceTransformationV1 {
    Verbatim,
    ListItemCleaned,
    LabelPrefixRemoved,
    DelimiterSplit,
    NormalizedCase,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobSourceSpanV1 {
    pub start_line: usize,
    pub end_line: usize,
    pub excerpt: String,
    pub transformation: JobSourceTransformationV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobGroundedTextV1 {
    pub value: String,
    pub source: JobSourceSpanV1,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobNormalizedDocumentV1 {
    pub title: Option<JobGroundedTextV1>,
    pub company: Option<JobGroundedTextV1>,
    pub required_skills: Vec<JobGroundedTextV1>,
    pub preferred_skills: Vec<JobGroundedTextV1>,
    pub required_qualifications: Vec<JobGroundedTextV1>,
    pub preferred_qualifications: Vec<JobGroundedTextV1>,
    pub seniority_signals: Vec<JobGroundedTextV1>,
    pub experience_requirements: Vec<JobGroundedTextV1>,
    pub education_requirements: Vec<JobGroundedTextV1>,
    pub certification_requirements: Vec<JobGroundedTextV1>,
    pub responsibilities: Vec<JobGroundedTextV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobNormalizationSectionV1 {
    Required,
    Preferred,
    Responsibilities,
    Other,
}

impl JobNormalizationSectionV1 {
    pub const ALL: [Self; 4] = [
        Self::Required,
        Self::Preferred,
        Self::Responsibilities,
        Self::Other,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Required => "Required",
            Self::Preferred => "Preferred",
            Self::Responsibilities => "Responsibilities",
            Self::Other => "Other",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchedSectionV1 {
    pub section: JobNormalizationSectionV1,
    pub header_source: JobSourceSpanV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobUnmatchedLineV1 {
    pub line_number: usize,
    pub excerpt: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobNormalizationMetadataV1 {
    pub matched_sections: Vec<JobMatchedSectionV1>,
    pub unmatched_lines: Vec<JobUnmatchedLineV1>,
    pub unmatched_line_count: usize,
    pub unmatched_lines_truncated: bool,
    pub output_truncated: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobParseConfidenceLabelV1 {
    Unknown,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobConfidenceSignalKindV1 {
    TextQuality,
    TitleDetection,
    SkillsDetection,
    ResponsibilitiesDetection,
    RequirementDetection,
    StructureQuality,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobConfidenceSignalV1 {
    pub signal: JobConfidenceSignalKindV1,
    pub score: u8,
    pub max_score: u8,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobParseConfidenceV1 {
    pub label: JobParseConfidenceLabelV1,
    pub score: u8,
    pub max_score: u8,
    pub signals: Vec<JobConfidenceSignalV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobNormalizedFieldV1 {
    Title,
    Company,
    RequiredSkills,
    PreferredSkills,
    RequiredQualifications,
    PreferredQualifications,
    SenioritySignals,
    ExperienceRequirements,
    EducationRequirements,
    CertificationRequirements,
    Responsibilities,
}

impl JobNormalizedFieldV1 {
    pub const ALL: [Self; 11] = [
        Self::Title,
        Self::Company,
        Self::RequiredSkills,
        Self::PreferredSkills,
        Self::RequiredQualifications,
        Self::PreferredQualifications,
        Self::SenioritySignals,
        Self::ExperienceRequirements,
        Self::EducationRequirements,
        Self::CertificationRequirements,
        Self::Responsibilities,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Company => "Company",
            Self::RequiredSkills => "Required skills",
            Self::PreferredSkills => "Preferred skills",
            Self::RequiredQualifications => "Required qualifications",
            Self::PreferredQualifications => "Preferred qualifications",
            Self::SenioritySignals => "Seniority signals",
            Self::ExperienceRequirements => "Experience requirements",
            Self::EducationRequirements => "Education requirements",
            Self::CertificationRequirements => "Certification requirements",
            Self::Responsibilities => "Responsibilities",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobFieldDetectionStatusV1 {
    Detected,
    NotDetected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobFieldStatusV1 {
    pub field: JobNormalizedFieldV1,
    pub status: JobFieldDetectionStatusV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobNormalizationWarningCodeV1 {
    LimitedNormalizationScope,
    NoRecognizedSectionHeaders,
    UnclassifiedLinesPresent,
    ParseConfidenceProvisional,
    OutputTruncated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobNormalizationWarningV1 {
    pub code: JobNormalizationWarningCodeV1,
    pub message: String,
    pub related_fields: Vec<JobNormalizedFieldV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobNormalizationV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub reference_policy_version: String,
    pub core_version: String,
    pub document_id: Option<String>,
    pub deterministic_document: JobNormalizedDocumentV1,
    pub confidence: JobParseConfidenceV1,
    pub field_statuses: Vec<JobFieldStatusV1>,
    pub metadata: JobNormalizationMetadataV1,
    pub warnings: Vec<JobNormalizationWarningV1>,
}
