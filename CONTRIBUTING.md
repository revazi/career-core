# Contributing

Thank you for helping build `career-core`.

## Scope

The project is a deterministic, local-first Rust library. Keep LLM/provider calls, API keys, prompts, UI, persistence, remote fetching, billing, and platform-specific behavior outside the core package. Provider-neutral external proposals are untrusted input and must preserve the deterministic baseline.

Read [`AGENTS.md`](AGENTS.md) and the active phase in [`.agents/current-phase.md`](.agents/current-phase.md) before making changes.

## Setup

Install Rust 1.85 or newer with `rustfmt` and Clippy:

```bash
rustup toolchain install stable --component rustfmt,clippy
rustup default stable
```

## Verification

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features --locked
cargo run --quiet -p career-cli -- capabilities
cargo run --quiet -p career-cli -- resume evaluate \
  --input fixtures/resume/phase1/complete-sections.input.json
cargo run --quiet -p career-cli -- resume normalize \
  --input fixtures/resume/phase2/complete-normalization.input.json
cargo run --quiet -p career-cli -- resume enrich \
  --input fixtures/resume/phase2/messy-unlabeled.enrichment-input.json
git diff --check
```

## Contributions

- Keep each pull request limited to one phase-sized task.
- Add tests before or with behavior changes.
- Treat JSON fields, enum values, score rules, evidence identifiers, and exit codes as contracts.
- Explain compatibility impact for every contract change.
- Do not add real resumes, personal data, API keys, or proprietary job descriptions to fixtures.
- Record source provenance for fixtures adapted from the sibling Django reference repository.
- Do not add dependencies without documenting why the standard library and existing dependencies are insufficient.

## Commits and pull requests

Use focused `feature/`, `fix/`, or `chore/` branches and concise imperative commit messages. Wait for all CI checks and explicit approval, then squash merge. Do not publish crates or create releases from an unreviewed branch.
