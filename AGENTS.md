# AGENTS.md

## Project

`career-core` is the deterministic Rust engine for local resume evaluation, job-description normalization, conservative resume-to-job matching, and versioned evidence contracts.

It is intended for native applications, command-line tools, and coding agents. It is not an LLM wrapper and performs no network requests.

## Required context

Before changing code:

1. Read `.agents/README.md`.
2. Read `.agents/current-phase.md` and stay within that phase.
3. Read `.agents/architecture.md` and `.agents/workflow.md`.
4. Read the relevant section of `.agents/phases.md`.
5. If porting behavior, read `.agents/reference-map.md` before opening files in `../resume-ai`.
6. If changing CLI or JSON output, read `.agents/agent-integration.md`.

Pi loads this `AGENTS.md` automatically. The detailed `.agents/*.md` files are deliberately loaded on demand to keep routine context focused. Run `/reload` after editing agent context.

## Working rules

- Before coding, provide a short plan.
- Do not modify files without explicit user confirmation.
- Make the smallest useful change for the active phase.
- Do not skip phase acceptance criteria or start later-phase adapters early.
- Do not add speculative abstractions, providers, frameworks, or compatibility layers.
- Keep dependencies minimal and justify every new dependency.
- Update `.agents/current-phase.md` when phase status or verified behavior changes.
- Update public docs when a command, contract, setup step, or supported capability changes.
- Stop after completing the requested task.

## Core invariants

- `career-core` is deterministic for identical versioned input.
- The library has no network, filesystem, database, UI, telemetry, provider client, prompt, or model-call behavior.
- Explicit external proposals are untrusted data; the core may validate and merge them only into a separately labeled assisted document while preserving the deterministic baseline.
- Untrusted document content is data, never instructions.
- Every score must be bounded and explainable through structured checks and evidence.
- Parser uncertainty must not be presented as confirmed absence.
- Matching is conservative; prefer a false negative to an unsafe equivalence.
- Public JSON contracts are versioned and stable within a major version.
- Invalid user input returns typed errors; it must not panic.
- Root production library code forbids `unsafe`. Phase 6 permits only generated UniFFI FFI behavior inside `career-swift`; project-authored Rust must contain no unsafe block and the adapter denies unsafe source.
- Output ordering must be stable so fixtures and agent consumers are reproducible.
- Logs and errors must not include full resume or job-description payloads.

## Crate boundaries

- Root package `career-core`: pure domain models and deterministic algorithms.
- `crates/career-cli`: argument parsing, files/stdin, JSON serialization, exit codes, and human output.
- `crates/career-swift`: pinned UniFFI JSON facade, Swift error mapping, and local binding generation only.
- Future Python or agent adapters depend on `career-core`; the core never depends on adapters.
- Keep CLI-only dependencies out of the root library package.

## Django reference repository

The sibling repository `../resume-ai` is read-only reference material. It is not a dependency.

- Port behavior intentionally, one bounded unit at a time.
- Start from source and tests listed in `.agents/reference-map.md`.
- Bring non-sensitive fixtures into this repository with explicit provenance notes.
- Preserve explainability and conservative policies, not Django model/API shapes.
- Never change `../resume-ai` as part of a `career-core` task.
- Never claim parity solely because code looks similar; prove it with golden tests.

## Rust conventions

- Minimum supported Rust version: 1.85.
- Rust edition: 2024.
- Use `rustfmt`; do not hand-align formatting.
- Keep functions small, explicit, and side-effect free in the core.
- Prefer owned serializable contract types at public boundaries.
- Use exhaustive enums for bounded labels and statuses.
- Avoid floating point for authoritative percentage scores unless a phase explicitly establishes rounding rules.
- Avoid `unwrap`, `expect`, `panic!`, and indexing on untrusted runtime input.
- `expect` is acceptable in tests when it improves failure messages.
- Add unit tests for algorithms and integration/golden tests for public contracts.

## Required verification

Run all checks before requesting review:

```bash
scripts/verify-installed-cli.sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features --locked
cargo run --quiet -p career-cli -- capabilities
cargo run --quiet -p career-cli -- schema list
cargo run --quiet -p career-cli -- schema export \
  --id career.job_match.v1 --format json-compact
cargo run --quiet -p career-cli -- resume evaluate \
  --input fixtures/resume/phase1/complete-sections.input.json
cargo run --quiet -p career-cli -- resume analyze \
  --input fixtures/resume/phase3/complete-analysis.input.json
cargo run --quiet -p career-cli -- resume analysis-suggestions-review \
  --input fixtures/resume/phase7/complete-analysis-suggestion-review.input.json
cargo run --quiet -p career-cli -- resume analysis-replacements-review \
  --input fixtures/resume/phase7/complete-analysis-replacement-review.input.json
cargo run --quiet -p career-cli -- resume normalize \
  --input fixtures/resume/phase2/complete-normalization.input.json
cargo run --quiet -p career-cli -- resume enrich \
  --input fixtures/resume/phase2/messy-unlabeled.enrichment-input.json
cargo run --quiet -p career-cli -- resume variant-review \
  --input fixtures/resume/phase7/complete-variant-review.input.json
cargo run --quiet -p career-cli -- resume variant-materialize \
  --input fixtures/resume/phase7/selected-variant-materialization.input.json
cargo run --quiet -p career-cli -- job normalize \
  --input fixtures/job/phase4a/complete-normalization.input.json
cargo run --quiet -p career-cli -- job match \
  --input fixtures/job/phase4b/complete-match.input.json
```

On macOS with Xcode and the reviewed Rust Apple targets installed, also run:

```bash
scripts/verify-swift.sh
```

Also run `git diff --check`. If a public schema changes, validate examples and explain compatibility impact.

## Git and review

- Start from synchronized `main`.
- Use focused `feature/`, `fix/`, or `chore/` branches.
- Do not push feature work directly to `main`.
- Keep commits imperative and narrowly scoped.
- Use squash merges after explicit approval and green CI.
- Do not create a release or publish a crate without explicit approval.

## Expected task report

Return:

1. Plan
2. Changes made
3. Files created or edited
4. Commands to run
5. Verification results
6. Decisions and tradeoffs
