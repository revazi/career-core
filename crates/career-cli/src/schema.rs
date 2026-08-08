use std::collections::{BTreeMap, BTreeSet};

use clap::ValueEnum;
use serde::Serialize;
use serde_json::{Map, Value};

const BUNDLE_CONTAINER_KEY: &str = "careerSchemaBundle";

pub(crate) const SCHEMA_CATALOG_VERSION: &str = "career.schema_catalog.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum SchemaId {
    #[value(name = "career.capabilities.v1")]
    CapabilitiesV1,
    #[value(name = "career.error.v1")]
    ErrorV1,
    #[value(name = "career.job_input.v1")]
    JobInputV1,
    #[value(name = "career.operation_catalog.v1")]
    OperationCatalogV1,
    #[value(name = "career.job_match.v1")]
    JobMatchV1,
    #[value(name = "career.job_match_input.v1")]
    JobMatchInputV1,
    #[value(name = "career.job_normalization.v1")]
    JobNormalizationV1,
    #[value(name = "career.resume_analysis.v1")]
    ResumeAnalysisV1,
    #[value(name = "career.resume_analysis_replacement_proposal.v1")]
    ResumeAnalysisReplacementProposalV1,
    #[value(name = "career.resume_analysis_replacement_review.v1")]
    ResumeAnalysisReplacementReviewV1,
    #[value(name = "career.resume_analysis_replacement_review_input.v1")]
    ResumeAnalysisReplacementReviewInputV1,
    #[value(name = "career.resume_analysis_suggestion_proposal.v1")]
    ResumeAnalysisSuggestionProposalV1,
    #[value(name = "career.resume_analysis_suggestion_review.v1")]
    ResumeAnalysisSuggestionReviewV1,
    #[value(name = "career.resume_analysis_suggestion_review_input.v1")]
    ResumeAnalysisSuggestionReviewInputV1,
    #[value(name = "career.resume_enrichment_input.v1")]
    ResumeEnrichmentInputV1,
    #[value(name = "career.resume_enrichment_proposal.v1")]
    ResumeEnrichmentProposalV1,
    #[value(name = "career.resume_enrichment_result.v1")]
    ResumeEnrichmentResultV1,
    #[value(name = "career.resume_evaluation.v1")]
    ResumeEvaluationV1,
    #[value(name = "career.resume_input.v1")]
    ResumeInputV1,
    #[value(name = "career.resume_normalization.v1")]
    ResumeNormalizationV1,
    #[value(name = "career.resume_variant_materialization_input.v1")]
    ResumeVariantMaterializationInputV1,
    #[value(name = "career.resume_variant_proposal.v1")]
    ResumeVariantProposalV1,
    #[value(name = "career.resume_variant_review.v1")]
    ResumeVariantReviewV1,
    #[value(name = "career.resume_variant_review_input.v1")]
    ResumeVariantReviewInputV1,
    #[value(name = "career.resume_variant.v1")]
    ResumeVariantV1,
    #[value(name = "career.schema_catalog.v1")]
    SchemaCatalogV1,
}

impl SchemaId {
    pub(crate) const ALL: [Self; 26] = [
        Self::CapabilitiesV1,
        Self::ErrorV1,
        Self::JobInputV1,
        Self::JobMatchV1,
        Self::JobMatchInputV1,
        Self::JobNormalizationV1,
        Self::ResumeAnalysisV1,
        Self::ResumeAnalysisReplacementProposalV1,
        Self::ResumeAnalysisReplacementReviewV1,
        Self::ResumeAnalysisReplacementReviewInputV1,
        Self::ResumeAnalysisSuggestionProposalV1,
        Self::ResumeAnalysisSuggestionReviewV1,
        Self::ResumeAnalysisSuggestionReviewInputV1,
        Self::ResumeEnrichmentInputV1,
        Self::ResumeEnrichmentProposalV1,
        Self::ResumeEnrichmentResultV1,
        Self::ResumeEvaluationV1,
        Self::ResumeInputV1,
        Self::ResumeNormalizationV1,
        Self::ResumeVariantMaterializationInputV1,
        Self::ResumeVariantProposalV1,
        Self::ResumeVariantReviewV1,
        Self::ResumeVariantReviewInputV1,
        Self::ResumeVariantV1,
        Self::SchemaCatalogV1,
        Self::OperationCatalogV1,
    ];
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EmbeddedSchema {
    pub(crate) id: &'static str,
    pub(crate) file_name: &'static str,
    pub(crate) title: &'static str,
    pub(crate) document: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SchemaCatalogV1 {
    pub(crate) schema_version: &'static str,
    pub(crate) schemas: Vec<SchemaCatalogEntryV1>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SchemaCatalogEntryV1 {
    pub(crate) id: &'static str,
    pub(crate) file_name: &'static str,
    pub(crate) title: &'static str,
}

pub(crate) fn schema_catalog() -> SchemaCatalogV1 {
    SchemaCatalogV1 {
        schema_version: SCHEMA_CATALOG_VERSION,
        schemas: SchemaId::ALL
            .iter()
            .map(|id| {
                let schema = embedded_schema(*id);
                SchemaCatalogEntryV1 {
                    id: schema.id,
                    file_name: schema.file_name,
                    title: schema.title,
                }
            })
            .collect(),
    }
}

pub(crate) const fn embedded_schema(id: SchemaId) -> EmbeddedSchema {
    match id {
        SchemaId::CapabilitiesV1 => EmbeddedSchema {
            id: "career.capabilities.v1",
            file_name: "capabilities-v1.schema.json",
            title: "career-core capabilities v1",
            document: include_str!("../../../schemas/capabilities-v1.schema.json"),
        },
        SchemaId::ErrorV1 => EmbeddedSchema {
            id: "career.error.v1",
            file_name: "error-v1.schema.json",
            title: "career CLI error v1",
            document: include_str!("../../../schemas/error-v1.schema.json"),
        },
        SchemaId::JobInputV1 => EmbeddedSchema {
            id: "career.job_input.v1",
            file_name: "job-input-v1.schema.json",
            title: "career-core job-description input v1",
            document: include_str!("../../../schemas/job-input-v1.schema.json"),
        },
        SchemaId::JobMatchV1 => EmbeddedSchema {
            id: "career.job_match.v1",
            file_name: "job-match-v1.schema.json",
            title: "career-core deterministic job match v1",
            document: include_str!("../../../schemas/job-match-v1.schema.json"),
        },
        SchemaId::JobMatchInputV1 => EmbeddedSchema {
            id: "career.job_match_input.v1",
            file_name: "job-match-input-v1.schema.json",
            title: "career-core job match input v1",
            document: include_str!("../../../schemas/job-match-input-v1.schema.json"),
        },
        SchemaId::JobNormalizationV1 => EmbeddedSchema {
            id: "career.job_normalization.v1",
            file_name: "job-normalization-v1.schema.json",
            title: "career-core deterministic job-description normalization v1",
            document: include_str!("../../../schemas/job-normalization-v1.schema.json"),
        },
        SchemaId::OperationCatalogV1 => EmbeddedSchema {
            id: "career.operation_catalog.v1",
            file_name: "operation-catalog-v1.schema.json",
            title: "career CLI operation catalog v1",
            document: include_str!("../../../schemas/operation-catalog-v1.schema.json"),
        },
        SchemaId::ResumeAnalysisV1 => EmbeddedSchema {
            id: "career.resume_analysis.v1",
            file_name: "resume-analysis-v1.schema.json",
            title: "career-core deterministic resume analysis v1",
            document: include_str!("../../../schemas/resume-analysis-v1.schema.json"),
        },
        SchemaId::ResumeAnalysisReplacementProposalV1 => EmbeddedSchema {
            id: "career.resume_analysis_replacement_proposal.v1",
            file_name: "resume-analysis-replacement-proposal-v1.schema.json",
            title: "career-core external resume analysis replacement proposal v1",
            document: include_str!(
                "../../../schemas/resume-analysis-replacement-proposal-v1.schema.json"
            ),
        },
        SchemaId::ResumeAnalysisReplacementReviewV1 => EmbeddedSchema {
            id: "career.resume_analysis_replacement_review.v1",
            file_name: "resume-analysis-replacement-review-v1.schema.json",
            title: "career-core reviewed external resume analysis replacements v1",
            document: include_str!(
                "../../../schemas/resume-analysis-replacement-review-v1.schema.json"
            ),
        },
        SchemaId::ResumeAnalysisReplacementReviewInputV1 => EmbeddedSchema {
            id: "career.resume_analysis_replacement_review_input.v1",
            file_name: "resume-analysis-replacement-review-input-v1.schema.json",
            title: "career-core external resume analysis replacement review input v1",
            document: include_str!(
                "../../../schemas/resume-analysis-replacement-review-input-v1.schema.json"
            ),
        },
        SchemaId::ResumeAnalysisSuggestionProposalV1 => EmbeddedSchema {
            id: "career.resume_analysis_suggestion_proposal.v1",
            file_name: "resume-analysis-suggestion-proposal-v1.schema.json",
            title: "career-core external resume analysis suggestion proposal v1",
            document: include_str!(
                "../../../schemas/resume-analysis-suggestion-proposal-v1.schema.json"
            ),
        },
        SchemaId::ResumeAnalysisSuggestionReviewV1 => EmbeddedSchema {
            id: "career.resume_analysis_suggestion_review.v1",
            file_name: "resume-analysis-suggestion-review-v1.schema.json",
            title: "career-core reviewed external resume analysis suggestions v1",
            document: include_str!(
                "../../../schemas/resume-analysis-suggestion-review-v1.schema.json"
            ),
        },
        SchemaId::ResumeAnalysisSuggestionReviewInputV1 => EmbeddedSchema {
            id: "career.resume_analysis_suggestion_review_input.v1",
            file_name: "resume-analysis-suggestion-review-input-v1.schema.json",
            title: "career-core external resume analysis suggestion review input v1",
            document: include_str!(
                "../../../schemas/resume-analysis-suggestion-review-input-v1.schema.json"
            ),
        },
        SchemaId::ResumeEnrichmentInputV1 => EmbeddedSchema {
            id: "career.resume_enrichment_input.v1",
            file_name: "resume-enrichment-input-v1.schema.json",
            title: "career-core resume enrichment input v1",
            document: include_str!("../../../schemas/resume-enrichment-input-v1.schema.json"),
        },
        SchemaId::ResumeEnrichmentProposalV1 => EmbeddedSchema {
            id: "career.resume_enrichment_proposal.v1",
            file_name: "resume-enrichment-proposal-v1.schema.json",
            title: "career-core resume enrichment proposal v1",
            document: include_str!("../../../schemas/resume-enrichment-proposal-v1.schema.json"),
        },
        SchemaId::ResumeEnrichmentResultV1 => EmbeddedSchema {
            id: "career.resume_enrichment_result.v1",
            file_name: "resume-enrichment-result-v1.schema.json",
            title: "career-core resume enrichment result v1",
            document: include_str!("../../../schemas/resume-enrichment-result-v1.schema.json"),
        },
        SchemaId::ResumeEvaluationV1 => EmbeddedSchema {
            id: "career.resume_evaluation.v1",
            file_name: "resume-evaluation-v1.schema.json",
            title: "career-core resume evaluation v1",
            document: include_str!("../../../schemas/resume-evaluation-v1.schema.json"),
        },
        SchemaId::ResumeInputV1 => EmbeddedSchema {
            id: "career.resume_input.v1",
            file_name: "resume-input-v1.schema.json",
            title: "career-core resume input v1",
            document: include_str!("../../../schemas/resume-input-v1.schema.json"),
        },
        SchemaId::ResumeNormalizationV1 => EmbeddedSchema {
            id: "career.resume_normalization.v1",
            file_name: "resume-normalization-v1.schema.json",
            title: "career-core resume normalization v1",
            document: include_str!("../../../schemas/resume-normalization-v1.schema.json"),
        },
        SchemaId::ResumeVariantMaterializationInputV1 => EmbeddedSchema {
            id: "career.resume_variant_materialization_input.v1",
            file_name: "resume-variant-materialization-input-v1.schema.json",
            title: "career-core assisted resume variant materialization input v1",
            document: include_str!(
                "../../../schemas/resume-variant-materialization-input-v1.schema.json"
            ),
        },
        SchemaId::ResumeVariantProposalV1 => EmbeddedSchema {
            id: "career.resume_variant_proposal.v1",
            file_name: "resume-variant-proposal-v1.schema.json",
            title: "career-core assisted resume variant proposal v1",
            document: include_str!("../../../schemas/resume-variant-proposal-v1.schema.json"),
        },
        SchemaId::ResumeVariantReviewV1 => EmbeddedSchema {
            id: "career.resume_variant_review.v1",
            file_name: "resume-variant-review-v1.schema.json",
            title: "career-core assisted resume variant review v1",
            document: include_str!("../../../schemas/resume-variant-review-v1.schema.json"),
        },
        SchemaId::ResumeVariantReviewInputV1 => EmbeddedSchema {
            id: "career.resume_variant_review_input.v1",
            file_name: "resume-variant-review-input-v1.schema.json",
            title: "career-core assisted resume variant review input v1",
            document: include_str!("../../../schemas/resume-variant-review-input-v1.schema.json"),
        },
        SchemaId::ResumeVariantV1 => EmbeddedSchema {
            id: "career.resume_variant.v1",
            file_name: "resume-variant-v1.schema.json",
            title: "career-core assisted resume variant v1",
            document: include_str!("../../../schemas/resume-variant-v1.schema.json"),
        },
        SchemaId::SchemaCatalogV1 => EmbeddedSchema {
            id: "career.schema_catalog.v1",
            file_name: "schema-catalog-v1.schema.json",
            title: "career CLI schema catalog v1",
            document: include_str!("../../../schemas/schema-catalog-v1.schema.json"),
        },
    }
}

pub(crate) fn schema_bundle(id: SchemaId) -> Result<Value, ()> {
    let root = embedded_schema(id);
    let embedded_schemas = SchemaId::ALL.map(embedded_schema);
    let documents = parse_embedded_documents(&embedded_schemas)?;

    let mut dependency_files = BTreeSet::new();
    collect_dependencies(
        root.file_name,
        root.file_name,
        &documents,
        &mut dependency_files,
    )?;

    let mut bundled_dependencies = Map::new();
    for dependency_file in &dependency_files {
        let mut dependency = documents.get(dependency_file.as_str()).cloned().ok_or(())?;
        let dependency_object = dependency.as_object_mut().ok_or(())?;
        dependency_object.remove("$schema");
        dependency_object.remove("$id");
        rewrite_references(&mut dependency, dependency_file, root.file_name, &documents)?;
        bundled_dependencies.insert(dependency_file.clone(), dependency);
    }

    let mut bundled_root = documents.get(root.file_name).cloned().ok_or(())?;
    rewrite_references(
        &mut bundled_root,
        root.file_name,
        root.file_name,
        &documents,
    )?;

    if !bundled_dependencies.is_empty() {
        let root_object = bundled_root.as_object_mut().ok_or(())?;
        let root_definitions = root_object
            .entry("$defs")
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or(())?;
        if root_definitions.contains_key(BUNDLE_CONTAINER_KEY) {
            return Err(());
        }
        root_definitions.insert(
            BUNDLE_CONTAINER_KEY.to_owned(),
            Value::Object(Map::from_iter([(
                "$defs".to_owned(),
                Value::Object(bundled_dependencies),
            )])),
        );
    }

    Ok(bundled_root)
}

fn parse_embedded_documents(
    embedded_schemas: &[EmbeddedSchema],
) -> Result<BTreeMap<&str, Value>, ()> {
    let mut schema_ids = BTreeSet::new();
    let mut documents = BTreeMap::new();
    for schema in embedded_schemas {
        if !schema_ids.insert(schema.id) {
            return Err(());
        }
        let document = serde_json::from_str::<Value>(schema.document).map_err(|_| ())?;
        if documents.insert(schema.file_name, document).is_some() {
            return Err(());
        }
    }
    Ok(documents)
}

fn collect_dependencies(
    current_file: &str,
    root_file: &str,
    documents: &BTreeMap<&str, Value>,
    dependency_files: &mut BTreeSet<String>,
) -> Result<(), ()> {
    let document = documents.get(current_file).ok_or(())?;
    let mut references = Vec::new();
    collect_references(document, &mut references)?;

    for reference in references {
        let Some((referenced_file, _)) = split_external_reference(reference)? else {
            continue;
        };
        if !documents.contains_key(referenced_file) {
            return Err(());
        }
        if referenced_file != root_file && dependency_files.insert(referenced_file.to_owned()) {
            collect_dependencies(referenced_file, root_file, documents, dependency_files)?;
        }
    }

    Ok(())
}

fn collect_references<'a>(value: &'a Value, references: &mut Vec<&'a str>) -> Result<(), ()> {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref") {
                references.push(reference.as_str().ok_or(())?);
            }
            for child in object.values() {
                collect_references(child, references)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_references(child, references)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn rewrite_references(
    value: &mut Value,
    current_file: &str,
    root_file: &str,
    documents: &BTreeMap<&str, Value>,
) -> Result<(), ()> {
    match value {
        Value::Object(object) => {
            if let Some(reference_value) = object.get_mut("$ref") {
                let reference = reference_value.as_str().ok_or(())?;
                let rewritten = if let Some(local_suffix) = reference.strip_prefix('#') {
                    validate_fragment(local_suffix)?;
                    if current_file == root_file {
                        reference.to_owned()
                    } else {
                        format!("{}{}", dependency_pointer(current_file), local_suffix)
                    }
                } else {
                    let (referenced_file, fragment) =
                        split_external_reference(reference)?.ok_or(())?;
                    if !documents.contains_key(referenced_file) {
                        return Err(());
                    }
                    if referenced_file == root_file {
                        format!("#{fragment}")
                    } else {
                        format!("{}{fragment}", dependency_pointer(referenced_file))
                    }
                };
                *reference_value = Value::String(rewritten);
            }
            for child in object.values_mut() {
                rewrite_references(child, current_file, root_file, documents)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                rewrite_references(child, current_file, root_file, documents)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn split_external_reference(reference: &str) -> Result<Option<(&str, &str)>, ()> {
    if let Some(fragment) = reference.strip_prefix('#') {
        validate_fragment(fragment)?;
        return Ok(None);
    }

    let (file_name, fragment) = reference.split_once('#').unwrap_or((reference, ""));
    if file_name.is_empty() {
        return Err(());
    }
    validate_fragment(fragment)?;
    Ok(Some((file_name, fragment)))
}

fn validate_fragment(fragment: &str) -> Result<(), ()> {
    if fragment.is_empty() || fragment.starts_with('/') {
        Ok(())
    } else {
        Err(())
    }
}

fn dependency_pointer(file_name: &str) -> String {
    format!(
        "#/$defs/{BUNDLE_CONTAINER_KEY}/$defs/{}",
        file_name.replace('~', "~0").replace('/', "~1")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundle_is_deterministic_and_has_only_resolved_local_references() {
        for id in SchemaId::ALL {
            let bundle = schema_bundle(id).expect("embedded schemas should bundle");
            assert_eq!(
                bundle,
                schema_bundle(id).expect("repeated bundling should succeed")
            );
            assert_eq!(
                bundle["$schema"],
                "https://json-schema.org/draft/2020-12/schema"
            );

            let mut references = Vec::new();
            collect_references(&bundle, &mut references).expect("bundle refs should be strings");
            for reference in references {
                assert!(
                    reference.starts_with('#'),
                    "non-local reference: {reference}"
                );
                let pointer = reference
                    .strip_prefix('#')
                    .expect("local reference should start with a hash");
                assert!(
                    bundle.pointer(pointer).is_some(),
                    "unresolved local reference: {reference}"
                );
            }
        }
    }

    #[test]
    fn composite_bundle_resolves_references_recursively() {
        let bundle = schema_bundle(SchemaId::ResumeVariantMaterializationInputV1)
            .expect("composite schema should bundle");
        let dependency_definitions = bundle["$defs"][BUNDLE_CONTAINER_KEY]["$defs"]
            .as_object()
            .expect("bundle should contain dependency schemas");

        assert!(dependency_definitions.contains_key("resume-variant-review-input-v1.schema.json"));
        assert!(dependency_definitions.contains_key("resume-variant-proposal-v1.schema.json"));
        assert!(dependency_definitions.contains_key("resume-input-v1.schema.json"));
        assert!(dependency_definitions.contains_key("job-input-v1.schema.json"));
    }

    #[test]
    fn duplicate_embedded_schema_ids_or_files_fail_closed() {
        const DOCUMENT: &str =
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object"}"#;
        let first = EmbeddedSchema {
            id: "career.first.v1",
            file_name: "first-v1.schema.json",
            title: "first",
            document: DOCUMENT,
        };

        assert!(parse_embedded_documents(&[first, first]).is_err());
        let duplicate_id = EmbeddedSchema {
            file_name: "second-v1.schema.json",
            ..first
        };
        assert!(parse_embedded_documents(&[first, duplicate_id]).is_err());
        let duplicate_file = EmbeddedSchema {
            id: "career.second.v1",
            ..first
        };
        assert!(parse_embedded_documents(&[first, duplicate_file]).is_err());
    }

    #[test]
    fn unknown_or_remote_references_fail_closed() {
        let documents = BTreeMap::from([(
            "root.schema.json",
            serde_json::json!({"$ref": "https://example.invalid/remote.schema.json"}),
        )]);
        let mut dependencies = BTreeSet::new();

        assert!(
            collect_dependencies(
                "root.schema.json",
                "root.schema.json",
                &documents,
                &mut dependencies
            )
            .is_err()
        );
    }
}
