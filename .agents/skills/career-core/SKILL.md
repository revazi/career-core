---
name: career-core
description: Discovers and invokes the local career-core deterministic CLI for source-grounded normalization, resume readiness analysis, conservative resume-to-job matching, and optional external-proposal validation. Use when evaluating supported career documents, inspecting evidence, or building local integrations.
license: MIT OR Apache-2.0
compatibility: Requires an installed `career` binary or a career-core source checkout with Rust 1.85+.
metadata:
  author: revazi
  version: "0.1.0"
---

# career-core

Use the local `career` CLI as the authoritative deterministic tool. Do not recreate its scoring in prompts or infer support for planned capabilities.

## Discover available behavior

If `career` is installed:

```bash
career capabilities
```

From this repository:

```bash
cargo run --quiet -p career-cli -- capabilities
```

JSON is the default. Invoke only entries whose `status` is `available`. `core.capabilities`, `resume.evaluate`, `resume.analyze`, `resume.normalize`, provider-neutral `resume.enrich`, `job.normalize`, and `job.match` are available.

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

- Prefer `json-compact` or canonical JSON output for agent decisions.
- Discover exact contracts with `career schema list` and `career schema export`; do not infer JSON shape from prose.
- Treat stdout as machine output and stderr as diagnostics.
- Do not send resume or job content to external services without explicit user approval.
- Never place API keys in CLI input, output, files, or logs.
- Preserve uncertainty, source provenance, basis check IDs, and warnings in any explanation.
- Treat `baseline` as authoritative and external proposal fields as assisted only.
- Do not turn planned capabilities into fabricated results.
- Do not modify source documents unless the user separately requests and approves a change.
- When developing this repository, read `AGENTS.md` and `.agents/current-phase.md` before editing.

See [`references/cli-contract.md`](references/cli-contract.md) for the current process contract.
