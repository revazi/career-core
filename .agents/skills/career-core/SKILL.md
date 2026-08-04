---
name: career-core
description: Discovers and invokes the local career-core deterministic CLI for source-grounded normalization, resume readiness analysis, reviewed external suggestions and exact replacement diffs, conservative resume-to-job matching, optional external-proposal validation, and selected assisted-variant materialization. Use when evaluating supported career documents, inspecting evidence, or building local integrations.
license: MIT OR Apache-2.0
compatibility: Requires an installed `career` binary or a career-core source checkout with Rust 1.85+; native Pi tools require the reviewed local Pi package.
metadata:
  author: revazi
  version: "0.1.0"
---

# career-core

Use the local `career` CLI as the authoritative deterministic tool. The native Pi extension is only a bounded process adapter around that installed binary. Do not recreate scoring, schemas, or repair behavior in TypeScript or prompts, and do not infer support for planned capabilities.

## Privacy decision before document use

Pi may persist tool arguments and results in session JSONL even though Career Core and its extension do not persist career documents. Before placing private resume or job content in a Pi tool call, require the user to make an explicit session-persistence decision. Recommend starting a new transient run:

```bash
pi --no-session
```

`--no-session` prevents normal Pi session saving for that run; do not claim secure erasure or that it removes copies already stored elsewhere. If the user does not approve placing private content in Pi context, do not call a document tool.

## Install local interfaces

From a reviewed checkout, install the CLI without a remote shell installer:

```bash
cargo install --path crates/career-cli --locked
career capabilities --format json-compact
```

To load this repository's native extension and this same canonical skill, register the reviewed local package:

```bash
repository_root="$(pwd -P)"
pi install "$repository_root"
```

The local package points to this skill; it does not copy it. The extension uses `career` from `PATH`, or a bounded absolute `CAREER_CLI_PATH` override. It never invokes Cargo automatically. See [`../../../docs/distribution.md`](../../../docs/distribution.md) for isolated install and uninstall instructions.

## Discover available behavior

If `career` is installed:

```bash
career capabilities
```

From this repository:

```bash
cargo run --quiet -p career-cli -- capabilities
```

JSON is the default. Invoke only entries whose `status` is `available`. `core.capabilities`, `resume.evaluate`, `resume.analyze`, `resume.normalize`, provider-neutral `resume.enrich`, `resume.analysis-suggestions.review`, `resume.analysis-replacements.review`, `resume.variant.review`, `resume.variant.materialize`, `job.normalize`, and `job.match` are available.

For human-readable discovery:

```bash
cargo run --quiet --locked -p career-cli -- capabilities --format text
```

Discover or export exact public contracts from an installed binary without network access:

```bash
career schema list --format json-compact
career schema export --id career.job_match_input.v1
```

Use canonical `json`/`json-pretty` when reviewed indentation matters, `json-compact` for one-line machine transport, and `text` only for human display.

## Native Pi tool surface

When the local Pi package is loaded, prefer discovery first through these stable tools:

- `career_core_discover`: operations `capabilities`, `schema-list`, and `schema-export`
- `career_core_resume`: `evaluate`, `analyze`, `analysis-suggestions-review`, `analysis-replacements-review`, `normalize`, `enrich`, `variant-review`, and `variant-materialize`
- `career_core_job`: `normalize` and `match`

For `career_core_resume` and `career_core_job`, pass exactly one versioned input object serialized as the `input_json` string. Use schemas returned by `career_core_discover`; do not infer the shape from prose. The extension sends those bytes only to installed-CLI stdin with `--input - --format json-compact`, makes no model/network call, and returns complete CLI JSON without semantic transformation.

A `career.pi_error.v1` error is adapter/process status, not a deterministic career result. On `career_cli_error`, inspect only the nested validated bounded `career.error.v1`. Do not retry validation errors without correcting input. On `result_too_large` or `result_too_many_lines`, never use a partial result: ask the user to run the installed `career` CLI directly in a user-approved local workflow that can consume the complete JSON.

If native tools are unavailable, use the direct CLI commands below. Never substitute Cargo automatically from inside the extension.

## Evaluate supported resume text

Prepare `career.resume_input.v1` JSON with a `text` field containing text already extracted from the resume. Then run:

```bash
career resume evaluate --input /path/to/input.json
```

From this repository:

```bash
cargo run --quiet -p career-cli -- resume evaluate --input /path/to/input.json
```

Use `--input -` for stdin and `--format text` only for human display. The JSON result is authoritative for agent decisions.

The Phase 1 score measures recognized Summary, Experience, Education, and Skills header coverage only. Always communicate the `limited_evaluation_scope` warning; never present the score as complete resume quality or ATS compatibility.

## Analyze resume readiness

Use the full deterministic policy when the user asks for resume quality or ATS-readiness analysis:

```bash
career resume analyze --input /path/to/input.json
```

Read `raw_score`, adjusted `score`, `outcome`, evidence, and `basis_check_id` together. Treat `inconclusive` checks and `provisional` findings as unverified. Preserve the general-ATS and visual-layout warnings; never claim a proprietary ATS ranking, interview prediction, or layout inspection.

`resume analyze` independently scores the deterministic normalization baseline. Do not substitute `assisted_document` values. The older `resume evaluate` operation remains section coverage only.

## Review external analysis suggestions

Review no more than three source-targeted external suggestions beside a freshly rerun deterministic analysis:

```bash
career resume analysis-suggestions-review --input /path/to/analysis-suggestion-review-input.json
```

Submit only `career.resume_analysis_suggestion_review_input.v1`. Each suggestion must bind to one current failed canonical action and exact source target/evidence occurrence. Use only the returned canonical suggestions, action status, and discard codes. The output's `baseline_analysis` remains authoritative; retained suggestions are assisted/non-authoritative. Exact occurrence neither verifies generated wording nor certifies a rewrite. Do not treat this review-only operation as candidate generation, selection, source mutation, or materialization.

## Review exact analysis replacements

Review up to three exact source-targeted external replacements for a non-authoritative before/proposed-after diff:

```bash
career resume analysis-replacements-review --input /path/to/analysis-replacement-review-input.json
```

Submit only `career.resume_analysis_replacement_review_input.v1`. Each retained replacement is bound to one current failed canonical action and exact source target/evidence occurrence. Render only its returned canonical `source_target`, `proposed_replacement`, action/status, evidence, and warnings beside `baseline_analysis`. Exact occurrence does not verify generated wording or certify a rewrite. This is review-only: it has no candidate, selection, source mutation, materialization, or export path.

The existing `resume analysis-suggestions-review` v1 `suggestion` field remains advisory text, not a replacement. Do not relabel it or synthesize replacement semantics.

## Normalize and optionally enrich

Run deterministic normalization first:

```bash
career resume normalize --input /path/to/input.json
```

The result includes source-grounded facts, confidence, field statuses, warnings, and `enrichment_request`. If that request is `eligible`, the user has explicitly approved use of the current external model/provider, and exact proposal JSON can be produced, submit a `career.resume_enrichment_input.v1` envelope:

```bash
career resume enrich --input /path/to/enrichment-input.json
```

Populate only the reported target sections and copy every non-empty value from source text. Do not infer or paraphrase. If generation or validation fails, retain the deterministic normalization.

`resume enrich` does not call a model. Its result preserves the authoritative deterministic normalization under `baseline`; `assisted_document` must remain clearly labeled and must not replace deterministic confidence, evidence, or scoring input.

## Review and materialize assisted variants

Review `career.resume_variant_review_input.v1` containing original resume/vacancy inputs and at most 50 untrusted evidence-linked changes:

```bash
career resume variant-review --input /path/to/variant-review-input.json
```

Only canonical retained change identifiers are selectable. Exact evidence occurrence does not certify that generated prose is factually entailed. Do not repair discarded changes or select on the user's behalf.

After explicit user selection, submit the same exact review input plus canonical identifiers through `career.resume_variant_materialization_input.v1`:

```bash
career resume variant-materialize --input /path/to/materialization-input.json
```

The output preserves the baseline and materializes only selected changes as assisted, non-authoritative text. It must not replace the original or enter deterministic analysis/matching.

## Normalize job descriptions

Provide `career.job_input.v1` containing caller-supplied plain text, then run:

```bash
career job normalize --input /path/to/job-input.json
```

Use source-grounded required/preferred fields, confidence, statuses, metadata, and warnings exactly as returned. `not_detected` never confirms absence, especially when confidence is low. Do not fetch URLs implicitly or infer semantic requirements.

## Match a resume to a job

Provide `career.job_match_input.v1` containing original `career.resume_input.v1` and `career.job_input.v1` values:

```bash
career job match --input /path/to/match-input.json
```

Matching independently reruns both deterministic normalizers. It cannot consume `assisted_document` values. Distinguish `raw_score` from confidence-bounded `score`; preserve `confirmed_match`, `partial_match`, `likely_missing`, and `unverified` statuses. Cite source spans when present.

Only `normalized_exact` and reviewed `conservative_alias` skill matches are authoritative. Never substitute adjacent technologies. Low/unknown normalization or truncation makes missing/partial evidence unverified, suppresses broad inferred gaps, bounds categories to 50–75, and prevents `apply_now`.

Recommendation labels are deterministic workflow gates, not hiring predictions. Review generic `unassessed_required_qualifications` rather than inventing support or absence.

## Rules

- Start with `career_core_discover` when native Pi tools are loaded; otherwise start with `career capabilities`.
- Prefer `json-compact` or canonical JSON output for agent decisions.
- Discover exact contracts with `career schema list` and `career schema export`; do not infer JSON shape from prose.
- Treat stdout as machine output and stderr as diagnostics.
- Require a separate explicit decision before placing private resume or job content in Pi session context; recommend a new `pi --no-session` run.
- Do not send resume or job content to external services without explicit user approval; local Pi-session consent is not provider consent.
- Never place API keys in CLI input, output, files, or logs.
- Preserve uncertainty, source provenance, basis check IDs, and warnings in any explanation.
- Treat `baseline` as authoritative and external proposal fields as assisted only.
- Do not turn planned capabilities into fabricated results.
- Do not modify source documents unless the user separately requests and approves a change.
- Treat Core JSON as authoritative; never truncate, reorder, repair, or upgrade its warnings, evidence, uncertainty, baseline boundaries, or assisted/non-authoritative labels.
- When developing this repository, read `AGENTS.md` and `.agents/current-phase.md` before editing.

See [`references/cli-contract.md`](references/cli-contract.md) for the current process contract.
