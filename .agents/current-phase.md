# Current phase

## Most recently completed phase

**Phase 0 — Repository foundation and agent discovery**

## Status

Complete locally. Phase 1 has not started.

A GitHub remote has not been created, so the committed workflow has not yet run on GitHub Actions. Its equivalent commands pass from a clean local clone.

## Implemented

- Root Rust package named `career-core`.
- Workspace CLI package producing the `career` executable.
- Versioned capability discovery with JSON as the default output.
- Truthful available/planned capability statuses.
- `unsafe` forbidden in current production Rust crates.
- Dual MIT/Apache-2.0 license files and package metadata.
- Root `AGENTS.md` and detailed `.agents/` phased handbook.
- Agent Skills-compatible `.agents/skills/career-core/SKILL.md`.
- Public README, contribution, security, changelog, schema, Dependabot, pull-request, and CI documents.
- Committed `Cargo.lock` and reproducible locked builds.

## Verification completed

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features --locked` — 6 tests passed
- `cargo build --workspace --all-features --locked`
- `cargo run --quiet --locked -p career-cli -- capabilities` — valid JSON
- `cargo run --quiet --locked -p career-cli -- capabilities --format text`
- clean-clone repetition of formatting, Clippy, tests, build, and JSON smoke checks
- `cargo install --path crates/career-cli --locked` into an isolated prefix
- capability schema JSON parsing
- Agent Skill frontmatter/name/description validation
- repository credential-pattern scan
- `git diff --check`

## Explicitly unavailable

- resume normalization and evaluation
- job-description normalization
- resume-to-job matching
- PDF/DOCX ingestion
- LLM or network behavior
- Swift bindings
- MCP or provider-specific agent adapters

These remain reported as planned, not available.

## Next phase after approval

Phase 1 will define bounded versioned resume contracts and deliver one small deterministic section-coverage evaluation through the library and JSON CLI. Do not begin Phase 1 until the maintainer explicitly requests it.
