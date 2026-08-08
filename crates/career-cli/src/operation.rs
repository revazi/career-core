use career_core::capabilities;
use serde::Serialize;

pub(crate) const OPERATION_CATALOG_VERSION: &str = "career.operation_catalog.v1";
pub(crate) const MAX_CLI_INPUT_BYTES: usize = 262_144;
pub(crate) const MAX_CLI_COMPOSITE_INPUT_BYTES: usize = 1_048_576;
pub(crate) const MAX_SUCCESSFUL_MACHINE_OUTPUT_BYTES: usize = 33_554_432;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OperationAvailabilityV1 {
    Available,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InputTransportV1 {
    None,
    CliArguments,
    JsonFileOrStdin,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct OperationCatalogV1 {
    pub(crate) schema_version: &'static str,
    pub(crate) core_version: String,
    pub(crate) operations: Vec<OperationDescriptorV1>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct OperationDescriptorV1 {
    pub(crate) operation_id: &'static str,
    pub(crate) capability_id: Option<&'static str>,
    pub(crate) availability: OperationAvailabilityV1,
    pub(crate) cli_path: Vec<&'static str>,
    pub(crate) input_transport: InputTransportV1,
    pub(crate) input_schema_id: Option<&'static str>,
    pub(crate) output_schema_id: &'static str,
    pub(crate) maximum_input_bytes: Option<usize>,
    pub(crate) maximum_successful_machine_output_bytes: usize,
}

pub(crate) fn operation_catalog() -> OperationCatalogV1 {
    OperationCatalogV1 {
        schema_version: OPERATION_CATALOG_VERSION,
        core_version: capabilities().core_version,
        operations: vec![
            descriptor(
                "core.capabilities",
                &["capabilities"],
                InputTransportV1::None,
                None,
                "career.capabilities.v1",
                None,
            ),
            bootstrap_descriptor(
                "core.operations",
                &["operations"],
                InputTransportV1::None,
                "career.operation_catalog.v1",
            ),
            bootstrap_descriptor(
                "schema.list",
                &["schema", "list"],
                InputTransportV1::None,
                "career.schema_catalog.v1",
            ),
            bootstrap_descriptor(
                "schema.export",
                &["schema", "export"],
                InputTransportV1::CliArguments,
                "https://json-schema.org/draft/2020-12/schema",
            ),
            bootstrap_descriptor(
                "schema.bundle",
                &["schema", "bundle"],
                InputTransportV1::CliArguments,
                "https://json-schema.org/draft/2020-12/schema",
            ),
            descriptor(
                "resume.evaluate",
                &["resume", "evaluate"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_input.v1"),
                "career.resume_evaluation.v1",
                Some(MAX_CLI_INPUT_BYTES),
            ),
            descriptor(
                "resume.analyze",
                &["resume", "analyze"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_input.v1"),
                "career.resume_analysis.v1",
                Some(MAX_CLI_INPUT_BYTES),
            ),
            descriptor(
                "resume.normalize",
                &["resume", "normalize"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_input.v1"),
                "career.resume_normalization.v1",
                Some(MAX_CLI_INPUT_BYTES),
            ),
            descriptor(
                "resume.enrich",
                &["resume", "enrich"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_enrichment_input.v1"),
                "career.resume_enrichment_result.v1",
                Some(MAX_CLI_INPUT_BYTES),
            ),
            descriptor(
                "resume.analysis-suggestions.review",
                &["resume", "analysis-suggestions-review"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_analysis_suggestion_review_input.v1"),
                "career.resume_analysis_suggestion_review.v1",
                Some(MAX_CLI_INPUT_BYTES),
            ),
            descriptor(
                "resume.analysis-replacements.review",
                &["resume", "analysis-replacements-review"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_analysis_replacement_review_input.v1"),
                "career.resume_analysis_replacement_review.v1",
                Some(MAX_CLI_INPUT_BYTES),
            ),
            descriptor(
                "resume.variant.review",
                &["resume", "variant-review"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_variant_review_input.v1"),
                "career.resume_variant_review.v1",
                Some(MAX_CLI_COMPOSITE_INPUT_BYTES),
            ),
            descriptor(
                "resume.variant.materialize",
                &["resume", "variant-materialize"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.resume_variant_materialization_input.v1"),
                "career.resume_variant.v1",
                Some(MAX_CLI_COMPOSITE_INPUT_BYTES),
            ),
            descriptor(
                "job.normalize",
                &["job", "normalize"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.job_input.v1"),
                "career.job_normalization.v1",
                Some(MAX_CLI_INPUT_BYTES),
            ),
            descriptor(
                "job.match",
                &["job", "match"],
                InputTransportV1::JsonFileOrStdin,
                Some("career.job_match_input.v1"),
                "career.job_match.v1",
                Some(MAX_CLI_COMPOSITE_INPUT_BYTES),
            ),
        ],
    }
}

fn descriptor(
    operation_id: &'static str,
    cli_path: &[&'static str],
    input_transport: InputTransportV1,
    input_schema_id: Option<&'static str>,
    output_schema_id: &'static str,
    maximum_input_bytes: Option<usize>,
) -> OperationDescriptorV1 {
    OperationDescriptorV1 {
        operation_id,
        capability_id: Some(operation_id),
        availability: OperationAvailabilityV1::Available,
        cli_path: cli_path.to_vec(),
        input_transport,
        input_schema_id,
        output_schema_id,
        maximum_input_bytes,
        maximum_successful_machine_output_bytes: MAX_SUCCESSFUL_MACHINE_OUTPUT_BYTES,
    }
}

fn bootstrap_descriptor(
    operation_id: &'static str,
    cli_path: &[&'static str],
    input_transport: InputTransportV1,
    output_schema_id: &'static str,
) -> OperationDescriptorV1 {
    OperationDescriptorV1 {
        operation_id,
        capability_id: None,
        availability: OperationAvailabilityV1::Available,
        cli_path: cli_path.to_vec(),
        input_transport,
        input_schema_id: None,
        output_schema_id,
        maximum_input_bytes: None,
        maximum_successful_machine_output_bytes: MAX_SUCCESSFUL_MACHINE_OUTPUT_BYTES,
    }
}

#[cfg(test)]
mod tests {
    use career_core::CapabilityStatus;

    use super::*;

    #[test]
    fn catalog_is_deterministic_and_completely_maps_available_capabilities() {
        assert_eq!(operation_catalog(), operation_catalog());

        let available_capability_ids = capabilities()
            .capabilities
            .into_iter()
            .filter(|capability| capability.status == CapabilityStatus::Available)
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        let catalog = operation_catalog();
        let mapped_capability_ids = catalog
            .operations
            .iter()
            .filter_map(|operation| operation.capability_id)
            .collect::<Vec<_>>();
        let capability_backed_operation_ids = catalog
            .operations
            .iter()
            .filter(|operation| operation.capability_id.is_some())
            .map(|operation| operation.operation_id)
            .collect::<Vec<_>>();

        assert_eq!(mapped_capability_ids, available_capability_ids);
        assert_eq!(capability_backed_operation_ids, mapped_capability_ids);
    }

    #[test]
    fn every_input_operation_declares_exact_transport_schema_and_bounds() {
        for operation in operation_catalog().operations {
            assert_eq!(
                operation.maximum_successful_machine_output_bytes,
                MAX_SUCCESSFUL_MACHINE_OUTPUT_BYTES
            );
            match operation.input_transport {
                InputTransportV1::None | InputTransportV1::CliArguments => {
                    assert_eq!(operation.input_schema_id, None);
                    assert_eq!(operation.maximum_input_bytes, None);
                }
                InputTransportV1::JsonFileOrStdin => {
                    assert!(operation.input_schema_id.is_some());
                    assert!(matches!(
                        operation.maximum_input_bytes,
                        Some(MAX_CLI_INPUT_BYTES | MAX_CLI_COMPOSITE_INPUT_BYTES)
                    ));
                }
            }
        }
    }
}
