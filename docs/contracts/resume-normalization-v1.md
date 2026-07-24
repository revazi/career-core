# Resume normalization and external enrichment v1

## Purpose

Phase 2 converts bounded caller-extracted resume text into source-grounded structured facts. It also defines a provider-neutral boundary through which an agent or application may submit an optional external normalization proposal.

The root `career-core` library never reads API keys, selects a provider, constructs hidden prompts, or makes a network request.

## Contracts and policies

| Surface | Version |
|---|---|
| Resume input | `career.resume_input.v1` |
| Deterministic normalization | `career.resume_normalization.v1` |
| Deterministic policy | `resume_normalization_v1` |
| External proposal | `career.resume_enrichment_proposal.v1` |
| Enrichment input | `career.resume_enrichment_input.v1` |
| Enrichment result | `career.resume_enrichment_result.v1` |
| Enrichment policy | `resume_normalization_enrichment_v1` |
| Errors | `career.error.v1` |

## Deterministic document

`deterministic_document` contains bounded structures for:

- contact name, email, phone, and links
- summary
- experience entries and bullets
- education entries
- skills
- projects
- certifications

Every non-empty value is represented as a `value` plus a source span containing one-based start/end lines, a bounded excerpt, and an explicit transformation label.

Transformation labels are:

- `verbatim`
- `whitespace_joined`
- `delimiter_split`
- `conservative_fallback`
- `external_source_grounded_proposal`

The last label appears only in the separate assisted document.

## Header and fallback policy

Section headers use explicit normalized whole-line aliases adapted from Django policy `resume_normalization_v7`. There is no fuzzy or prose matching. Summary, Experience, Education, and Skills are expected core sections. Projects and Certifications are optional.

When stronger section parsing leaves a field empty, bounded deterministic fallbacks may recover:

- experience near explicit date ranges and role terms
- comma-separated skills with prose rejection gates
- education near explicit degree and institution terms

Fallbacks never overwrite non-empty section parser output. Their identifiers and source transformations remain visible.

## Confidence

Parser confidence is an integer from 0 through 100 using five fixed signals:

| Signal | Maximum |
|---|---:|
| Extraction quality | 25 |
| Expected-section coverage | 25 |
| Contact completeness | 15 |
| Experience completeness | 25 |
| Skills completeness | 10 |

Labels follow the selected Django thresholds:

- `high`: total at least 75 plus minimum gates for all five signals
- `medium`: total at least 50 and extraction quality at least 8
- `low`: total at least 20
- `unknown`: below 20

Signal scores and bounded evidence strings are public. Fallback or assisted data does not recalculate or upgrade deterministic confidence.

## Field statuses

Fields use:

- `detected`: deterministic structured evidence exists
- `likely_missing`: parsing evidence is strong enough to treat an empty expected field as likely missing
- `not_detected`: the parser could not verify the field; this is not confirmed absence

Projects and Certifications are optional and therefore are not labeled `likely_missing` merely because they were not found.

## Output limits

| Value | Limit |
|---|---:|
| Source text | 50,000 characters |
| Source lines | 2,000 |
| Source line | 2,000 characters |
| Summary | 2,000 characters |
| Experience entries | 15 |
| Education entries | 10 |
| Skills | 50 |
| Projects | 15 |
| Certifications | 15 |
| Entry raw text | 4,000 characters |
| Short field | 300 characters |
| Experience bullets per entry | 20 |
| Bullet | 1,000 characters |
| Links | 10 |
| Project description | 2,000 characters |
| Source excerpt | 160 characters |
| CLI JSON input | 262,144 bytes |
| External proposal total string content | 50,000 characters |

A deterministic output that reaches a normalization limit includes `output_truncated` metadata and a warning. The CLI byte limit is enforced before JSON parsing.

## Enrichment eligibility

A deterministic normalization emits an `enrichment_request` rather than making a provider call.

The request is `eligible` only when:

1. deterministic parse confidence is exactly `low`; and
2. at least one of Summary, Experience, Education, or Skills remains empty.

Targets are derived by the core in canonical order. A host must not invent or expand the target list.

Unknown, medium, and high confidence results are not eligible. This follows the bounded trigger used by the Django fallback rather than sending every resume to a model.

## Host-orchestrated flow

An integrating host may perform this flow only after the user has enabled external enrichment:

1. Call deterministic normalization.
2. Inspect `enrichment_request.status` and `target_sections`.
3. If eligible and a provider is configured, send only reviewed bounded context to that provider.
4. Require exact `career.resume_enrichment_proposal.v1` JSON.
5. Submit the original resume input and proposal through `apply_resume_enrichment` or `career resume enrich`.
6. If provider or validation fails, retain and return the deterministic normalization.

API keys, provider errors, raw provider responses, and prompts do not enter core output.

## Proposal rules

A proposal always contains exact Summary, Experience, Education, and Skills keys. Non-target fields must remain empty.

The core rejects a proposal when:

- its schema version is unsupported
- its total string content exceeds 50,000 characters
- a list or string exceeds its field limit
- required nested fields are empty
- an experience entry lacks both company and date evidence
- an education entry lacks both institution and degree evidence
- any non-empty value cannot be located in the supplied source text
- a non-target field is populated

Grounding comparison normalizes Unicode lowercase and whitespace. It does not permit paraphrases, inferred facts, or semantically similar unsupported values.

## Merge and scoring isolation

The enrichment result contains both:

- `baseline`: the complete deterministic normalization
- `assisted_document`: a clone with accepted proposal values filled only into eligible empty fields

It also records applied fields and canonical field provenance:

- `deterministic`
- `external_source_grounded_proposal`
- `not_available`

The baseline is never mutated. Parser confidence and field statuses remain deterministic. Future authoritative evaluation and matching must consume the deterministic baseline, not silently substitute the assisted document.

A consumer may display assisted fields and feedback when clearly labeled, but it must not present them as deterministic extraction or use them to replace an authoritative score.

## CLI

Deterministic normalization:

```bash
career resume normalize --input <path|-> [--format json|json-pretty|json-compact|text]
```

Validate and merge an explicit proposal:

```bash
career resume enrich --input <path|-> [--format json|json-pretty|json-compact|text]
```

`resume enrich` accepts one `career.resume_enrichment_input.v1` envelope containing both the original resume input and proposal. Neither command makes a network request.

## Django provenance and intentional differences

Reference concepts were adapted from:

- `accounts/services/resume_normalization.py` (`resume_normalization_v7`)
- `accounts/services/resume_normalization_fallback_validation.py`
- `accounts/services/normalization_fallback_merge.py`
- normalization and fallback tests/fixtures listed in `.agents/reference-map.md`

Intentional differences:

- Rust performs no provider call and contains no provider client.
- The deterministic baseline and assisted document are separate public values.
- Assisted fields cannot silently influence authoritative scoring.
- Date ranges are rejected as phone-number evidence.
- Education date lines remain attached to their source-grounded education entry.
- External-proposal terminology is provider-neutral; the proposal may come from an agent or local application, not necessarily an LLM service.

This is selected fixture/rule adaptation, not full policy parity with the Django application.
