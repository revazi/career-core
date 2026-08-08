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
- `operation-catalog-v1.schema.json` — stable callable operations, capability mapping, transports, schemas, and byte bounds from `career operations`
- `resume-input-v1.schema.json` — bounded plain-text resume input
- `resume-evaluation-v1.schema.json` — Phase 1 section-coverage evaluation
- `resume-analysis-v1.schema.json` — full deterministic resume-readiness scoring, checks, evidence, uncertainty, and actions
- `resume-analysis-replacement-proposal-v1.schema.json` — bounded untrusted exact external replacements
- `resume-analysis-replacement-review-input-v1.schema.json` — original resume, expected analysis policy, and replacement proposal
- `resume-analysis-replacement-review-v1.schema.json` — preserved analysis baseline, canonical assisted replacements, discards, and warnings
- `resume-analysis-suggestion-proposal-v1.schema.json` — bounded untrusted source-targeted external suggestions
- `resume-analysis-suggestion-review-input-v1.schema.json` — original resume, expected analysis policy, and proposal
- `resume-analysis-suggestion-review-v1.schema.json` — preserved analysis baseline, canonical assisted suggestions, discards, and warnings
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

Composite analysis, enrichment, and matching schemas reference sibling schema files. Offline validators may resolve them from this directory; for `check-jsonschema`, pass `--base-uri "file://$(pwd)/schemas/"` from the repository root. An installed CLI can instead emit a recursively self-contained schema whose refs all use root-local JSON Pointers:

```bash
career schema bundle --id career.resume_variant_materialization_input.v1
```

The requested root retains its Draft 2020-12 marker and `$id`; embedded dependency resources are placed under the reserved `careerSchemaBundle` definition with dependency `$schema`/`$id` removed. Only exact embedded sibling references are accepted. Unknown, remote, or unresolved references fail closed without source-tree or network lookup.

An installed CLI embeds these exact reviewed files:

```bash
career operations --format json-compact
career schema list --format json-compact
career schema export --id career.resume_input.v1
career schema bundle --id career.job_match_input.v1
```

Both catalogs are static and ordered. Schema export preserves reviewed file bytes. Bundle construction uses only embedded documents and deterministic local-reference rewriting; neither command performs source-tree lookup or network requests.
