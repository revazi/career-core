---
name: career-core
description: Discovers and invokes the local career-core deterministic resume CLI, including source-grounded normalization and optional external-proposal validation. Use when evaluating supported career documents, inspecting evidence, or building local integrations.
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

JSON is the default. Invoke only entries whose `status` is `available`. `core.capabilities`, `resume.evaluate`, `resume.normalize`, and provider-neutral `resume.enrich` are available; job matching remains planned.

For human-readable discovery:

```bash
cargo run --quiet -p career-cli -- capabilities --format text
```

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

## Rules

- Prefer JSON output for agent decisions.
- Treat stdout as machine output and stderr as diagnostics.
- Do not send resume or job content to external services without explicit user approval.
- Never place API keys in CLI input, output, files, or logs.
- Preserve uncertainty, source provenance, and warnings in any explanation.
- Treat `baseline` as authoritative and external proposal fields as assisted only.
- Do not turn planned capabilities into fabricated results.
- Do not modify source documents unless the user separately requests and approves a change.
- When developing this repository, read `AGENTS.md` and `.agents/current-phase.md` before editing.

See [`references/cli-contract.md`](references/cli-contract.md) for the current process contract.
