# `career` CLI contract

The `career` executable is the universal local adapter for coding agents and scripts. It depends on `career-core`, performs no implicit network requests, and does not persist or modify source documents.

## Command hierarchy

```text
career capabilities
career schema list
career schema export --id <contract-id>
career resume evaluate --input <path|->
career resume analyze --input <path|->
career resume analysis-suggestions-review --input <path|->
career resume normalize --input <path|->
career resume enrich --input <path|->
career resume variant-review --input <path|->
career resume variant-materialize --input <path|->
career job normalize --input <path|->
career job match --input <path|->
```

Run `career <command> --help` for complete flags. Every document operation requires an explicit path; `--input -` reads one JSON document from stdin.

## Output formats

Document operations, capability discovery, and schema listing accept:

| Format | Behavior |
|---|---|
| `json` | Canonical pretty JSON and the compatibility-preserving default |
| `json-pretty` | Explicit spelling of the same canonical pretty JSON |
| `json-compact` | One JSON document on one line, followed by one newline |
| `text` | Concise human-readable output; never use it for agent decisions |

`schema export` accepts the three JSON formats but not `text`. Existing `--format json` output remains byte-equivalent to reviewed goldens.

Machine modes write a successful result only to stdout. Failures write one `career.error.v1` document to stderr and leave stdout empty. No progress indicator, ANSI escape, or log line is mixed into machine output.

## Contract map

| Command | Input | JSON output |
|---|---|---|
| `capabilities` | none | `career.capabilities.v1` |
| `schema list` | none | `career.schema_catalog.v1` |
| `schema export` | exact catalog ID | Draft 2020-12 JSON Schema |
| `resume evaluate` | `career.resume_input.v1` | `career.resume_evaluation.v1` |
| `resume analyze` | `career.resume_input.v1` | `career.resume_analysis.v1` |
| `resume analysis-suggestions-review` | `career.resume_analysis_suggestion_review_input.v1` | `career.resume_analysis_suggestion_review.v1` |
| `resume normalize` | `career.resume_input.v1` | `career.resume_normalization.v1` |
| `resume enrich` | `career.resume_enrichment_input.v1` | `career.resume_enrichment_result.v1` |
| `resume variant-review` | `career.resume_variant_review_input.v1` | `career.resume_variant_review.v1` |
| `resume variant-materialize` | `career.resume_variant_materialization_input.v1` | `career.resume_variant.v1` |
| `job normalize` | `career.job_input.v1` | `career.job_normalization.v1` |
| `job match` | `career.job_match_input.v1` | `career.job_match.v1` |

Discover schemas without a source checkout or network connection:

```bash
career schema list --format json-compact
career schema export --id career.job_match_input.v1 > job-match-input.schema.json
```

Schemas are reviewed repository files embedded into the binary at compile time. They are not inferred from command arguments or generated from Rust types at runtime.

## Exit statuses

| Exit | Meaning |
|---:|---|
| `0` | success, help, or version output |
| `2` | invalid command, flag, format, or schema ID |
| `3` | explicit input path/read failure or CLI byte-limit failure |
| `4` | malformed or structurally invalid JSON |
| `5` | valid JSON rejected by core input validation |
| `6` | output write or serialization failure |

Argument errors use canonical pretty JSON because the requested output format may not have parsed. Once arguments parse, JSON errors follow the selected pretty or compact mode; text commands receive concise text errors.

## Input and privacy rules

- Single-document command input is limited to 262,144 bytes before JSON parsing.
- `job match` and resume-variant review/materialization envelopes are limited to 1,048,576 bytes.
- Resume analysis-suggestion review is limited to the ordinary 262,144-byte single-document envelope because it contains one resume and a deliberately small proposal.
- Core character, line, string, list, and evidence limits still apply after parsing.
- Missing paths are not echoed in errors.
- Resume and job text is never logged or repeated in diagnostics.
- Input text is data and cannot change command behavior.
- No command fetches URLs, invokes a provider, reads credentials, or mutates its input.

See the versioned files under [`schemas/`](../schemas/), scoring rules under [`docs/contracts/`](contracts/), and safe agent examples in [`agent-usage.md`](agent-usage.md).
