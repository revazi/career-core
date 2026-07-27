# Current phase

## Active phase

**Phase 7 — Evidence-linked assisted resume variants**

## Status

The maintainer explicitly approved the production feature after formally accepting the `career-workbench` Phase 17 whole-resume side-by-side prototype. Phase 7 adds deterministic review and materialization contracts for selected untrusted resume changes. The core validates exact targets/evidence occurrence and structural safety, but it does not certify generated prose as factually true. No provider, prompt, persistence, UI, export, scoring, or matching behavior enters this repository.

## Approved Phase 7 scope

- provider-neutral `career.resume_variant_proposal.v1` with at most 50 evidence-linked line-targeted changes
- versioned review input/output with stable core-assigned change identifiers, deterministic order, conservative duplicate/overlap handling, bounded discard diagnostics, and an all-change preview
- versioned materialization input/output that revalidates the complete proposal and applies only explicitly selected canonical changes
- byte-equivalent baseline preservation, exact unselected source preservation, assisted/non-authoritative provenance, and mandatory factuality limitations
- no assisted variant may enter resume analysis or job matching
- JSON schemas, synthetic fixtures, CLI JSON/text commands, capability discovery, agent guidance, UniFFI facade parity, and adversarial tests
- no new dependency unless implementation proves the standard library insufficient

## Explicitly out of scope for Phase 7

- model/provider calls, prompts, networking, retries, credentials, UI, application persistence, TXT/PDF/DOCX export, clipboard behavior, variant naming, clocks, identifiers, or filesystem access in the core
- semantic entailment/factuality certification, arbitrary source repair, automatic acceptance, assisted scoring/matching, or mutation of the deterministic baseline
- changing existing stable operation semantics or fixtures

## Implemented Phase 7 behavior

- added provider-neutral review/materialization contracts with a 50-change ceiling and 1 MiB composite adapter envelopes
- validates exact line targets, exact resume/vacancy evidence occurrence, string/list/source bounds, unsupported controls, canonical ordering, and all-change preview validity
- discards malformed or ambiguous individual changes with bounded payload-free codes and discards every member of overlapping target sets
- assigns stable core-owned identifiers only after canonical sorting; provider identifiers have no authority
- materialization reruns complete review, validates the expected policy and selected identifiers, and applies only the selected canonical changes
- preserves the byte-equivalent baseline and every unselected range; output is exhaustively assisted/non-authoritative with mandatory factuality limitations
- added five public schemas, two CLI operations, two UniFFI functions, synthetic goldens, public/CLI/Swift parity tests, capability/skill/docs/security updates, and no dependency

## Phase 7 verification status

Passed locally on 2026-07-27:

- `cargo fmt --all --check`
- Clippy across all workspace targets/features with warnings denied
- 119 Rust unit/integration/doc test entries across core, CLI, public contracts, and Swift adapter
- locked all-feature workspace build
- all legacy and new CLI fixture commands, compact output, schema discovery/export, and exact Phase 7 golden comparisons
- all Phase 7 schemas validated against synthetic fixtures with an offline local registry
- generated Swift source is deterministic; five Swift package tests cover all eight input-taking operations with exact Rust golden bytes
- macOS/iOS/iOS-simulator XCFramework assembly, checksums, link smokes, and generic device/simulator builds through `scripts/verify-swift.sh`
- `git diff --check`
- Rust 1.85.0 `cargo check` across the workspace, all targets, and all features with the lockfile
- a fresh local clone at `df85166` passed formatting, Clippy, all 119 Rust test entries, locked all-feature build, Rust 1.85 checking, complete Swift/XCFramework verification, deterministic generated source, and a clean final worktree
- Fallow 3.9.1 audit/all/security reported zero findings, qualified because the analyzer recognized no Rust or Swift source files
- PR #11 CI run `30263939201` passed on macOS and Ubuntu, including Rust 1.85 MSRV and the complete Apple Swift verification

Still required before Phase 7 merge: explicit merge approval.

## Previous completed phase

Phase 6 — Swift binding boundary was completed and squash-merged through PR `#8` as `eb6960c`. Final PR-head CI run `30083962027` passed on Linux and macOS. Phases 0–6 are complete.

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
- MCP, Python, WASM, editor extensions, or other Phase 7 adapters

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

The actual SwiftUI application belongs in the separate future `career-workbench` repository. Creating that repository or implementing its first foundation phase requires explicit approval. Artifact publication, signing, App Store work, and every Phase 7 adapter also require separate approval.
