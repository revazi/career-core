use clap::ValueEnum;
use serde::Serialize;

pub(crate) const SCHEMA_CATALOG_VERSION: &str = "career.schema_catalog.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum SchemaId {
    #[value(name = "career.capabilities.v1")]
    CapabilitiesV1,
    #[value(name = "career.error.v1")]
    ErrorV1,
    #[value(name = "career.job_input.v1")]
    JobInputV1,
    #[value(name = "career.job_match.v1")]
    JobMatchV1,
    #[value(name = "career.job_match_input.v1")]
    JobMatchInputV1,
    #[value(name = "career.job_normalization.v1")]
    JobNormalizationV1,
    #[value(name = "career.resume_analysis.v1")]
    ResumeAnalysisV1,
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
    pub(crate) const ALL: [Self; 19] = [
        Self::CapabilitiesV1,
        Self::ErrorV1,
        Self::JobInputV1,
        Self::JobMatchV1,
        Self::JobMatchInputV1,
        Self::JobNormalizationV1,
        Self::ResumeAnalysisV1,
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
        SchemaId::ResumeAnalysisV1 => EmbeddedSchema {
            id: "career.resume_analysis.v1",
            file_name: "resume-analysis-v1.schema.json",
            title: "career-core deterministic resume analysis v1",
            document: include_str!("../../../schemas/resume-analysis-v1.schema.json"),
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
