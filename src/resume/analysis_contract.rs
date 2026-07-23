use serde::{Deserialize, Serialize};

use super::normalization_contract::{
    NORMALIZATION_POLICY_VERSION, ResumeDeterministicFallbackV1, ResumeFieldDetectionStatusV1,
    ResumeFieldStatusV1, ResumeNormalizedFieldV1, ResumeParseConfidenceV1, ResumeSourceSpanV1,
};

pub const ANALYSIS_SCHEMA_VERSION: &str = "career.resume_analysis.v1";
pub const ANALYSIS_POLICY_VERSION: &str = "resume_analysis_v1";
pub const ANALYSIS_REFERENCE_POLICY_VERSION: &str = "deterministic_v2";
pub const ANALYSIS_NORMALIZATION_POLICY_VERSION: &str = NORMALIZATION_POLICY_VERSION;
pub const ANALYSIS_UNCERTAIN_MISSING_SCORE: u8 = 50;
pub const MAX_ANALYSIS_EVIDENCE_PER_CHECK: usize = 2;
pub const MAX_ANALYSIS_EVIDENCE_VALUE_CHARACTERS: usize = 240;
pub const MAX_ANALYSIS_FINDINGS_PER_KIND: usize = 3;
pub const MAX_ANALYSIS_ACTIONS: usize = 3;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisCategoryV1 {
    FormatAts,
    ContentStrength,
    ExperienceImpact,
    SkillsCoverage,
    Presentation,
    Completeness,
}

impl ResumeAnalysisCategoryV1 {
    pub const ALL: [Self; 6] = [
        Self::FormatAts,
        Self::ContentStrength,
        Self::ExperienceImpact,
        Self::SkillsCoverage,
        Self::Presentation,
        Self::Completeness,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FormatAts => "format_ats",
            Self::ContentStrength => "content_strength",
            Self::ExperienceImpact => "experience_impact",
            Self::SkillsCoverage => "skills_coverage",
            Self::Presentation => "presentation",
            Self::Completeness => "completeness",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisCategoryValuesV1 {
    pub format_ats: u8,
    pub content_strength: u8,
    pub experience_impact: u8,
    pub skills_coverage: u8,
    pub presentation: u8,
    pub completeness: u8,
}

impl ResumeAnalysisCategoryValuesV1 {
    #[must_use]
    pub const fn get(&self, category: ResumeAnalysisCategoryV1) -> u8 {
        match category {
            ResumeAnalysisCategoryV1::FormatAts => self.format_ats,
            ResumeAnalysisCategoryV1::ContentStrength => self.content_strength,
            ResumeAnalysisCategoryV1::ExperienceImpact => self.experience_impact,
            ResumeAnalysisCategoryV1::SkillsCoverage => self.skills_coverage,
            ResumeAnalysisCategoryV1::Presentation => self.presentation,
            ResumeAnalysisCategoryV1::Completeness => self.completeness,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisCheckIdV1 {
    ContactEmailPresent,
    ContactNamePresent,
    ContactPhoneOrLinkPresent,
    SummaryPresent,
    ExperienceSectionPresent,
    EducationSectionPresent,
    SkillsSectionPresent,
    ExperienceBulletsPresent,
    ExperienceBulletLengthQuality,
    ExperienceDateRangesPresent,
    ExperienceChronologyConsistency,
    MeasurableImpactInBullets,
    SkillsCountQuality,
    WeakPhrasingPenalty,
    KeywordRepetitionPenalty,
    AtsSectionHeaderClarity,
    AtsLineDensity,
    ParseQualityProxy,
}

impl ResumeAnalysisCheckIdV1 {
    pub const ALL: [Self; 18] = [
        Self::ContactEmailPresent,
        Self::ContactNamePresent,
        Self::ContactPhoneOrLinkPresent,
        Self::SummaryPresent,
        Self::ExperienceSectionPresent,
        Self::EducationSectionPresent,
        Self::SkillsSectionPresent,
        Self::ExperienceBulletsPresent,
        Self::ExperienceBulletLengthQuality,
        Self::ExperienceDateRangesPresent,
        Self::ExperienceChronologyConsistency,
        Self::MeasurableImpactInBullets,
        Self::SkillsCountQuality,
        Self::WeakPhrasingPenalty,
        Self::KeywordRepetitionPenalty,
        Self::AtsSectionHeaderClarity,
        Self::AtsLineDensity,
        Self::ParseQualityProxy,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContactEmailPresent => "contact_email_present",
            Self::ContactNamePresent => "contact_name_present",
            Self::ContactPhoneOrLinkPresent => "contact_phone_or_link_present",
            Self::SummaryPresent => "summary_present",
            Self::ExperienceSectionPresent => "experience_section_present",
            Self::EducationSectionPresent => "education_section_present",
            Self::SkillsSectionPresent => "skills_section_present",
            Self::ExperienceBulletsPresent => "experience_bullets_present",
            Self::ExperienceBulletLengthQuality => "experience_bullet_length_quality",
            Self::ExperienceDateRangesPresent => "experience_date_ranges_present",
            Self::ExperienceChronologyConsistency => "experience_chronology_consistency",
            Self::MeasurableImpactInBullets => "measurable_impact_in_bullets",
            Self::SkillsCountQuality => "skills_count_quality",
            Self::WeakPhrasingPenalty => "weak_phrasing_penalty",
            Self::KeywordRepetitionPenalty => "keyword_repetition_penalty",
            Self::AtsSectionHeaderClarity => "ats_section_header_clarity",
            Self::AtsLineDensity => "ats_line_density",
            Self::ParseQualityProxy => "parse_quality_proxy",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisCheckOutcomeV1 {
    Passed,
    Failed,
    Inconclusive,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisEvidenceKindV1 {
    NormalizedField,
    DerivedMetric,
    ParseConfidence,
    FieldStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisEvidenceV1 {
    pub evidence_id: String,
    pub kind: ResumeAnalysisEvidenceKindV1,
    pub source: Option<ResumeSourceSpanV1>,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisCheckV1 {
    pub check_id: ResumeAnalysisCheckIdV1,
    pub category: ResumeAnalysisCategoryV1,
    pub raw_score: u8,
    pub score: u8,
    pub score_adjusted: bool,
    pub passed: bool,
    pub outcome: ResumeAnalysisCheckOutcomeV1,
    pub detection_status: Option<ResumeFieldDetectionStatusV1>,
    pub explanation: String,
    pub evidence: Vec<ResumeAnalysisEvidenceV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisFindingStatusV1 {
    Confirmed,
    Provisional,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisFindingV1 {
    pub area: ResumeAnalysisCategoryV1,
    pub title: String,
    pub reason: String,
    pub basis_check_id: ResumeAnalysisCheckIdV1,
    pub status: ResumeAnalysisFindingStatusV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisActionV1 {
    pub priority: u8,
    pub area: ResumeAnalysisCategoryV1,
    pub action: String,
    pub basis_check_id: ResumeAnalysisCheckIdV1,
    pub status: ResumeAnalysisFindingStatusV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisScoreSourceV1 {
    DeterministicNormalizationBaseline,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisConfidenceContextV1 {
    pub score_source: ResumeAnalysisScoreSourceV1,
    pub parse_confidence: ResumeParseConfidenceV1,
    pub field_statuses: Vec<ResumeFieldStatusV1>,
    pub uncertain_fields: Vec<ResumeNormalizedFieldV1>,
    pub likely_missing_fields: Vec<ResumeNormalizedFieldV1>,
    pub fallbacks_applied: Vec<ResumeDeterministicFallbackV1>,
    pub uncertain_score_floor: u8,
    pub adjusted_check_ids: Vec<ResumeAnalysisCheckIdV1>,
    pub adjustment_applied: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeAnalysisWarningCodeV1 {
    GeneralAtsReadinessOnly,
    VisualLayoutNotEvaluated,
    ParseConfidenceProvisional,
    FieldsNotDetected,
    ConservativeFallbackApplied,
    NormalizationOutputTruncated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisWarningV1 {
    pub code: ResumeAnalysisWarningCodeV1,
    pub message: String,
    pub related_fields: Vec<ResumeNormalizedFieldV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeAnalysisV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub reference_policy_version: String,
    pub normalization_policy_version: String,
    pub core_version: String,
    pub document_id: Option<String>,
    pub scoring_weights: ResumeAnalysisCategoryValuesV1,
    pub overall_score: u8,
    pub category_scores: ResumeAnalysisCategoryValuesV1,
    pub summary: String,
    pub checks: Vec<ResumeAnalysisCheckV1>,
    pub confidence_context: ResumeAnalysisConfidenceContextV1,
    pub top_strengths: Vec<ResumeAnalysisFindingV1>,
    pub top_weaknesses: Vec<ResumeAnalysisFindingV1>,
    pub improvement_actions: Vec<ResumeAnalysisActionV1>,
    pub warnings: Vec<ResumeAnalysisWarningV1>,
}
