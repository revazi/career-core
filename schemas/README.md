# Public schemas

This directory contains versioned machine-readable JSON schemas for available `career-core` and `career` CLI contracts.

Rules:

- Never edit an existing stable schema to introduce a breaking semantic change.
- Keep schema version values aligned with Rust contract constants and CLI examples.
- Planned capabilities do not receive operation schemas until their phase defines and tests the contract.
- Examples and schemas must contain synthetic data only.

Current schemas:

- `capabilities-v1.schema.json` — capability discovery
- `schema-catalog-v1.schema.json` — embedded offline schema discovery from `career schema list`
- `resume-input-v1.schema.json` — bounded plain-text resume input
- `resume-evaluation-v1.schema.json` — Phase 1 section-coverage evaluation
- `resume-analysis-v1.schema.json` — full deterministic resume-readiness scoring, checks, evidence, uncertainty, and actions
- `resume-normalization-v1.schema.json` — deterministic normalized resume, confidence, provenance, warnings, and enrichment eligibility
- `resume-enrichment-proposal-v1.schema.json` — provider-neutral source-grounded proposal
- `resume-enrichment-input-v1.schema.json` — resume plus explicit proposal envelope
- `resume-enrichment-result-v1.schema.json` — preserved deterministic baseline plus separate assisted document and merge provenance
- `resume-variant-proposal-v1.schema.json` — bounded evidence-linked external line changes
- `resume-variant-review-input-v1.schema.json` — original resume/vacancy plus untrusted variant proposal
- `resume-variant-review-v1.schema.json` — canonical selectable changes, assisted preview, discard diagnostics, and warnings
- `resume-variant-materialization-input-v1.schema.json` — exact review input plus selected canonical change identifiers
- `resume-variant-v1.schema.json` — preserved baseline plus deterministically materialized non-authoritative variant
- `job-input-v1.schema.json` — bounded plain-text job-description input
- `job-normalization-v1.schema.json` — deterministic source-grounded job fields, confidence, metadata, statuses, and warnings
- `job-match-input-v1.schema.json` — original resume plus job-description input envelope
- `job-match-v1.schema.json` — conservative deterministic scores, evidence, confidence, strengths, gaps, and recommendation gates
- `error-v1.schema.json` — machine-readable core and CLI failures

Composite analysis, enrichment, and matching schemas reference sibling schema files. Offline validators should resolve them from this directory; for `check-jsonschema`, pass `--base-uri "file://$(pwd)/schemas/"` from the repository root.

An installed CLI embeds these exact reviewed files:

```bash
career schema list --format json-compact
career schema export --id career.resume_input.v1
```

The catalog is static and ordered. Schema export performs no source-tree lookup, runtime generation, or network request.
