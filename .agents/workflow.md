# Development workflow

## Before implementation

1. Confirm the user requested a file change.
2. Synchronize `main` and create a focused branch.
3. Read `AGENTS.md`, `current-phase.md`, architecture, and the active phase.
4. State a short plan and the simplest sensible interpretation.
5. Identify contract, compatibility, security, and reference-fixture impact.

## Implementation loop

1. Add or select the smallest failing unit/golden test.
2. Implement pure core behavior before adapter behavior.
3. Run the narrow test while iterating.
4. Run formatting and Clippy before broad tests.
5. Update capability status only when the operation is fully implemented and its acceptance gate passes.
6. Update schemas, examples, changelog, and agent skill together with public contract changes.
7. Update `current-phase.md` with actual verification, not planned results.

## Django-reference loop

When porting behavior:

1. Read the exact source and tests listed in `reference-map.md`.
2. Record the source policy/version and fixture provenance.
3. Reduce framework-specific state to explicit Rust input.
4. Port one behavior cluster, not an entire module blindly.
5. Verify with Rust-owned golden fixtures.
6. Document intentional differences.
7. Leave `../resume-ai` unchanged.

## Verification ladder

During iteration:

```bash
cargo test -p career-core <test-name>
cargo test -p career-cli <test-name>
```

Before review:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features --locked
cargo run --quiet -p career-cli -- capabilities
cargo run --quiet -p career-cli -- resume evaluate \
  --input fixtures/resume/phase1/complete-sections.input.json
cargo run --quiet -p career-cli -- resume analyze \
  --input fixtures/resume/phase3/complete-analysis.input.json
cargo run --quiet -p career-cli -- resume normalize \
  --input fixtures/resume/phase2/complete-normalization.input.json
cargo run --quiet -p career-cli -- resume enrich \
  --input fixtures/resume/phase2/messy-unlabeled.enrichment-input.json
git diff --check
```

If the CLI contract changes, also invoke every changed command in JSON and text modes and parse JSON with an independent parser.

## Dependency policy

A new dependency requires all of:

- concrete active-phase need
- maintenance and license review
- explanation in the pull request
- no simpler standard-library or existing-dependency solution
- placement in the narrowest package that needs it

Core dependencies face a higher bar than adapter dependencies. No dependency may add telemetry or implicit network behavior.

## Fixture policy

Fixtures must be synthetic or clearly licensed and contain no real personal data. Every adapted fixture records:

- source repository/path
- source policy version if relevant
- modifications made
- expected behavior

Keep fixtures small enough to review. Add adversarial fixtures for ambiguity, oversized input, prompt-like text, Unicode, malformed structures, and close-but-non-equivalent skills as their phases begin.

## Pull-request report

Include:

- active phase and acceptance criterion
- scope and explicit non-scope
- public contract/schema impact
- reference files and fixtures used
- dependencies added or removed
- security/privacy impact
- exact verification results
- known differences and follow-up work

Do not mark a phase complete until all acceptance checks are present and green.
