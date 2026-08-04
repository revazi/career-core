# Contributing

Thank you for helping build `career-core`.

## Scope

The project is a deterministic, local-first Rust library with narrow CLI and Swift adapters. Keep LLM/provider calls, API keys, prompts, UI, persistence, remote fetching, billing, and platform-specific behavior outside the core package. Provider-neutral external proposals are untrusted input and must preserve the deterministic baseline.

Read [`AGENTS.md`](AGENTS.md) and the active phase in [`.agents/current-phase.md`](.agents/current-phase.md) before making changes.

## Setup

Install Rust 1.85 or newer with `rustfmt` and Clippy:

```bash
rustup toolchain install stable --component rustfmt,clippy
rustup default stable
```

## Verification

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
git diff --check
```

Swift-boundary changes additionally require a full Xcode installation and:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
scripts/verify-swift.sh
```

The Swift scripts resolve Cargo and rustc through rustup's active toolchain even
when another installation appears first in `PATH`. To select another installed
toolchain intentionally, use the standard rustup variable for both target
installation and verification:

```bash
RUSTUP_TOOLCHAIN=stable rustup target add aarch64-apple-ios aarch64-apple-ios-sim
RUSTUP_TOOLCHAIN=stable scripts/verify-swift.sh
```

## Contributions

- Keep each pull request limited to one phase-sized task.
- Add tests before or with behavior changes.
- Treat JSON fields, enum values, score rules, evidence identifiers, schema catalog entries, output formats, and exit codes as contracts.
- Explain compatibility impact for every contract change.
- Do not add real resumes, personal data, API keys, or proprietary job descriptions to fixtures.
- Record source provenance for fixtures adapted from the sibling Django reference repository.
- Do not add dependencies without documenting why the standard library and existing dependencies are insufficient.
- For FFI dependencies, document MSRV, generated code, unsafe isolation, licenses, artifact targets, and checksum impact.

## Commits and pull requests

Use focused `feature/`, `fix/`, or `chore/` branches and concise imperative commit messages. Wait for all CI checks and explicit approval, then squash merge. Do not publish crates or create releases from an unreviewed branch.
