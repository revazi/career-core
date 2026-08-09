# Current phase

## Active phase

**Phase 9 extension — prepare proposed lockstep `v0.1.1` for an exact eight-target npm native matrix. Phases 0–8 and the public `v0.1.0` release remain complete and immutable. No new `0.1.1` platform is supported until its final packaged binary executes on the exact native OS and architecture in CI.**

## Phase 9 extension authorized scope

The extension retains one user-facing npm surface, `@revazi/career`, and exact internal optional packages. Proposed patch `0.1.1` catalogs Darwin ARM64/x64, GNU Linux x64/ARM64, musl Linux x64/ARM64, and MSVC Windows x64/ARM64 in one reviewed ordered target catalog. Catalog entries and cross-compilation are policy data, not release evidence. Every target requires exact native package execution; skipped, emulated, metadata-only, and cross-compiled-only results remain release blockers.

The first reviewed sub-gate is complete: the catalog, eight private lockstep templates, positive glibc/musl classification, exact Mach-O/ELF/PE architecture checks, target-specific Unix/Windows file invariants, package/provenance v2, synthetic adversarial coverage, and generalized offline candidate policy merged through PR `#24` as `2d38fb92b79d26ad0958328416bdb86ca09d95b5`. Exact-head CI run `31319422699` and preparation-only Darwin ARM64/GNU Linux x64 run `31319422714` passed. It did not add a new support claim or authorize publication.

The second reviewed sub-gate is complete: exact native Darwin x64 and GNU Linux ARM64 packaged-binary evidence merged through PR `#25` as `c02a2f7a77c46f3c4ad7f23f3c57aef651502604` while reproving Darwin ARM64 and GNU Linux x64. Exact-head preparation-only run `31322184434` and full CI run `31322184442` passed.

The third reviewed sub-gate is complete: exact native musl x64/ARM64 packaged-binary execution and target-specific static-linkage evidence merged through PR `#26` as `7054a4a0ec05c278a00ba1b53d5365634fc95f37`. Exact-head six-target preparation-only run `31325713420` and full CI run `31325713347` passed. Musl x64 is ELF `DYN` static PIE with `NOW PIE`; musl AArch64 is ELF `EXEC`; both use exact Alpine 3.22/musl 1.2.5 and have no interpreter, shared-library import, or imported GLIBC symbol.

The active fourth sub-gate adds exact native MSVC Windows x64/ARM64 packaged-binary execution and bounded PE32+ import evidence on `windows-2025` and `windows-11-arm`. It must positively prove native OS/process/Python/Node/Rust architecture, exact PE machine and executable name, Windows regular non-symlink semantics without a Unix mode claim, reviewed system-DLL imports, offline extracted package/launcher parity, and bounded source/catalog/version/size/SHA-256 evidence. The remaining later sub-gate is the exact nine-package protected candidate plus final documentation. Do not begin it without explicit maintainer approval. The protected `v0.1.1` workflow remains deferred until that final publication-candidate sub-gate; the existing `v0.1.0` workflow and public bytes remain historical and immutable.

The deterministic Core algorithms, public v1 operation/schema descriptors, limits, ordering, warnings, evidence, uncertainty, and assisted authority remain unchanged. The Cargo/npm/CLI version metadata advances to `0.1.1`, so `career operations` and all successful result documents report exact `core_version: "0.1.1"`; descriptor and schema semantics do not change.

Local first-sub-gate verification on Apple Silicon macOS passed the 59-entry Node launcher/adversarial suite, including architecture-bound libc evidence and pathname-replacement rejection; exact private current-host package staging and operation parity; nine-package registry-free candidate/fake-registry policy tests with bounded manifest/member/decompression and no-npm attack regressions; the complete Rust workspace and required CLI/schema/managed-adapter ladder; Rust 1.85 MSRV; Swift verification; the transitional `pi-career` runtime-artifact check; and Fallow changed/all/security checks with no correctness or security findings and no high/critical health finding. Non-host target artifacts in these tests are synthetic policy fixtures only. They are not native execution evidence, do not satisfy any later sub-gate, and add no public platform support.

Second-sub-gate exact-head run `31322184434` at `59342a1a3a7e9f61e56d3c4116f8183eadc116c2` passed all four exact native private packaged-binary jobs: `macos-14` ARM64, `macos-15-intel` x64, `ubuntu-22.04` x64/glibc 2.35, and `ubuntu-24.04-arm` with native Ubuntu 22.04 ARM64/glibc 2.35. Every job used rustc/Cargo 1.97.1 and Node 22.19.0, passed offline extracted-package all-operation parity, and emitted bounded source/catalog/host/version/format/linkage/import/interpreter/mode/size/SHA-256 evidence without upload or publication. Both GNU binaries imported no GLIBC symbol newer than 2.34, within the proposed 2.35 floor. This merged intermediate evidence proves the four runner/package paths but is not a public support claim; final release source must reprove all eight targets.

## Completed `v0.1.0` Phase 9 baseline

Career Core owns one user-facing public npm surface: exact `@revazi/career@0.1.0`, bin `career`, at annotated tag `v0.1.0`. Exactly two lockstep optional native packages implement Apple Silicon macOS and x86-64 GNU/Linux internally; users and `pi-career` never address them directly. The launcher is Node 22+, resolves only its package-local implementation, requires glibc 2.35+ on Linux, validates exact package/provenance/version/target/type/mode/size/SHA-256 consistency, and uses argv-only inherited-stdio execution without network, PATH fallback, install hooks, providers, telemetry, or bypass.

Checked-in templates remain `private: true`; generated public manifests, binaries, provenance, and tarballs remain outside the checkout. The explicit candidate path requires clean exact fetched `origin/main`, reviewed SHA, annotated unmoved `v0.1.0`, exact Node 22.19.0/npm 11.6.2 and rustc/Cargo 1.97.1, `macos-14` ARM64 or `ubuntu-22.04` x64/glibc 2.35, strict property/file/mode/source-byte allowlists, and native-before-launcher assembly. Rust 1.85 remains a separate MSRV gate.

The sole publish path is protected manual `.github/workflows/npm-publish.yml` with `npm-production`, explicit temporary-token bootstrap/recovery or default token-free OIDC trusted publishing, npm public access/provenance, exact registry integrity/SLSA-attestation/idempotence/conflict checks, and final no-secret public acceptance on both hosts. Package-contained SHA-256 is consistency-only; npm registry provenance is separate; independent native-binary signatures remain explicitly absent. After npm and both public acceptance jobs pass, the parent maintainer creates the annotated-tag GitHub Release with dated notes and no custom assets. No crate publication, signing, or notarization is part of this phase.

This phase must preserve byte-equivalent Core algorithms, public schemas, capabilities, operation catalog, operation outputs/order/warnings/evidence/uncertainty/assisted authority, the 33,554,432-byte successful machine-output bound, and Swift behavior. The existing `pi-career` runtime-artifact path remains transitional until exact public package acceptance and the separate consumer migration pass.

Implementation now includes:

- exact private source templates plus bounded user-facing launcher README/public metadata
- glibc 2.35 runtime/package/provenance/build-symbol floor and strict release toolchain/runner records
- clean main/annotated-tag/SHA source gate; public native staging; exact three-tarball assembly; `career.npm_publication_candidate.v1`
- exact manifest/tar/mode/entry/source-byte verification, extracted candidate all-operation parity, and synthetic npx-equivalent execution
- tracked minimal publish driver with explicit ephemeral bootstrap npmrc versus auth-free OIDC npmrc, partial/idempotent retry, conflicts, bounded transients, launcher-last checks, and mandatory registry attestations
- manual publication workflow with corrected immutable action pins, protected environment, one-time bootstrap, steady OIDC, and public-registry matrix acceptance
- registry-free synthetic Git and fake-npm adversarial tests, plus exact post-bootstrap `npm trust`/token-deletion governance
- retained transitional `pi-career` artifacts and an exact public-package-only consumer handoff

The parent maintainer proved `@revazi` control through authenticated publication, configured protected `npm-production`, completed temporary-token bootstrap, removed the GitHub secret, reported revoking the npm bootstrap token, and configured the exact trusted publishers for all three packages. Exact public registry integrity and SLSA provenance are available for `@revazi/career@0.1.0` and both internal native packages.

Hosted PR-head CI run `31284082728` exposed a real Linux cancellation race: the inherited native stdout pipe could report readiness before the launcher returned from `spawn()` and installed signal handlers, allowing default launcher termination and an orphan native process holding the pipe open. The local follow-up at committed base `3fc8cdc73e677c8bd80327b5423602fea27d7d46` installs handlers before spawn, queues a signal received before child assignment, and removes handlers on synchronous launch failure. A deterministic ordering regression and Linux `/proc` caught-signal readiness gate precede cancellation; cleanup polls for process absence with a bound rather than relying on a fixed sleep.

Prepublication local verification passed on Apple Silicon macOS on 2026-08-09, and the Linux cancellation follow-up subsequently passed independent review and hosted CI:

- Node 22.19 passed 46 launcher/selection/glibc-floor/manifest/provenance/toolchain/runner/binary/argv/stdio/exit/signal/metadata/public-boundary test entries
- Linux amd64 under Docker/QEMU with exact Node 22.19.0 and Rust 1.97.1 passed all 46 entries and 50/50 focused cancellation iterations; macOS passed 100/100 focused cancellation iterations
- private current-host staging passed offline exact tarball/install/README/license allowlists and every-operation native parity
- publication fixtures passed canonical-origin/main/clean/annotated-tag/SHA/version gates; dirty/missing/lightweight/moved/non-main/credential-origin rejection; exact public property/file/mode/byte/order checks; extra directory/FIFO/nesting-style rejection; wrong SHA/target/toolchain/glibc/provenance/version/dependency/lifecycle mutations; extracted candidate every-operation parity; and synthetic npx-equivalent invocation
- fake npm passed all-absent bootstrap, exact idempotence, first/second-package interruption recovery, partial/conflicting state rejection, token/OIDC isolation, OIDC prerequisites, transient retry, auth/permanent native failure, launcher-last protection, and missing/malformed/eventually consistent registry-attestation tests without npm contact
- installed CLI, formatting, warnings-denied Clippy, all 143 Rust test entries, locked all-feature build, and Rust 1.85 all-target/all-feature check passed
- pinned `check-jsonschema` 0.34.1 independently validated all managed bundles and representative instances; the complete required CLI ladder and exact goldens passed
- transitional `pi-career` runtime-artifact verification passed
- `scripts/verify-swift.sh` passed deterministic generated source, five Swift tests, all Apple slices/checksums/link/build gates, and no tracked Swift change
- exact clean Phase 8 base parity passed for capabilities, operation catalog, schema catalog, all 26 exports, all 26 bundles, and all ten input operations
- shell/Python/Node/JSON/YAML syntax, relative Markdown links, action/package/workflow/generated-payload/credential guards, `git diff --check`, and Fallow audit/all/changed/security passed with zero findings

PR `#22` squash-merged as `478e9b15f489c3785dc470af8eb83a7fed9e75e8`; merged-main CI `31285098017` and private two-platform preparation run `31285098073` passed. Annotated tag `v0.1.0` resolves to that commit. Bootstrap/public acceptance run `31286374878` passed after safe idempotent recovery from npm's observed 3–5 minute first-package registry propagation; exact Darwin, Linux, then launcher bytes and SLSA provenance were preserved. No-token OIDC steady-state/public acceptance run `31287506624` then passed on `macos-14` and `ubuntu-22.04`. The `NPM_TOKEN` GitHub secret is absent, and the asset-free GitHub Release is public at `https://github.com/revazi/career-core/releases/tag/v0.1.0`. A bounded follow-up replaces the 50-second checks with one combined ten-minute integrity/provenance readiness window per package and raises the protected publish-job ceiling to 40 minutes, with delayed/never-visible integrity and provenance fake-registry regressions. No crate, custom asset, signature, notarization, or `pi-career` source edit occurred.

## Completed Phase 8 status

The demonstrated external `pi-career` consumer authorized deterministic CLI discovery/bundling/output-bound contracts and maintainer artifact compatibility evidence. Career Core still owns no Pi package, `career_run` runtime, handles, projection, persistence, provider/model behavior, networking, UI, extraction/export, or adapter state.

Implemented on the Phase 8 branch:

- additive `career.operation_catalog.v1` with exact runtime `core_version`, 11 one-to-one available capability mappings, four explicit bootstrap operations, stable order/CLI paths/transports/schemas, input ceilings, and a 33,554,432-byte (32 MiB) successful machine-output ceiling
- deterministic `career schema bundle --id ...` with recursively embedded dependencies, retained root resource, root-local JSON Pointers, and fail-closed unknown/remote reference handling
- full serialization before successful machine stdout, with exact-bound/one-byte-over tests and schema-valid `output_write_failed` overflow reporting
- operation-catalog public schema/golden, all-bundle deterministic/local-ref tests, independent installed-binary inspection, Draft 2020-12 metaschema checks, and representative bundled-schema instance validation
- versioned `career.pi_career_managed_adapter_compatibility.v1` artifact evidence retaining existing source/target/dirty/executable/license/synthetic/no-release gates and adding catalog, mapping, bundle, output-bound, and deterministic-result proofs
- installed-CLI and CI coverage outside the checkout; generic agents remain discovery-first while reviewed managed adapters may internally cache only non-sensitive metadata per exact Core version

`career.capabilities.v1`, existing capability-backed successful output bytes, unbundled schema-export bytes, Core algorithms, dependencies, Swift facade behavior, and generated Swift bytes are unchanged.

Local Phase 8 verification passed on Apple Silicon macOS on 2026-08-08:

- formatting, warnings-denied Clippy, 143 Rust unit/integration test entries, and locked all-feature workspace build passed
- Rust 1.85.0 checked every workspace target/feature with the lockfile
- all 11 pre-Phase 8 successful capability outputs and all 25 existing unbundled schema exports were byte-equivalent to clean base `60d4a04a7be1ebd5ca7c7577458bd45fdbb41ab2`
- actual 32 MiB exact-bound machine JSON succeeded; one byte over failed before stdout and reported schema-valid `output_write_failed`
- installed-CLI acceptance ran outside the checkout, generated every bundle, and independently proved catalog completeness, schemas, transports, input/output bounds, local refs, and deterministic framing
- pinned `check-jsonschema` 0.34.1 validated all 26 emitted bundles against the Draft 2020-12 metaschema and validated discovery plus representative operation inputs/outputs against those bundles
- the complete required CLI ladder passed, including operation catalog and bundle discovery
- maintainer artifact shell/metadata tests passed target/dirty/output-location/archive/license/synthetic gates plus catalog/mapping/bundle/bound/result compatibility proofs and five adversarial metadata mutations
- complete `scripts/verify-swift.sh` passed deterministic generated-source, exact Rust/Swift operation parity, macOS/iOS/iOS-simulator XCFramework/checksum/link/build gates with no tracked Swift change
- shell/Python/CI-YAML syntax, `git diff --check`, Fallow changed-code/all/security, and Markdown link inspection passed; Fallow reported zero findings but recognized no Rust/Swift source files

The manual hosted CI and runtime-artifact workflows were not dispatched from the implementation branch. Final reviewed-head Linux/macOS workflow evidence remains a maintainer review gate, not implementation-time publication approval.

The native Pi package and bundled Agent Skill remain separately maintained in [`pi-career`](https://github.com/revazi/pi-career). Unsigned native archives remain short-retention maintainer handoff inputs for approved native Ubuntu x86_64 GNU/Linux and Apple Silicon macOS runners, not releases or end-user downloads.

MCP, additional bindings/adapters, publication, release artifacts, signing, and additional targets remain unapproved.

## Earlier completed phase

Phase 7 evidence-linked assisted resume review and its review-only analysis-suggestion and analysis-replacement extensions are implemented and merged. The core validates exact targets/evidence occurrence and structural safety, but it does not certify generated prose as factually true. No provider, prompt, persistence, UI, export, assisted scoring, or assisted matching behavior enters the core.

## Approved Phase 7 scope

- provider-neutral `career.resume_variant_proposal.v1` with at most 50 evidence-linked line-targeted changes
- review-only `career.resume_analysis_suggestion_review.v1` with at most three source-targeted advisory suggestions bound to a freshly rerun current failed action, confirmed/provisional status, stable identifiers/order, and bounded payload-free discards
- review-only `career.resume_analysis_replacement_review.v1` with at most three exact source-targeted before/proposed-after replacements bound to freshly rerun current failed actions, stable identifiers/order, and bounded payload-free discards
- versioned variant review input/output with stable core-assigned change identifiers, deterministic order, conservative duplicate/overlap handling, bounded discard diagnostics, and an all-change preview
- versioned materialization input/output that revalidates the complete proposal and applies only explicitly selected canonical changes
- byte-equivalent baseline preservation, exact unselected source preservation, assisted/non-authoritative provenance, and mandatory factuality limitations
- no assisted variant or suggestion may enter resume analysis or job matching
- JSON schemas, synthetic fixtures, CLI JSON/text commands, capability discovery, agent guidance, UniFFI facade parity, and adversarial tests
- no new dependency unless implementation proves the standard library insufficient

## Explicitly out of scope for Phase 7

- model/provider calls, prompts, networking, retries, credentials, UI, application persistence, TXT/PDF/DOCX export, clipboard behavior, variant naming, clocks, identifiers, or filesystem access in the core
- semantic entailment/factuality certification, arbitrary source repair, automatic acceptance, assisted scoring/matching, mutation of the deterministic baseline, or analysis-suggestion candidate/selection/application/materialization behavior
- changing existing stable operation semantics or fixtures

## Implemented Phase 7 behavior

- added provider-neutral review/materialization contracts with a 50-change ceiling and 1 MiB composite adapter envelopes
- validates exact line targets, exact resume/vacancy evidence occurrence, string/list/source bounds, unsupported controls, canonical ordering, and all-change preview validity
- discards malformed or ambiguous individual changes with bounded payload-free codes and discards every member of overlapping target sets
- assigns stable core-owned identifiers only after canonical sorting; provider identifiers have no authority
- materialization reruns complete review, validates the expected policy and selected identifiers, and applies only the selected canonical changes
- preserves the byte-equivalent baseline and every unselected range; output is exhaustively assisted/non-authoritative with mandatory factuality limitations
- added five public schemas, two CLI operations, two UniFFI functions, synthetic goldens, public/CLI/Swift parity tests, capability/docs/security updates, and no dependency
- added three public review-only analysis-suggestion schemas, one CLI operation, one UniFFI JSON function, synthetic golden/parity coverage, and no dependency; this contract reruns and preserves the authoritative analysis baseline and cannot materialize a resume
- added three public review-only analysis-replacement schemas, one CLI operation, one UniFFI JSON function, synthetic golden/parity coverage, and no dependency; this separate explicit contract returns canonical exact before/proposed-after values for non-authoritative diff display and cannot select, apply, materialize, or mutate a resume

## Phase 7 verification status

Passed locally on 2026-07-27:

- `cargo fmt --all --check`
- Clippy across all workspace targets/features with warnings denied
- 119 Rust unit/integration/doc test entries across core, CLI, public contracts, and Swift adapter
- locked all-feature workspace build
- all legacy and new CLI fixture commands, compact output, schema discovery/export, and exact Phase 7 golden comparisons
- all Phase 7 schemas validated against synthetic fixtures with an offline local registry
- generated Swift source is deterministic; five Swift package tests cover all ten input-taking operations with exact Rust golden bytes
- macOS/iOS/iOS-simulator XCFramework assembly, checksums, link smokes, and generic device/simulator builds through `scripts/verify-swift.sh`
- `git diff --check`
- Rust 1.85.0 `cargo check` across the workspace, all targets, and all features with the lockfile
- a fresh local clone at `df85166` passed formatting, Clippy, all 119 Rust test entries, locked all-feature build, Rust 1.85 checking, complete Swift/XCFramework verification, deterministic generated source, and a clean final worktree
- Fallow 3.9.1 audit/all/security reported zero findings, qualified because the analyzer recognized no Rust or Swift source files
- PR #11 CI run `30263939201` passed on macOS and Ubuntu, including Rust 1.85 MSRV and the complete Apple Swift verification
- hosted CI is now manual-dispatch only; the workflow-policy change passed the complete local Rust suite, locked build, CLI fixture commands, Swift/XCFramework verification, and `git diff --check` without spending a new hosted run

### Review-only analysis-suggestion extension verification

Passed locally on 2026-08-03:

- `cargo fmt --all --check`, Clippy with warnings denied, all-feature workspace tests (126 Rust unit/integration/doc test entries), and locked workspace build
- Rust 1.85.0 workspace/all-target/all-feature locked check
- all legacy CLI fixture commands plus canonical, compact, and text analysis-suggestion review output; canonical output exactly matches the new synthetic golden
- all new schemas pass Draft 2020-12 metaschema validation; the proposal, review input, and review golden each pass an independent offline-base-uri schema validation
- regenerated Swift source exposes only core-defined JSON facade functions, including `resumeAnalysisSuggestionsReviewJson` and `resumeAnalysisReplacementsReviewJson`; `scripts/verify-swift.sh` passes macOS/iOS/iOS-simulator builds, checksums, link smokes, and five Swift tests covering all ten input-taking operations
- Fallow changed-code audit/all/security reported zero findings, qualified because the analyzer recognizes no Rust or Swift source files
- `git diff --check`

### Review-only analysis-replacement extension verification

Passed locally on 2026-08-03:

- `cargo fmt --all --check`, Clippy with warnings denied, all-feature workspace tests (135 Rust unit/integration/doc test entries), and locked workspace build
- Rust 1.85.0 workspace/all-target/all-feature locked check
- all required legacy CLI fixture commands plus canonical, compact, and text analysis-replacement review output; canonical output exactly matches the new synthetic golden
- all 25 Draft 2020-12 schemas passed metaschema validation in an offline local registry; replacement proposal, review input, and review golden passed independent offline schema validation
- regenerated Swift source exposes the core-defined `resumeAnalysisReplacementsReviewJson` function; `scripts/verify-swift.sh` passed macOS/iOS/iOS-simulator builds, checksums, link smokes, and five Swift tests covering all ten input-taking operations with exact Rust golden bytes
- `git diff --check`

Phase 7 landed on `main` through the reviewed variant, analysis-suggestion, and analysis-replacement changes in PRs `#11`, `#12`, and `#13`. The subsequent Swift toolchain-selection fix landed through PR `#14`. No Phase 7 merge gate remains open.

### Phase 0–7 completeness re-audit

Passed locally on 2026-08-04:

- formatting, Clippy with warnings denied, all 135 Rust unit/integration/doc test entries, and the locked all-feature workspace build
- Rust 1.85.0 workspace/all-target/all-feature locked check
- all 11 available capabilities mapped across Core, CLI, schemas, goldens, Swift, and public docs, with no missing deterministic operation
- canonical, compact, and text invocation of all ten input-taking commands; every canonical output remained byte-equivalent to its reviewed golden
- all 25 public schemas passed Draft 2020-12 metaschema validation and independent representative-instance validation through an offline local registry
- temporary-root source installation followed by installed-binary capability and schema discovery outside the checkout
- documented exit statuses `2`–`6` and bounded payload-free diagnostics independently exercised
- complete `scripts/verify-swift.sh` XCFramework, checksum, generated-source, link, Apple build, and exact Rust/Swift parity verification
- no Core algorithm, public JSON/CLI/Swift contract, schema, fixture, dependency, or Phase 8 adapter change

## Earlier completed Phase 6

Phase 6 — Swift binding boundary was completed and squash-merged through PR `#8` as `eb6960c`. Final PR-head CI run `30083962027` passed on Linux and macOS.

## Completed Phase 6 scope

- evaluate UniFFI against Rust 1.85, Rust 2024, current stable Swift, and Apple targets
- add a narrow Swift adapter crate depending inward on `career-core`
- expose all stable deterministic operations through owned FFI-safe values
- map malformed JSON, adapter limits, typed core errors, and serialization failures into Swift errors
- generate deterministic Swift source, C headers, and module metadata
- build one XCFramework containing Apple Silicon macOS, iOS device, and Apple Silicon iOS simulator static libraries
- provide a local Swift Package wrapper and minimal smoke executable/tests
- prove exact fixture parity across the Rust CLI and Swift calls
- add reproducible local build/checksum guidance and macOS CI verification
- isolate and document all generated FFI unsafe behavior

## Explicitly out of scope

- SwiftUI product screens or navigation
- persistence, SQLite, SwiftData, Core Data, migrations, or application tracking
- document pickers, PDF/DOCX extraction, OCR, or visual layout inspection
- Keychain, provider clients, prompts, model calls, consent UI, or network access
- App Store packaging, signing, notarization, TestFlight, or cloud sync
- publishing a Swift package, GitHub release, XCFramework, crate, or binary artifact
- changing deterministic core algorithms, existing JSON contracts, CLI behavior, or schemas
- MCP, Python, WASM, editor extensions, or other optional adapters

## Compatibility and boundary requirements

- `career-core` remains unaware of UniFFI, Swift, Xcode, files, and platform state
- the adapter accepts and returns canonical versioned JSON strings so Swift does not mirror the entire internal Rust type graph at the FFI boundary
- the adapter applies the existing CLI JSON-envelope byte ceilings before parsing
- successful JSON includes the canonical pretty representation and trailing newline used by CLI goldens
- assisted resume values remain separate and cannot enter baseline analysis or matching
- generated Swift errors contain bounded codes/messages/field paths and never source payloads
- no handwritten `unsafe` is allowed; any unsafe expansion must be limited to pinned UniFFI scaffolding and documented
- Apple deployment targets and package versioning are explicit and reproducible

## UniFFI evaluation

- UniFFI `0.32.x` and `0.31.x` require Rust 1.87 and are incompatible with the project MSRV.
- UniFFI `0.30.0` supports Rust 2024 with an upstream MSRV below Rust 1.85 and includes deterministic Swift initialization ordering.
- The adapter pins exact UniFFI `0.30.0`; newer minor versions require an explicit MSRV and generated-output review.
- Proc-macro/library mode avoids a duplicate UDL interface while keeping the exported surface narrow.
- UniFFI is MPL-2.0; it remains adapter-only and does not change the licensing of project source files.

## Acceptance gate (passed)

- Rust 1.85 builds the entire workspace including the Swift adapter and pinned bindgen tool
- generated Swift source/header/module map is deterministic and reviewable
- Swift Package tests call every stable core operation and match canonical fixture bytes
- invalid, oversized, and core-rejected inputs become typed Swift errors without payload echo
- one XCFramework contains valid macOS arm64, iOS arm64, and iOS-simulator arm64 slices
- Swift Package builds for macOS, generic iOS device, and generic iOS simulator destinations
- no core source or existing CLI/schema/golden output changes
- checksums and version metadata for locally generated Apple artifacts are reproducible
- formatting, Clippy, Rust tests, Swift tests, Apple builds, clean clone, security checks, and GitHub Actions pass

## Implemented

- Added adapter-only `career-swift` with exact UniFFI `0.30.0`, proc-macro/library-mode generation, and no core dependency reversal.
- Added seven JSON facade functions covering capability discovery and every stable input-taking deterministic operation.
- Added typed `InvalidJson`, `InputTooLarge`, `InvalidInput`, and `OutputSerialization` Swift errors.
- Added CLI-equivalent 262,144-byte single-input and 1,048,576-byte match-envelope limits before parsing.
- Added exact canonical pretty JSON plus trailing-newline output parity.
- Added reviewed generated `CareerCore.swift`, local Swift Package, smoke executable, and five Swift tests.
- Added reproducible build/verify scripts for macOS arm64, iOS arm64, and iOS-simulator arm64 static libraries and XCFramework assembly.
- Added canonical artifact metadata, deterministic XCFramework plist ordering, and per-file SHA-256 manifests.
- Added arm64 iOS device/simulator link smokes plus generic Xcode package builds.
- Added UniFFI/MSRV/module-map evaluation, unsafe isolation, MPL notices, integration docs, release guidance, and macOS CI coverage.
- Added no SwiftUI, persistence, extraction, provider, network, signing, or publication behavior.

## Verification

- formatting and Clippy pass across the workspace with warnings denied
- 109 Rust tests pass with all features and the lockfile
- locked workspace build and rustdoc with warnings denied pass
- Rust 1.85 builds all targets/features, including the pinned bindgen tool
- five Swift tests pass and execute every stable operation against canonical Rust fixture bytes
- Swift smoke capabilities output is byte-equivalent to CLI output
- malformed, oversized, and core-rejected inputs map to typed source-payload-free Swift errors
- checked-in generated Swift source is byte-equivalent after regeneration
- XCFramework metadata contains exactly macOS arm64, iOS arm64, and iOS-simulator arm64
- artifact metadata records package `0.1.0`, UniFFI `0.30.0`, deployment targets, and Rust targets
- SHA-256 manifests validate and match across no-change rebuilds and a fresh Cargo target directory
- arm64 iOS device/simulator Swift link smokes contain exported UniFFI symbols
- macOS Swift tests plus generic iOS-device and iOS-simulator Xcode builds pass
- every prior CLI/schema/golden test remains unchanged and green
- `career-core` and `career-cli` dependency trees contain no UniFFI dependency
- no provider, TLS, async-runtime, telemetry, or network stack was added
- project-authored Rust contains no unsafe block; `career-swift` denies unsafe source
- all new dependency licenses were reviewed; MPL components are isolated and recorded in `THIRD_PARTY_NOTICES.md`
- relative Markdown links, shell syntax, CI YAML, credential scans, and `git diff --check` pass
- Fallow changed-code and security checks report no findings; Fallow does not analyze Rust or Swift source
- a clean local clone passes the full Rust, MSRV, Swift, XCFramework, checksum, schema, and prior-golden suite without tracked changes
- final PR-head GitHub Actions run `30083962027` passed on Ubuntu and macOS, including 109 Rust tests per platform and the complete macOS Swift verification script

## Deferred gate

The actual SwiftUI application belongs in the separate `career-workbench` repository. Artifact publication, signing, App Store work, and every future optional Core adapter or binding require separate approval.
