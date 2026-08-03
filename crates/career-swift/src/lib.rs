#![doc = "Narrow UniFFI adapter for versioned career-core JSON operations."]
#![deny(unsafe_code)]

use std::error::Error;
use std::fmt;

use career_core::{
    CareerErrorV1, JobInputV1, JobMatchInputV1, ResumeAnalysisSuggestionReviewInputV1,
    ResumeEnrichmentInputV1, ResumeInputV1, ResumeVariantMaterializationInputV1,
    ResumeVariantReviewInputV1, analyze_resume, apply_resume_enrichment, capabilities,
    evaluate_resume, match_job, materialize_resume_variant, normalize_job, normalize_resume,
    review_resume_analysis_suggestions, review_resume_variant,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

const MAX_SINGLE_INPUT_BYTES: usize = 262_144;
const MAX_MATCH_INPUT_BYTES: usize = 1_048_576;

/// Bounded errors translated into generated Swift `Error` values.
#[derive(Debug, uniffi::Error)]
pub enum CareerSwiftError {
    InvalidJson {
        message: String,
    },
    InputTooLarge {
        actual_bytes: u64,
        maximum_bytes: u64,
    },
    InvalidInput {
        code: String,
        message: String,
        field_path: String,
    },
    OutputSerialization {
        message: String,
    },
}

impl fmt::Display for CareerSwiftError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { message }
            | Self::InvalidInput { message, .. }
            | Self::OutputSerialization { message } => formatter.write_str(message),
            Self::InputTooLarge {
                actual_bytes,
                maximum_bytes,
            } => write!(
                formatter,
                "Swift adapter input must contain at most {maximum_bytes} bytes; received {actual_bytes} bytes."
            ),
        }
    }
}

impl Error for CareerSwiftError {}

/// Returns canonical `career.capabilities.v1` JSON.
#[uniffi::export]
pub fn capabilities_json() -> Result<String, CareerSwiftError> {
    serialize_json(&capabilities())
}

/// Evaluates `career.resume_input.v1` and returns canonical evaluation JSON.
#[uniffi::export]
pub fn resume_evaluate_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input = parse_input::<ResumeInputV1>(
        &input_json,
        "career.resume_input.v1",
        MAX_SINGLE_INPUT_BYTES,
    )?;
    serialize_json(&evaluate_resume(&input).map_err(map_core_error)?)
}

/// Analyzes `career.resume_input.v1` and returns canonical analysis JSON.
#[uniffi::export]
pub fn resume_analyze_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input = parse_input::<ResumeInputV1>(
        &input_json,
        "career.resume_input.v1",
        MAX_SINGLE_INPUT_BYTES,
    )?;
    serialize_json(&analyze_resume(&input).map_err(map_core_error)?)
}

/// Reviews `career.resume_analysis_suggestion_review_input.v1` as canonical JSON.
#[uniffi::export]
pub fn resume_analysis_suggestions_review_json(
    input_json: String,
) -> Result<String, CareerSwiftError> {
    let input = parse_input::<ResumeAnalysisSuggestionReviewInputV1>(
        &input_json,
        "career.resume_analysis_suggestion_review_input.v1",
        MAX_SINGLE_INPUT_BYTES,
    )?;
    serialize_json(&review_resume_analysis_suggestions(&input).map_err(map_core_error)?)
}

/// Normalizes `career.resume_input.v1` and returns canonical normalization JSON.
#[uniffi::export]
pub fn resume_normalize_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input = parse_input::<ResumeInputV1>(
        &input_json,
        "career.resume_input.v1",
        MAX_SINGLE_INPUT_BYTES,
    )?;
    serialize_json(&normalize_resume(&input).map_err(map_core_error)?)
}

/// Validates `career.resume_enrichment_input.v1` and returns canonical result JSON.
#[uniffi::export]
pub fn resume_enrich_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input = parse_input::<ResumeEnrichmentInputV1>(
        &input_json,
        "career.resume_enrichment_input.v1",
        MAX_SINGLE_INPUT_BYTES,
    )?;
    serialize_json(&apply_resume_enrichment(&input).map_err(map_core_error)?)
}

/// Reviews `career.resume_variant_review_input.v1` and returns canonical review JSON.
#[uniffi::export]
pub fn resume_variant_review_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input = parse_input::<ResumeVariantReviewInputV1>(
        &input_json,
        "career.resume_variant_review_input.v1",
        MAX_MATCH_INPUT_BYTES,
    )?;
    serialize_json(&review_resume_variant(&input).map_err(map_core_error)?)
}

/// Materializes `career.resume_variant_materialization_input.v1` as canonical JSON.
#[uniffi::export]
pub fn resume_variant_materialize_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input = parse_input::<ResumeVariantMaterializationInputV1>(
        &input_json,
        "career.resume_variant_materialization_input.v1",
        MAX_MATCH_INPUT_BYTES,
    )?;
    serialize_json(&materialize_resume_variant(&input).map_err(map_core_error)?)
}

/// Normalizes `career.job_input.v1` and returns canonical normalization JSON.
#[uniffi::export]
pub fn job_normalize_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input =
        parse_input::<JobInputV1>(&input_json, "career.job_input.v1", MAX_SINGLE_INPUT_BYTES)?;
    serialize_json(&normalize_job(&input).map_err(map_core_error)?)
}

/// Matches `career.job_match_input.v1` and returns canonical match JSON.
#[uniffi::export]
pub fn job_match_json(input_json: String) -> Result<String, CareerSwiftError> {
    let input = parse_input::<JobMatchInputV1>(
        &input_json,
        "career.job_match_input.v1",
        MAX_MATCH_INPUT_BYTES,
    )?;
    serialize_json(&match_job(&input).map_err(map_core_error)?)
}

fn parse_input<T: DeserializeOwned>(
    input_json: &str,
    contract: &str,
    maximum_bytes: usize,
) -> Result<T, CareerSwiftError> {
    if input_json.len() > maximum_bytes {
        return Err(CareerSwiftError::InputTooLarge {
            actual_bytes: usize_to_u64(input_json.len()),
            maximum_bytes: usize_to_u64(maximum_bytes),
        });
    }

    serde_json::from_str(input_json).map_err(|error| CareerSwiftError::InvalidJson {
        message: format!(
            "Input must be valid {contract} JSON (line {}, column {}).",
            error.line(),
            error.column()
        ),
    })
}

fn serialize_json(document: &impl Serialize) -> Result<String, CareerSwiftError> {
    let mut output = serde_json::to_string_pretty(document).map_err(|_| {
        CareerSwiftError::OutputSerialization {
            message: "Could not serialize Swift adapter output.".to_owned(),
        }
    })?;
    output.push('\n');
    Ok(output)
}

fn map_core_error(error: CareerErrorV1) -> CareerSwiftError {
    CareerSwiftError::InvalidInput {
        code: error.code.as_str().to_owned(),
        message: error.message,
        field_path: error.field_path,
    }
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).map_or(u64::MAX, |converted| converted)
}

uniffi::setup_scaffolding!("career_swift");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_operations_match_canonical_cli_goldens() {
        for (input, expected, operation) in [
            (
                include_str!("../../../fixtures/resume/phase1/complete-sections.input.json"),
                include_str!("../../../fixtures/resume/phase1/complete-sections.expected.json"),
                resume_evaluate_json as fn(String) -> Result<String, CareerSwiftError>,
            ),
            (
                include_str!("../../../fixtures/resume/phase3/complete-analysis.input.json"),
                include_str!("../../../fixtures/resume/phase3/complete-analysis.expected.json"),
                resume_analyze_json,
            ),
            (
                include_str!(
                    "../../../fixtures/resume/phase7/complete-analysis-suggestion-review.input.json"
                ),
                include_str!(
                    "../../../fixtures/resume/phase7/complete-analysis-suggestion-review.expected.json"
                ),
                resume_analysis_suggestions_review_json,
            ),
            (
                include_str!("../../../fixtures/resume/phase2/complete-normalization.input.json"),
                include_str!(
                    "../../../fixtures/resume/phase2/complete-normalization.expected.json"
                ),
                resume_normalize_json,
            ),
            (
                include_str!(
                    "../../../fixtures/resume/phase2/messy-unlabeled.enrichment-input.json"
                ),
                include_str!(
                    "../../../fixtures/resume/phase2/messy-unlabeled.enrichment-expected.json"
                ),
                resume_enrich_json,
            ),
            (
                include_str!("../../../fixtures/resume/phase7/complete-variant-review.input.json"),
                include_str!(
                    "../../../fixtures/resume/phase7/complete-variant-review.expected.json"
                ),
                resume_variant_review_json,
            ),
            (
                include_str!(
                    "../../../fixtures/resume/phase7/selected-variant-materialization.input.json"
                ),
                include_str!(
                    "../../../fixtures/resume/phase7/selected-variant-materialization.expected.json"
                ),
                resume_variant_materialize_json,
            ),
            (
                include_str!("../../../fixtures/job/phase4a/complete-normalization.input.json"),
                include_str!("../../../fixtures/job/phase4a/complete-normalization.expected.json"),
                job_normalize_json,
            ),
            (
                include_str!("../../../fixtures/job/phase4b/complete-match.input.json"),
                include_str!("../../../fixtures/job/phase4b/complete-match.expected.json"),
                job_match_json,
            ),
        ] {
            let actual = operation(input.to_owned()).expect("reviewed input should succeed");
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn capabilities_are_canonical_and_errors_are_bounded() {
        let capabilities = capabilities_json().expect("capabilities should serialize");
        let parsed: serde_json::Value =
            serde_json::from_str(&capabilities).expect("capabilities should be JSON");
        assert_eq!(parsed["schema_version"], "career.capabilities.v1");
        assert!(capabilities.ends_with('\n'));

        let malformed = resume_evaluate_json(
            r#"{"schema_version":"career.resume_input.v1","text":}"#.to_owned(),
        )
        .expect_err("malformed input should fail");
        let CareerSwiftError::InvalidJson { message } = malformed else {
            panic!("malformed input should return InvalidJson");
        };
        assert!(message.contains("career.resume_input.v1"));
        assert!(!message.contains("resume payload"));

        let secret = "private resume payload";
        let invalid = resume_evaluate_json(format!(
            r#"{{"schema_version":"career.resume_input.v1","text":"{}"}}"#,
            secret.repeat(3_000)
        ))
        .expect_err("oversized core input should fail");
        let CareerSwiftError::InvalidInput {
            code,
            message,
            field_path,
        } = invalid
        else {
            panic!("core rejection should return InvalidInput");
        };
        assert_eq!(code, "source_text_too_large");
        assert_eq!(field_path, "text");
        assert!(!message.contains(secret));

        let oversized = "x".repeat(MAX_SINGLE_INPUT_BYTES + 1);
        let error = resume_evaluate_json(oversized).expect_err("adapter limit should fail");
        assert!(matches!(
            error,
            CareerSwiftError::InputTooLarge {
                actual_bytes: 262_145,
                maximum_bytes: 262_144,
            }
        ));

        let oversized_match = "x".repeat(MAX_MATCH_INPUT_BYTES + 1);
        let match_error =
            job_match_json(oversized_match).expect_err("match adapter limit should fail");
        assert!(matches!(
            match_error,
            CareerSwiftError::InputTooLarge {
                actual_bytes: 1_048_577,
                maximum_bytes: 1_048_576,
            }
        ));
    }
}
