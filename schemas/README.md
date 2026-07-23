# Public schemas

This directory contains versioned machine-readable JSON schemas for available `career-core` and `career` CLI contracts.

Rules:

- Never edit an existing stable schema to introduce a breaking semantic change.
- Keep schema version values aligned with Rust contract constants and CLI examples.
- Planned capabilities do not receive operation schemas until their phase defines and tests the contract.
- Examples and schemas must contain synthetic data only.

Current schemas:

- `capabilities-v1.schema.json` — capability discovery
- `resume-input-v1.schema.json` — bounded plain-text resume input
- `resume-evaluation-v1.schema.json` — Phase 1 section-coverage evaluation
- `resume-analysis-v1.schema.json` — full deterministic resume-readiness scoring, checks, evidence, uncertainty, and actions
- `resume-normalization-v1.schema.json` — deterministic normalized resume, confidence, provenance, warnings, and enrichment eligibility
- `resume-enrichment-proposal-v1.schema.json` — provider-neutral source-grounded proposal
- `resume-enrichment-input-v1.schema.json` — resume plus explicit proposal envelope
- `resume-enrichment-result-v1.schema.json` — preserved deterministic baseline plus separate assisted document and merge provenance
- `job-input-v1.schema.json` — bounded plain-text job-description input
- `job-normalization-v1.schema.json` — deterministic source-grounded job fields, confidence, metadata, statuses, and warnings
- `error-v1.schema.json` — machine-readable core and CLI failures

Composite analysis and enrichment schemas reference sibling schema files. Offline validators should resolve them from this directory; for `check-jsonschema`, pass `--base-uri "file://$(pwd)/schemas/"` from the repository root.
