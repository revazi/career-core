## Scope

Describe the active phase, requested behavior, and explicit non-scope.

## Contracts and compatibility

Describe JSON schema, CLI, score semantics, evidence identifiers, or compatibility impact. Write `None` when unchanged.

## Reference provenance

List any files, policy versions, and fixtures consulted in `../resume-ai`. Write `None` when not applicable.

## Verification

List exact commands and results.

- [ ] `node --test npm/tests/launcher.test.js`
- [ ] `scripts/test-npm-cli-packages.sh`
- [ ] `scripts/test-npm-publication.sh`
- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo build --workspace --all-features --locked`
- [ ] `cargo run --quiet -p career-cli -- capabilities`
- [ ] `cargo run --quiet -p career-cli -- schema list`
- [ ] `cargo run --quiet -p career-cli -- schema export --id career.job_match.v1 --format json-compact`
- [ ] `cargo run --quiet -p career-cli -- resume evaluate --input fixtures/resume/phase1/complete-sections.input.json`
- [ ] `cargo run --quiet -p career-cli -- resume analyze --input fixtures/resume/phase3/complete-analysis.input.json`
- [ ] `cargo run --quiet -p career-cli -- resume normalize --input fixtures/resume/phase2/complete-normalization.input.json`
- [ ] `cargo run --quiet -p career-cli -- resume enrich --input fixtures/resume/phase2/messy-unlabeled.enrichment-input.json`
- [ ] `cargo run --quiet -p career-cli -- job normalize --input fixtures/job/phase4a/complete-normalization.input.json`
- [ ] `cargo run --quiet -p career-cli -- job match --input fixtures/job/phase4b/complete-match.input.json`
- [ ] `git diff --check`
- [ ] Swift-boundary or phase-status changes: `scripts/verify-swift.sh`
- [ ] npm distribution changes: private current-host staging, publication source/candidate/fake-registry adversarial tests, exact tarball/install allowlists, and packaged/native parity

## Security and privacy

Describe input-boundary, payload-logging, network, dependency, and sensitive-data impact.

## Decisions and tradeoffs

Document intentional differences, dependencies, limitations, and deferred work.
