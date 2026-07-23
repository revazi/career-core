## Scope

Describe the active phase, requested behavior, and explicit non-scope.

## Contracts and compatibility

Describe JSON schema, CLI, score semantics, evidence identifiers, or compatibility impact. Write `None` when unchanged.

## Reference provenance

List any files, policy versions, and fixtures consulted in `../resume-ai`. Write `None` when not applicable.

## Verification

List exact commands and results.

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo build --workspace --all-features --locked`
- [ ] `cargo run --quiet -p career-cli -- capabilities`
- [ ] `git diff --check`

## Security and privacy

Describe input-boundary, payload-logging, network, dependency, and sensitive-data impact.

## Decisions and tradeoffs

Document intentional differences, dependencies, limitations, and deferred work.
