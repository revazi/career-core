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
6. Update schemas, examples, changelog, and coding-agent guidance together with public contract changes.
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

If the CLI contract changes, also invoke every changed command in canonical and compact modes (plus text where supported) and parse JSON independently. Managed-adapter changes require pinned `check-jsonschema` 0.34.1 through `CHECK_JSONSCHEMA_BIN`, all emitted bundles checked against the Draft 2020-12 metaschema, representative instances validated against bundles, and exact-bound/one-byte-over output checks. For distribution changes, install the CLI into a temporary root from a clean clone and run capability/operation/schema discovery and bundling through the installed binary outside the checkout. Phase 9 npm changes additionally require Node launcher/adversarial/signal tests; private offline `npm pack`; exact tarball/install allowlists/licenses/modes; current-host all-operation parity; and `scripts/test-npm-publication.sh`. The proposed `0.1.1` matrix also requires exact catalog-byte validation, all-eight synthetic selection/format/architecture policy tests, positive glibc/musl classification, GNU/musl substitution rejection, and explicit Unix/Windows file invariants. The Darwin/GNU native sub-gate additionally requires exact native `macos-14` ARM64, `macos-15-intel` x64, `ubuntu-22.04` x64/glibc 2.35, and `ubuntu-24.04-arm` with native Ubuntu 22.04 ARM64/glibc 2.35; offline extracted-package operation parity; and bounded `career.npm_native_inspection.v1` format, architecture, interpreter, dynamic-import, highest-GLIBC-symbol, size, mode, and SHA-256 evidence. The musl sub-gate requires digest-pinned Node 22.19.0 Alpine 3.22/musl 1.2.5 on architecture-matched native x64 and ARM64 hosted runners, positive outer/inner/Rust/loader architecture agreement, x64 ELF `DYN` static-PIE/AArch64 ELF `EXEC` plus no-interpreter/no-shared-library-import evidence, and the same extracted-package parity and bounded source/binary binding; `--platform`, QEMU, cross-compilation, and unknown-libc fallback are forbidden. The Windows sub-gate requires exact native `windows-2025` x64 and `windows-11-arm` ARM64, native process/Python/Node/Rust architecture agreement, all-operation offline extracted-package parity, `career.exe` regular non-symlink policy without a Unix runtime mode, and bounded PE32+/machine/system-DLL-import/size/SHA-256 evidence. Synthetic and cross-compiled artifacts are policy fixtures only, never release evidence. Publication fixtures must cover clean annotated main/tag/SHA binding, dirty/missing/lightweight/moved/non-main/version rejection, exact nine-package candidate property sets/bytes/modes/order, wrong SHA/target/provenance, artifact nesting/extra entry types, bootstrap/OIDC auth isolation, exact partial retries/conflicts, eight-native-before-launcher behavior, npm registry attestation validation, and synthetic npx-equivalent execution without contacting npm. Dirty local stages remain private non-candidates.

For Swift-boundary or final phase-status changes on macOS:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
scripts/verify-swift.sh
```

Regeneration must leave checked-in `CareerCore.swift` byte-equivalent. Inspect XCFramework slices, validate checksums, run all Swift tests, and build generic iOS-device and iOS-simulator destinations without signing.

## CI usage policy

The normal hosted CI workflow runs only by explicit `workflow_dispatch`; pull requests and pushes do not start Linux or macOS jobs automatically. Complete local verification first, then dispatch that workflow once for the final reviewed head when hosted verification is required. Concurrency cancellation prevents a superseded manual run from continuing to consume runner time.

The separate `pi-career-runtime-artifacts.yml` workflow is also manual-only and must not be folded into normal CI. It remains a transitional short-retention unsigned native CLI handoff to the external `pi-career` repository. Run it only for an exact reviewed commit; importing or tracking an archive is a separate `pi-career` review step, not a Core release.

The Phase 9 `npm-cli-packages.yml` workflow remains manual preparation-only. Historical `npm-publish.yml` records the immutable `v0.1.0` publication path. Final-sub-gate `npm-publish-v0.1.1.yml` preserves `npm-production`, pinned tooling/actions, bootstrap/OIDC isolation, exact npm provenance/integrity checks, all eight native packages before the launcher, and no-secret public acceptance on all eight targets. It publishes no crate, signature, notarization, GitHub Release, or custom asset. Implementation sessions must not dispatch it, authenticate/query npm, create the tag/release, or use credentials.

## Dependency policy

A new dependency requires all of:

- concrete active-phase need
- maintenance and license review
- explanation in the pull request
- no simpler standard-library or existing-dependency solution
- placement in the narrowest package that needs it

Core dependencies face a higher bar than adapter dependencies. No dependency may add telemetry or implicit network behavior. FFI dependencies additionally require MSRV, generated-code, unsafe-boundary, artifact-size, and license review; record distributable third-party notices before publication.

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
