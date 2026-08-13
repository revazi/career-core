# Contributing

Thank you for helping build `career-core`.

## Maintainer and governance

Career Core is primarily maintained by [Revaz Zakalashvili](https://github.com/revazi). See [`MAINTAINERS.md`](MAINTAINERS.md) for contact, support, ownership, and release-governance details. Use public issues only for sanitized reports; follow [`SECURITY.md`](SECURITY.md) for vulnerabilities.

## Scope

The project is a deterministic, local-first Rust library with narrow CLI/Swift adapters and one user-facing npm package, `@revazi/career`; native optional packages are internal implementation details. Keep LLM/provider calls, API keys, prompts, UI, persistence, remote fetching, billing, and platform-specific behavior outside the core package. npm launcher behavior must remain network-free, package-local, fail-closed, and transparent to native CLI bytes. Provider-neutral external proposals are untrusted input and must preserve the deterministic baseline.

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
node --test npm/tests/launcher.test.js
scripts/test-npm-cli-packages.sh
scripts/test-npm-publication.sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features --locked
cargo run --quiet -p career-cli -- capabilities
cargo run --quiet -p career-cli -- operations --format json-compact
cargo run --quiet -p career-cli -- schema list
cargo run --quiet -p career-cli -- schema export \
  --id career.job_match.v1 --format json-compact
cargo run --quiet -p career-cli -- schema bundle \
  --id career.job_match_input.v1 --format json-compact
scripts/verify-managed-adapter-contracts.sh target/debug/career
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
scripts/test-pi-career-runtime-artifact.sh
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

`scripts/verify-managed-adapter-contracts.sh` requires `check-jsonschema`; use reviewed pinned version `0.34.1` and set `CHECK_JSONSCHEMA_BIN` when it is not on `PATH`. It validates all emitted bundles against the Draft 2020-12 metaschema and representative instances independently of Rust tests.

## npm package verification

`scripts/test-npm-cli-packages.sh` performs private current-host staging, offline/ignore-scripts packing, exact allowlists, install outside the checkout, and all-operation packaged/native parity. Dirty staging is private/local/non-candidate only.

`scripts/test-npm-publication.sh` uses synthetic Git repositories, external candidate staging, an opposite-target format fixture, offline install/parity, adversarial tar/provenance/tag tests, and a fake npm registry to exercise bootstrap/OIDC, partial/idempotent retry, conflicts, registry attestations, auth failure, and launcher-last policy without contacting npm. Run it with Node 22.19.0 and reviewed local npm 10.9.3 or publication npm 11.6.2.

Source templates must stay `private: true`. Active source rejects native Windows Cargo targets, and active distribution supports macOS and Linux only; Windows hosts must build and run under WSL as Linux. Do not address internal native packages as user surfaces. Do not query/authenticate/publish npm, dispatch workflows, or create repository release state from an implementation session. See [`docs/contracts/npm-cli-distribution-v2.md`](docs/contracts/npm-cli-distribution-v2.md) and [`docs/releasing.md`](docs/releasing.md).

## Maintainer-only transitional `pi-career` runtime inputs

The normal verification ladder and normal manual CI do not produce binary artifacts. When changing the separate runtime-artifact script or workflow, validate the current native host into an external temporary directory:

```bash
scripts/test-pi-career-runtime-artifact.sh
```

The test exercises fail-closed target selection, calls the reusable preparation script in an external temporary directory, validates the archive allowlist and metadata, and reruns discovery from the packaged executable. The underlying `--allow-dirty` option exists only so an uncommitted script change can be tested; it marks the archive dirty and that archive must not be imported. Reviewed maintainer transfer artifacts require a clean exact commit and the manual `.github/workflows/pi-career-runtime-artifacts.yml` workflow. They are transitional unsigned short-retention inputs for separate `pi-career` review, not Core releases or end-user downloads.

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
