# CLI contract reference

## Capability discovery

```bash
career capabilities [--format json|text]
```

JSON is the default. Its schema is `schemas/capabilities-v1.schema.json`. Invoke only capabilities whose `status` is `available`.

## Resume section-coverage evaluation

```bash
career resume evaluate --input <path|-> [--format json|text]
```

Input is `career.resume_input.v1` JSON, documented by:

- `schemas/resume-input-v1.schema.json`
- `docs/contracts/resume-evaluation-v1.md`

Use `-` to read JSON from stdin. JSON output follows `schemas/resume-evaluation-v1.schema.json`. The operation accepts plain extracted text only; it does not open PDF or DOCX files.

Phase 1 evaluation scope is `section_coverage`. Scores only represent four exact-header checks and must not be described as a full resume-quality or ATS score. Preserve all warnings, especially `limited_evaluation_scope` and parser uncertainty wording.

## Resume normalization

```bash
career resume normalize --input <path|-> [--format json|text]
```

Input is `career.resume_input.v1`; JSON output follows `schemas/resume-normalization-v1.schema.json`. `deterministic_document` is source-grounded and includes parser confidence, field statuses, metadata, warnings, and `enrichment_request`.

## External proposal validation and merge

```bash
career resume enrich --input <path|-> [--format json|text]
```

Input follows `schemas/resume-enrichment-input-v1.schema.json` and contains the original resume input plus an exact `career.resume_enrichment_proposal.v1` proposal. The operation:

- makes no provider or network call
- accepts only low-confidence eligible targets selected by deterministic normalization
- requires every non-empty proposal string to occur in source text
- rejects populated non-targets and bounded-contract violations
- fills only empty eligible fields
- returns the unchanged normalization under `baseline`
- returns accepted fields separately under `assisted_document`

An agent must obtain user approval before using an external model. Provider failure or invalid output falls back to the earlier deterministic normalization. Assisted fields cannot replace authoritative confidence or scoring input.

See `docs/contracts/resume-normalization-v1.md` for all limits and merge rules.

## Machine-process rules

- stdout contains result JSON only on success
- stderr contains error JSON only on failure
- human output requires `--format text`
- no command makes network requests
- no command modifies its input

Current nonzero exit statuses:

| Exit | Meaning |
|---:|---|
| 2 | command-line usage error from argument parsing |
| 3 | input path/read/CLI byte-limit failure |
| 4 | malformed or structurally invalid JSON |
| 5 | valid JSON rejected by core input validation |
| 6 | output write/serialization failure |

Errors use `career.error.v1`. Messages are bounded and do not echo the source document.

## Execution from source

```bash
cargo run --quiet -p career-cli -- capabilities
cargo run --quiet -p career-cli -- resume evaluate --input fixtures/resume/phase1/complete-sections.input.json
cargo run --quiet -p career-cli -- resume normalize --input fixtures/resume/phase2/complete-normalization.input.json
cargo run --quiet -p career-cli -- resume enrich --input fixtures/resume/phase2/messy-unlabeled.enrichment-input.json
```

The `--quiet` Cargo flag suppresses Cargo status output; it does not alter `career` output.
