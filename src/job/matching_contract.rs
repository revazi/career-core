use serde::{Deserialize, Serialize};

use super::{JOB_NORMALIZATION_POLICY_VERSION, JobInputV1, JobParseConfidenceV1, JobSourceSpanV1};
use crate::{
    CareerErrorV1, NORMALIZATION_POLICY_VERSION, ResumeInputV1, ResumeParseConfidenceV1,
    ResumeSourceSpanV1,
};

pub const JOB_MATCH_INPUT_SCHEMA_VERSION: &str = "career.job_match_input.v1";
pub const JOB_MATCH_SCHEMA_VERSION: &str = "career.job_match.v1";
pub const JOB_MATCH_POLICY_VERSION: &str = "job_match_v1";
pub const JOB_MATCH_REFERENCE_POLICY_VERSION: &str = "job_match_deterministic_v2";
pub const SKILL_EQUIVALENCE_POLICY_VERSION: &str = "conservative_skill_equivalence_v1";
pub const JOB_MATCH_RECOMMENDATION_POLICY_VERSION: &str = "job_match_recommendation_v1";
pub const JOB_MATCH_RECOMMENDATION_REFERENCE_POLICY_VERSION: &str =
    "job_match_recommendation_safety_v1";
pub const JOB_MATCH_RESUME_NORMALIZATION_POLICY_VERSION: &str = NORMALIZATION_POLICY_VERSION;
pub const JOB_MATCH_JOB_NORMALIZATION_POLICY_VERSION: &str = JOB_NORMALIZATION_POLICY_VERSION;

pub const UNCERTAIN_MATCH_SCORE_FLOOR: u8 = 50;
pub const UNCERTAIN_MATCH_SCORE_CEILING: u8 = 75;
pub const MAX_JOB_MATCH_ITEMS_PER_CATEGORY: usize = 100;
pub const MAX_JOB_MATCH_METRICS_PER_CATEGORY: usize = 12;
pub const MAX_JOB_MATCH_STRENGTHS: usize = 5;
pub const MAX_JOB_MATCH_GAPS: usize = 5;
pub const MAX_JOB_MATCH_RECOMMENDATION_BLOCKERS: usize = 10;
pub const MAX_JOB_MATCH_METRIC_VALUE_CHARACTERS: usize = 240;

pub type JobMatchErrorV1 = CareerErrorV1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchInputV1 {
    pub schema_version: String,
    pub resume: ResumeInputV1,
    pub job: JobInputV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchCategoryV1 {
    SkillsMatch,
    ExperienceMatch,
    SeniorityFit,
    DomainFit,
    KeywordAlignment,
    EducationFit,
}

impl JobMatchCategoryV1 {
    pub const ALL: [Self; 6] = [
        Self::SkillsMatch,
        Self::ExperienceMatch,
        Self::SeniorityFit,
        Self::DomainFit,
        Self::KeywordAlignment,
        Self::EducationFit,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SkillsMatch => "skills_match",
            Self::ExperienceMatch => "experience_match",
            Self::SeniorityFit => "seniority_fit",
            Self::DomainFit => "domain_fit",
            Self::KeywordAlignment => "keyword_alignment",
            Self::EducationFit => "education_fit",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchCategoryValuesV1 {
    pub skills_match: u8,
    pub experience_match: u8,
    pub seniority_fit: u8,
    pub domain_fit: u8,
    pub keyword_alignment: u8,
    pub education_fit: u8,
}

impl JobMatchCategoryValuesV1 {
    #[must_use]
    pub const fn get(&self, category: JobMatchCategoryV1) -> u8 {
        match category {
            JobMatchCategoryV1::SkillsMatch => self.skills_match,
            JobMatchCategoryV1::ExperienceMatch => self.experience_match,
            JobMatchCategoryV1::SeniorityFit => self.seniority_fit,
            JobMatchCategoryV1::DomainFit => self.domain_fit,
            JobMatchCategoryV1::KeywordAlignment => self.keyword_alignment,
            JobMatchCategoryV1::EducationFit => self.education_fit,
        }
    }

    pub(crate) fn set(&mut self, category: JobMatchCategoryV1, value: u8) {
        match category {
            JobMatchCategoryV1::SkillsMatch => self.skills_match = value,
            JobMatchCategoryV1::ExperienceMatch => self.experience_match = value,
            JobMatchCategoryV1::SeniorityFit => self.seniority_fit = value,
            JobMatchCategoryV1::DomainFit => self.domain_fit = value,
            JobMatchCategoryV1::KeywordAlignment => self.keyword_alignment = value,
            JobMatchCategoryV1::EducationFit => self.education_fit = value,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchItemKindV1 {
    RequiredSkill,
    PreferredSkill,
    Keyword,
    Domain,
    Education,
    Certification,
    Experience,
    Seniority,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchItemStatusV1 {
    ConfirmedMatch,
    PartialMatch,
    LikelyMissing,
    Unverified,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchTypeV1 {
    NormalizedExact,
    ConservativeAlias,
    DerivedSignal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchCategoryItemV1 {
    pub kind: JobMatchItemKindV1,
    pub item: String,
    pub resume_item: Option<String>,
    pub status: JobMatchItemStatusV1,
    pub match_type: Option<JobMatchTypeV1>,
    pub job_source: Option<JobSourceSpanV1>,
    pub resume_source: Option<ResumeSourceSpanV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchMetricIdV1 {
    ResumeSkillCount,
    RequiredSkillCount,
    PreferredSkillCount,
    RequiredSkillMatches,
    PreferredSkillMatches,
    RequiredSkillRawScore,
    PreferredSkillRawScore,
    ExperienceEntryCount,
    DatedExperienceEntryCount,
    RequiredYears,
    EstimatedResumeYears,
    JobSenioritySignals,
    ResumeSenioritySignals,
    JobDomains,
    ResumeDomains,
    TargetKeywordCount,
    MatchedKeywordCount,
    MissingKeywordCount,
    ResumeDegrees,
    RequiredDegrees,
    ResumeCertifications,
    RequiredCertifications,
    UnassessedRequiredQualificationCount,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchMetricV1 {
    pub metric_id: JobMatchMetricIdV1,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchCategoryResultV1 {
    pub category: JobMatchCategoryV1,
    pub weight: u8,
    pub raw_score: u8,
    pub score: u8,
    pub score_adjusted: bool,
    pub explanation: String,
    pub items: Vec<JobMatchCategoryItemV1>,
    pub metrics: Vec<JobMatchMetricV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchScoreSourceV1 {
    DeterministicNormalizationBaselines,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchUncertaintySourceV1 {
    ResumeParseConfidence,
    JobParseConfidence,
    ResumeNormalizationTruncated,
    JobNormalizationTruncated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchMissingClaimStatusV1 {
    LikelyMissing,
    Unverified,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchConfidenceContextV1 {
    pub score_source: JobMatchScoreSourceV1,
    pub resume_parse_confidence: ResumeParseConfidenceV1,
    pub job_parse_confidence: JobParseConfidenceV1,
    pub resume_normalization_truncated: bool,
    pub job_normalization_truncated: bool,
    pub is_uncertain: bool,
    pub uncertainty_sources: Vec<JobMatchUncertaintySourceV1>,
    pub score_floor: u8,
    pub score_ceiling: u8,
    pub adjusted_categories: Vec<JobMatchCategoryV1>,
    pub adjustment_applied: bool,
    pub missing_claims_status: JobMatchMissingClaimStatusV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchFindingStatusV1 {
    Confirmed,
    Partial,
    LikelyMissing,
    Unverified,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchStrengthV1 {
    pub category: JobMatchCategoryV1,
    pub item: String,
    pub reason: String,
    pub status: JobMatchFindingStatusV1,
    pub match_type: JobMatchTypeV1,
    pub job_source: Option<JobSourceSpanV1>,
    pub resume_source: Option<ResumeSourceSpanV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchGapV1 {
    pub category: JobMatchCategoryV1,
    pub item: String,
    pub reason: String,
    pub status: JobMatchFindingStatusV1,
    pub job_source: Option<JobSourceSpanV1>,
    pub resume_source: Option<ResumeSourceSpanV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchRecommendationLabelV1 {
    ImproveFirst,
    ApplyAfterSmallEdits,
    ApplyNow,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchRecommendationStatusV1 {
    Deterministic,
    Provisional,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchRecommendationGateV1 {
    OverallScoreBelowApplyAfterThreshold,
    OverallScoreBelowApplyNowThreshold,
    NormalizationConfidenceUncertain,
    CoreCategoryBelowApplyAfterThreshold,
    CoreCategoryBelowApplyNowThreshold,
    DeterministicRequiredSkillGap,
    DeterministicCoreEvidenceGap,
    UnassessedRequiredQualification,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchCategoryScoreV1 {
    pub category: JobMatchCategoryV1,
    pub score: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchRecommendationThresholdsV1 {
    pub apply_now_minimum_overall_score: u8,
    pub apply_after_edits_minimum_overall_score: u8,
    pub apply_after_edits_minimum_core_category_score: u8,
    pub apply_now_minimum_core_category_score: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchRecommendationV1 {
    pub policy_version: String,
    pub label: JobMatchRecommendationLabelV1,
    pub status: JobMatchRecommendationStatusV1,
    pub reason: String,
    pub gate_reasons: Vec<JobMatchRecommendationGateV1>,
    pub thresholds: JobMatchRecommendationThresholdsV1,
    pub low_core_categories: Vec<JobMatchCategoryScoreV1>,
    pub blocking_required_skill_gaps: Vec<String>,
    pub blocking_core_evidence_gaps: Vec<String>,
    pub unassessed_required_qualifications: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobMatchWarningCodeV1 {
    DeterministicAlignmentOnly,
    SourceTextAndExplicitEvidenceOnly,
    NormalizationConfidenceProvisional,
    NormalizationOutputTruncated,
    UnassessedRequiredQualifications,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchWarningV1 {
    pub code: JobMatchWarningCodeV1,
    pub message: String,
    pub related_categories: Vec<JobMatchCategoryV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobMatchV1 {
    pub schema_version: String,
    pub policy_version: String,
    pub reference_policy_version: String,
    pub skill_equivalence_policy_version: String,
    pub recommendation_policy_version: String,
    pub recommendation_reference_policy_version: String,
    pub resume_normalization_policy_version: String,
    pub job_normalization_policy_version: String,
    pub core_version: String,
    pub resume_document_id: Option<String>,
    pub job_document_id: Option<String>,
    pub scoring_weights: JobMatchCategoryValuesV1,
    pub overall_score: u8,
    pub category_scores: JobMatchCategoryValuesV1,
    pub category_results: Vec<JobMatchCategoryResultV1>,
    pub confidence_context: JobMatchConfidenceContextV1,
    pub top_strengths: Vec<JobMatchStrengthV1>,
    pub top_gaps: Vec<JobMatchGapV1>,
    pub recommendation: JobMatchRecommendationV1,
    pub warnings: Vec<JobMatchWarningV1>,
}
