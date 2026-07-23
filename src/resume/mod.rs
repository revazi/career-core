mod contract;
mod evaluation;

pub use contract::{
    ERROR_SCHEMA_VERSION, EVALUATION_SCHEMA_VERSION, INPUT_SCHEMA_VERSION,
    MAX_DOCUMENT_ID_CHARACTERS, MAX_EVIDENCE_EXCERPT_CHARACTERS, MAX_RESUME_LINE_CHARACTERS,
    MAX_RESUME_LINES, MAX_RESUME_TEXT_CHARACTERS, RESUME_SECTION_COVERAGE_POLICY_VERSION,
    ResumeCheckCategoryV1, ResumeCheckV1, ResumeDetectionStatusV1, ResumeEvaluationErrorCodeV1,
    ResumeEvaluationErrorV1, ResumeEvaluationScopeV1, ResumeEvaluationV1, ResumeEvidenceKindV1,
    ResumeEvidenceV1, ResumeInputMetadataV1, ResumeInputV1, ResumeSectionDetectionV1,
    ResumeSectionV1, ResumeWarningCodeV1, ResumeWarningV1,
};
pub use evaluation::evaluate_resume;
