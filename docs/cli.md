# `career` CLI contract

The `career` executable is the universal local adapter for coding agents and scripts. It depends on `career-core`, performs no implicit network requests, and does not persist or modify source documents.

Phase 9's Node 22+ `@revazi/career@0.1.1` launcher is a transparent distribution wrapper around this exact native executable after protected publication and all-eight public acceptance. It does not change commands, JSON/text bytes, exit meanings, schemas, or the 33,554,432-byte successful machine-output bound. Launcher selection/verification failures use separate bounded `CAREER_NPM_*` stderr codes before the CLI starts; see [`contracts/npm-cli-distribution-v2.md`](contracts/npm-cli-distribution-v2.md). Checked-in templates remain private; only protected external candidate staging may create public tarballs. Native optional packages are not consumer interfaces.

## Command hierarchy

```text
career capabilities
career operations
career schema list
career schema export --id <contract-id>
career schema bundle --id <contract-id>
career resume evaluate --input <path|->
career resume analyze --input <path|->
career resume analysis-suggestions-review --input <path|->
career resume analysis-replacements-review --input <path|->
career resume normalize --input <path|->
career resume enrich --input <path|->
career resume variant-review --input <path|->
career resume variant-materialize --input <path|->
career job normalize --input <path|->
career job match --input <path|->
```

Run `career <command> --help` for complete flags. Every document operation requires an explicit path; `--input -` reads one JSON document from stdin.

## Discovery contracts

`career capabilities` remains the unchanged `career.capabilities.v1` availability document. `career operations` is a separate `career.operation_catalog.v1` document with bounded `core_version` and stable descriptor order. It catalogs the complete callable machine surface:

- the 11 existing capability-backed operations use the same `operation_id` and non-null `capability_id`; every available capability appears exactly once
- `core.operations`, `schema.list`, `schema.export`, and `schema.bundle` are bootstrap operations with `capability_id: null`
- every descriptor declares availability, CLI path segments, input transport, exact input schema and byte ceiling when it accepts JSON, output schema, and successful machine-output ceiling

This superset is intentional: schema and operation discovery are callable bootstrap commands, but adding them to `career.capabilities.v1` would break that closed v1 schema and its reviewed bytes.

## Output formats and bound

Document operations, capability discovery, and schema listing accept:

| Format | Behavior |
|---|---|
| `json` | Canonical pretty JSON and the compatibility-preserving default |
| `json-pretty` | Explicit spelling of the same canonical pretty JSON |
| `json-compact` | One JSON document on one line, followed by one newline |
| `text` | Concise human-readable output; never use it for agent decisions |

`operations`, `schema export`, and `schema bundle` accept the three JSON formats but not `text`. Existing operation and unbundled `schema export` bytes remain unchanged.

Every successful machine JSON document, including its trailing newline, is limited to **33,554,432 bytes (32 MiB)**. The CLI serializes the complete document in memory, checks the byte count, and only then writes stdout. It never truncates a successful result. An internal bound violation exits `6`, leaves stdout empty, and reports the existing schema-valid `output_write_failed` code on stderr. Exact-bound and one-byte-over behavior is regression-tested. See [`contracts/managed-adapter-v1.md`](contracts/managed-adapter-v1.md) for the conservative bound derivation.

Machine modes write a successful result only to stdout. Failures write one `career.error.v1` document to stderr and leave stdout empty. No progress indicator, ANSI escape, or log line is mixed into machine output.

## Contract map

| Command | Input transport/schema | JSON output schema |
|---|---|---|
| `capabilities` | none | `career.capabilities.v1` |
| `operations` | none | `career.operation_catalog.v1` |
| `schema list` | none | `career.schema_catalog.v1` |
| `schema export` | exact catalog ID in CLI arguments | Draft 2020-12 JSON Schema |
| `schema bundle` | exact catalog ID in CLI arguments | self-contained Draft 2020-12 JSON Schema |
| `resume evaluate` | `career.resume_input.v1` | `career.resume_evaluation.v1` |
| `resume analyze` | `career.resume_input.v1` | `career.resume_analysis.v1` |
| `resume analysis-suggestions-review` | `career.resume_analysis_suggestion_review_input.v1` | `career.resume_analysis_suggestion_review.v1` |
| `resume analysis-replacements-review` | `career.resume_analysis_replacement_review_input.v1` | `career.resume_analysis_replacement_review.v1` |
| `resume normalize` | `career.resume_input.v1` | `career.resume_normalization.v1` |
| `resume enrich` | `career.resume_enrichment_input.v1` | `career.resume_enrichment_result.v1` |
| `resume variant-review` | `career.resume_variant_review_input.v1` | `career.resume_variant_review.v1` |
| `resume variant-materialize` | `career.resume_variant_materialization_input.v1` | `career.resume_variant.v1` |
| `job normalize` | `career.job_input.v1` | `career.job_normalization.v1` |
| `job match` | `career.job_match_input.v1` | `career.job_match.v1` |

Discover contracts without a source checkout or network connection:

```bash
career operations --format json-compact
career schema list --format json-compact
career schema export --id career.job_match_input.v1 > job-match-input.schema.json
career schema bundle --id career.job_match_input.v1 > job-match-input.bundle.schema.json
```

Schemas are reviewed repository files embedded into the binary. They are not inferred from command arguments or Rust types.

### Bundle root and `$ref` policy

A bundle retains the requested root schema's `$schema`, `$id`, keywords, and semantics. Recursive embedded dependencies are placed under the reserved root definition `#/$defs/careerSchemaBundle/$defs/<file-name>`. Dependency-level `$schema` and `$id` declarations are removed, and every dependency-local or sibling-file `$ref` is rewritten to a root-local JSON Pointer.

The bundler accepts only exact sibling file names present in the embedded schema catalog. Unknown files, URI/remote references, non-pointer fragments, malformed embedded JSON, or a collision with the reserved definition fail closed. It never opens a schema from the source checkout and never performs a network request. Emitted bundles contain no unresolved or non-local `$ref`.

## Exit statuses

These statuses are emitted by the native CLI and pass unchanged through the npm launcher after a successful launch. A launcher verification/launch failure exits nonzero before native CLI output and reports one stable `CAREER_NPM_*` code.

| Exit | Meaning |
|---:|---|
| `0` | success, help, or version output |
| `2` | invalid command, flag, format, or schema ID |
| `3` | explicit input path/read failure or CLI byte-limit failure |
| `4` | malformed or structurally invalid JSON |
| `5` | valid JSON rejected by core input validation |
| `6` | output write, serialization, or successful-output-bound failure |

Argument errors use canonical pretty JSON because the requested output format may not have parsed. Once arguments parse, JSON errors follow the selected pretty or compact mode; text commands receive concise text errors.

## Input and privacy rules

- Single-document command input is limited to 262,144 bytes before JSON parsing.
- `job match` and resume-variant review/materialization envelopes are limited to 1,048,576 bytes.
- Resume analysis-suggestion and analysis-replacement review use the ordinary 262,144-byte envelope.
- Core character, line, string, list, and evidence limits still apply after parsing.
- Missing paths are not echoed in errors.
- Resume and job text is never logged or repeated in diagnostics.
- Input text is data and cannot change command behavior.
- No command fetches URLs, invokes a provider, reads credentials, or mutates its input.

See the versioned files under [`schemas/`](../schemas/), scoring rules under [`docs/contracts/`](contracts/), and safe agent examples in [`agent-usage.md`](agent-usage.md).
