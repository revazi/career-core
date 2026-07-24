# career-core

`career-core` is an open-source Rust library for deterministic, explainable resume evaluation and resume-to-job matching.

The project is intentionally **not an AI service**. The core performs no network requests and has no LLM dependency. Native applications, command-line tools, and coding-agent integrations can use its versioned evidence and scoring contracts. An opted-in host may submit an external source-grounded proposal for deterministic validation, but provider calls remain outside the authoritative core.

> **Status:** Phase 5 is complete. Deterministic resume/job operations, additive compact output, offline schema discovery, and the source-installation path are available. Release publication and Swift bindings remain separately gated.

## Goals

- deterministic output for identical versioned input
- explainable scores with bounded evidence and warnings
- conservative matching that prefers false negatives over unsafe equivalence
- stable JSON contracts suitable for CLIs and coding agents
- a portable Rust library suitable for future Swift bindings
- local-first operation with no telemetry or implicit network access

## Repository layout

```text
.
├── src/                         career-core library package
├── crates/career-cli/           `career` command-line adapter
├── schemas/                     versioned public JSON schemas
├── fixtures/                    synthetic reviewed golden contracts
├── docs/contracts/              public scoring and limit rules
├── .agents/                     project and agent-integration handbook
├── AGENTS.md                    instructions loaded by coding agents
├── LICENSE-MIT
└── LICENSE-APACHE
```

The repository root is the `career-core` library package. The CLI is a separate workspace package so UI and argument-parsing dependencies do not leak into the core library.

## Requirements

- Rust 1.85 or newer

## Build and verify

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features --locked
```

Install the local CLI from a reviewed checkout:

```bash
cargo install --path crates/career-cli --locked
career capabilities --format json-compact
```

No release binaries or package-manager channels are published yet. See [`docs/distribution.md`](docs/distribution.md) for source installation and future checksum verification.

## Coding-agent discovery

Run the CLI from the repository:

```bash
cargo run --quiet -p career-cli -- capabilities
```

JSON is the default output:

```json
{
  "schema_version": "career.capabilities.v1",
  "core_version": "0.1.0",
  "deterministic": true,
  "performs_network_requests": false,
  "capabilities": [
    {
      "id": "core.capabilities",
      "status": "available",
      "summary": "Discover versioned functionality exposed by this build."
    },
    {
      "id": "resume.evaluate",
      "status": "available",
      "summary": "Evaluate recognized resume section coverage with explainable deterministic checks."
    },
    {
      "id": "resume.analyze",
      "status": "available",
      "summary": "Analyze resume readiness with 18 explainable deterministic checks and confidence-aware evidence."
    },
    {
      "id": "resume.normalize",
      "status": "available",
      "summary": "Normalize bounded resume text into source-grounded deterministic facts and confidence."
    },
    {
      "id": "resume.enrich",
      "status": "available",
      "summary": "Validate and conservatively merge an explicit source-grounded external proposal without network access."
    },
    {
      "id": "job.normalize",
      "status": "available",
      "summary": "Normalize bounded job-description text into source-grounded deterministic facts and confidence."
    },
    {
      "id": "job.match",
      "status": "available",
      "summary": "Match deterministic resume and job baselines with conservative equivalence and confidence-aware evidence."
    }
  ]
}
```

Callers must only invoke entries whose `status` is `available`.

For concise human output:

```bash
cargo run --quiet -p career-cli -- capabilities --format text
```

Discover and export exact Draft 2020-12 contracts from the installed binary without network or filesystem lookup:

```bash
career schema list --format json-compact
career schema export --id career.job_match_input.v1
```

`--format json` remains canonical pretty output. `json-pretty` selects it explicitly, `json-compact` emits one JSON document on one line, and `text` is for human display. See [`docs/cli.md`](docs/cli.md) for the stable hierarchy, contract map, stream rules, formats, input limits, and exit statuses.

The project includes an Agent Skills-compatible guide at `.agents/skills/career-core/SKILL.md`. Pi discovers that skill after the repository is trusted. Other coding agents can read the same file or invoke the CLI directly. Shell-safe Pi, Claude Code, Codex, and generic subprocess examples are documented in [`docs/agent-usage.md`](docs/agent-usage.md).

## Evaluate resume section coverage

Provide versioned JSON containing text already extracted by the caller:

```json
{
  "schema_version": "career.resume_input.v1",
  "text": "SUMMARY\nBackend engineer.\nEXPERIENCE\nBuilt reliable services.\nEDUCATION\nExample University\nSKILLS\nRust, SQL",
  "metadata": {
    "document_id": "optional-caller-id"
  }
}
```

Evaluate a file:

```bash
cargo run --quiet -p career-cli -- \
  resume evaluate --input fixtures/resume/phase1/complete-sections.input.json
```

Or pipe JSON through stdin:

```bash
cat fixtures/resume/phase1/complete-sections.input.json | \
  cargo run --quiet -p career-cli -- resume evaluate --input -
```

Add `--format text` for concise human output. JSON results use `career.resume_evaluation.v1`; invalid input produces `career.error.v1` on stderr with a nonzero exit code.

Phase 1 scores only whether four recognized core headers have following content. It does **not** claim to measure complete resume quality or ATS compatibility. See [`docs/contracts/resume-evaluation-v1.md`](docs/contracts/resume-evaluation-v1.md) for limits, aliases, scoring, evidence, warnings, and provenance.

## Analyze resume readiness

Run the full deterministic scoring policy without changing the Phase 1 contract:

```bash
cargo run --quiet -p career-cli -- \
  resume analyze --input fixtures/resume/phase3/complete-analysis.input.json
```

`career.resume_analysis.v1` reports 18 bounded checks across ATS-readability signals, content strength, experience impact, skills coverage, presentation, and completeness. It includes raw and confidence-adjusted scores, source-grounded evidence, provisional findings, deterministic improvement actions, and explicit limitations.

This is a general text-based readiness analysis. It does not reproduce proprietary ATS rankings, inspect visual document layout, or guarantee hiring outcomes. Scores always consume the deterministic normalization baseline; assisted fields cannot alter them. See [`docs/contracts/resume-analysis-v1.md`](docs/contracts/resume-analysis-v1.md) for the complete policy and `deterministic_v2` compatibility matrix.

## Normalize resumes

Normalize caller-extracted text deterministically:

```bash
cargo run --quiet -p career-cli -- \
  resume normalize --input fixtures/resume/phase2/complete-normalization.input.json
```

The result includes source-grounded contact, summary, experience, education, skills, projects, and certifications; parser confidence; field statuses; warnings; and optional external-enrichment eligibility.

If an opted-in agent or application obtains an exact external proposal, validate and merge it without a provider call:

```bash
cargo run --quiet -p career-cli -- \
  resume enrich --input fixtures/resume/phase2/messy-unlabeled.enrichment-input.json
```

The enrichment result preserves the complete deterministic `baseline` and exposes accepted values only in `assisted_document`. Assisted fields never replace deterministic confidence or authoritative scoring input. See [`docs/contracts/resume-normalization-v1.md`](docs/contracts/resume-normalization-v1.md) for contracts, limits, grounding rules, host orchestration, provenance, and intentional Django differences.

## Normalize job descriptions

Normalize caller-supplied vacancy text without fetching a URL:

```bash
cargo run --quiet -p career-cli -- \
  job normalize --input fixtures/job/phase4a/complete-normalization.input.json
```

`career.job_normalization.v1` returns source-grounded required/preferred skills and qualifications, responsibilities, seniority, experience, education, and certification signals; six-signal parse confidence; field statuses; matched/unmatched metadata; and bounded warnings.

A field reported as `not_detected` is not confirmed absent. Low-confidence output must remain provisional. See [`docs/contracts/job-normalization-v1.md`](docs/contracts/job-normalization-v1.md) for exact aliases, classification rules, limits, confidence, provenance, and intentional reference differences.

## Match a resume to a job

Match original resume and job-description inputs through independently reproduced deterministic normalization baselines:

```bash
cargo run --quiet -p career-cli -- \
  job match --input fixtures/job/phase4b/complete-match.input.json
```

`career.job_match.v1` reports six bounded categories, raw and confidence-adjusted scores, exact or reviewed same-technology skill matches, source/derived evidence, confirmed strengths, partial/likely/unverified gaps, and a deterministically gated recommendation.

Low/unknown normalization bounds every category to 50–75, suppresses broad inferred gaps, and makes guidance provisional. Related technologies such as Kubernetes/Docker, PostgreSQL/MySQL, React/Angular, AWS/Azure, and Django/Flask remain non-equivalent. Matching never consumes assisted resume fields, invokes a provider, fetches a URL, or predicts a hiring outcome. See [`docs/contracts/job-match-v1.md`](docs/contracts/job-match-v1.md) for the full scoring, equivalence, confidence, evidence, recommendation, bounds, and reference-parity policy.

## Reference implementation

The sibling Django repository at `../resume-ai` is a read-only behavioral reference during the port. It contains mature deterministic normalization, scoring, confidence, matching, fixtures, and regression tests. `career-core` must not import it, execute it at runtime, or claim parity until Rust golden tests prove the behavior.

See [`.agents/reference-map.md`](.agents/reference-map.md) for the bounded reference map.

## Roadmap

The detailed, gated roadmap lives in [`.agents/phases.md`](.agents/phases.md). The broad order is:

1. versioned contracts and a minimal resume-evaluation vertical slice
2. deterministic resume normalization and provider-neutral assisted validation
3. explainable resume scoring parity and general ATS-readiness analysis
4. job-description normalization, followed by conservative matching
5. hardened CLI and coding-agent distribution
6. Swift bindings for a later SwiftUI application
7. optional adapters only when concrete consumers require them

## Security and privacy

The core is designed to process sensitive career documents locally. It must not add telemetry, remote fetching, hidden model calls, or payload logging. External enrichment is explicit and host-controlled: the host owns consent, keys, prompts, and network calls, while the core validates only the submitted proposal. See [`SECURITY.md`](SECURITY.md) and [`.agents/architecture.md`](.agents/architecture.md).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Coding agents must also follow [`AGENTS.md`](AGENTS.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
