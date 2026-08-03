#![doc = "Deterministic, explainable career-document analysis primitives."]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

mod job;
mod resume;

pub use job::*;
pub use resume::*;

/// Version of the machine-readable capability document.
pub const CAPABILITIES_SCHEMA_VERSION: &str = "career.capabilities.v1";

/// Describes whether a public capability can be used by a caller.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Available,
    Planned,
}

/// One discoverable core capability.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Capability {
    pub id: String,
    pub status: CapabilityStatus,
    pub summary: String,
}

/// Stable discovery response used by CLIs and agent adapters.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Capabilities {
    pub schema_version: String,
    pub core_version: String,
    pub deterministic: bool,
    pub performs_network_requests: bool,
    pub capabilities: Vec<Capability>,
}

/// Returns the capabilities supported or explicitly planned by this build.
///
/// Planned capabilities are deliberately reported as unavailable so callers do
/// not infer behavior that has not been implemented and tested.
#[must_use]
pub fn capabilities() -> Capabilities {
    Capabilities {
        schema_version: CAPABILITIES_SCHEMA_VERSION.to_owned(),
        core_version: env!("CARGO_PKG_VERSION").to_owned(),
        deterministic: true,
        performs_network_requests: false,
        capabilities: vec![
            Capability {
                id: "core.capabilities".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Discover versioned functionality exposed by this build.".to_owned(),
            },
            Capability {
                id: "resume.evaluate".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Evaluate recognized resume section coverage with explainable deterministic checks."
                    .to_owned(),
            },
            Capability {
                id: "resume.analyze".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Analyze resume readiness with 18 explainable deterministic checks and confidence-aware evidence."
                    .to_owned(),
            },
            Capability {
                id: "resume.normalize".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Normalize bounded resume text into source-grounded deterministic facts and confidence."
                    .to_owned(),
            },
            Capability {
                id: "resume.enrich".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Validate and conservatively merge an explicit source-grounded external proposal without network access."
                    .to_owned(),
            },
            Capability {
                id: "resume.analysis-suggestions.review".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Review bounded external suggestions against a freshly rerun deterministic analysis without changing it."
                    .to_owned(),
            },
            Capability {
                id: "resume.analysis-replacements.review".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Review bounded exact external replacements against a freshly rerun deterministic analysis without changing it."
                    .to_owned(),
            },
            Capability {
                id: "resume.variant.review".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Review bounded evidence-linked external resume changes without certifying generated prose."
                    .to_owned(),
            },
            Capability {
                id: "resume.variant.materialize".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Revalidate and deterministically materialize only explicitly selected assisted resume changes."
                    .to_owned(),
            },
            Capability {
                id: "job.normalize".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Normalize bounded job-description text into source-grounded deterministic facts and confidence."
                    .to_owned(),
            },
            Capability {
                id: "job.match".to_owned(),
                status: CapabilityStatus::Available,
                summary: "Match deterministic resume and job baselines with conservative equivalence and confidence-aware evidence."
                    .to_owned(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_are_deterministic_and_network_free() {
        let first = capabilities();
        let second = capabilities();

        assert_eq!(first, second);
        assert!(first.deterministic);
        assert!(!first.performs_network_requests);
        assert_eq!(first.schema_version, CAPABILITIES_SCHEMA_VERSION);
    }

    #[test]
    fn discovery_and_document_operations_are_available() {
        let capabilities = capabilities();
        let available_ids = capabilities
            .capabilities
            .iter()
            .filter(|capability| capability.status == CapabilityStatus::Available)
            .map(|capability| capability.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            available_ids,
            vec![
                "core.capabilities",
                "resume.evaluate",
                "resume.analyze",
                "resume.normalize",
                "resume.enrich",
                "resume.analysis-suggestions.review",
                "resume.analysis-replacements.review",
                "resume.variant.review",
                "resume.variant.materialize",
                "job.normalize",
                "job.match",
            ]
        );
    }

    #[test]
    fn capabilities_serialize_to_the_documented_shape() {
        let value = serde_json::to_value(capabilities()).expect("capabilities should serialize");

        assert_eq!(value["schema_version"], CAPABILITIES_SCHEMA_VERSION);
        assert_eq!(value["deterministic"], true);
        assert_eq!(value["performs_network_requests"], false);
        assert!(value["capabilities"].is_array());
    }
}
