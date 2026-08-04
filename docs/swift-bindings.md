# Swift binding boundary

Phase 6 exposes stable deterministic operations to a future native application without moving Apple-platform behavior into `career-core`.

```text
career-workbench
        │
        ▼
CareerCoreSwift package
        │ generated Swift + UniFFI C ABI
        ▼
career-swift adapter crate
        │
        ▼
career-core
```

## Facade design

`career-swift` accepts owned UTF-8 JSON strings and returns canonical versioned JSON strings. It exports:

- `capabilities_json`
- `resume_evaluate_json`
- `resume_analyze_json`
- `resume_analysis_suggestions_review_json`
- `resume_analysis_replacements_review_json`
- `resume_normalize_json`
- `resume_enrich_json`
- `resume_variant_review_json`
- `resume_variant_materialize_json`
- `job_normalize_json`
- `job_match_json`

The generated Swift names use lower camel case, such as `resumeAnalyzeJson(inputJson:)`.

This narrow boundary is intentional:

- existing schemas remain the public data contract
- Swift does not inherit the complete internal Rust type graph
- adding internal Rust fields does not silently create an FFI ABI change
- CLI and Swift can prove exact canonical-output parity
- app-owned `Codable` types can evolve in the future workbench repository

The facade does not accept normalized or assisted documents for authoritative analysis or matching. Core functions still rerun deterministic baselines.

## Input and error safety

Before JSON parsing, the adapter applies the same envelope ceilings as the CLI:

- 262,144 UTF-8 bytes for single-document, enrichment, analysis-suggestion review, and analysis-replacement review operations
- 1,048,576 UTF-8 bytes for job matching and resume-variant review/materialization

UniFFI maps `CareerSwiftError` into typed Swift errors:

| Swift case | Meaning |
|---|---|
| `InvalidJson` | malformed or structurally incompatible contract JSON |
| `InputTooLarge` | adapter envelope exceeded before parsing |
| `InvalidInput` | typed `career-core` validation failure with stable code and field path |
| `OutputSerialization` | bounded adapter serialization failure |

JSON parsing errors report only the expected contract and line/column. Core messages remain bounded and source-payload-free.

## UniFFI evaluation

The adapter pins exact UniFFI `0.30.0`:

- `0.31.x` and `0.32.x` require Rust 1.87 and violate the project Rust 1.85 MSRV
- `0.30.0` supports Rust 2024 with the earlier upstream MSRV
- `0.30.0` includes stable generated Swift initialization ordering
- proc-macro/library mode avoids maintaining a duplicate UDL interface
- generated Swift performs runtime contract-version and API-checksum checks

UniFFI and its component crates are MPL-2.0. They remain adapter-only. Project-authored source remains `MIT OR Apache-2.0`; see [`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).

The dependency is intentionally larger than a handwritten C ABI. UniFFI was selected because handwritten allocation, string, panic, and ownership code would require a broader reviewed `unsafe` surface. No handwritten project source contains an unsafe block. `career-swift` denies unsafe source; the generated C ABI implementation is owned by the pinned UniFFI macros/runtime and remains isolated from `career-core`.

## Generated files and artifacts

Reviewed source:

- `crates/career-swift/` — Rust facade and pinned bindgen executable
- `swift/CareerCoreSwift/Package.swift` — local package definition
- `swift/CareerCoreSwift/Sources/CareerCore/CareerCore.swift` — normalized generated Swift source
- `swift/CareerCoreSwift/Tests/` — exact parity and error tests

Ignored local artifacts:

- `swift/CareerCoreSwift/Artifacts/CareerCoreFFI.xcframework`
- `swift/CareerCoreSwift/Artifacts/CareerCoreFFI.metadata.json`
- `swift/CareerCoreSwift/Artifacts/CareerCoreFFI.sha256`
- SwiftPM `.build` and `.swiftpm` state

The XCFramework contains only Apple Silicon slices:

| Library identifier | Rust target |
|---|---|
| `macos-arm64` | `aarch64-apple-darwin` |
| `ios-arm64` | `aarch64-apple-ios` |
| `ios-arm64-simulator` | `aarch64-apple-ios-sim` |

Deployment targets are macOS 13 and iOS 16.

UniFFI's `--xcframework` module-map mode emits `framework module`, which SwiftPM does not expose for a static-library XCFramework. The build intentionally emits a normal `module CareerCoreFFI` map and then packages it with `xcodebuild -create-xcframework`; macOS, iOS-device, and iOS-simulator builds verify this arrangement.

## Build and verification

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
scripts/build-swift-xcframework.sh
scripts/verify-swift.sh
```

Both scripts resolve Cargo and rustc through rustup's active toolchain, rather
than the first Rust installation in `PATH`. To use another installed toolchain,
set the standard `RUSTUP_TOOLCHAIN` variable consistently while installing the
Apple targets and running the scripts:

```bash
RUSTUP_TOOLCHAIN=stable rustup target add aarch64-apple-ios aarch64-apple-ios-sim
RUSTUP_TOOLCHAIN=stable scripts/verify-swift.sh
```

Verification covers:

- three Rust static-library targets
- deterministic Swift/header/module-map generation
- exact checked-in generated Swift source
- expected XCFramework platform/architecture metadata
- version/deployment/target metadata and per-file SHA-256 manifest verification
- macOS Swift Package compilation and five Swift tests
- exact Swift/Rust capabilities bytes
- exact output parity for all ten input-taking operations
- typed malformed, oversized, and core-validation errors
- arm64 iOS-device and simulator link smokes containing the exported Rust symbols
- generic iOS-device and iOS-simulator Swift Package builds without signing

Phase 6 creates no release and uploads no artifact. Publishing, signing, notarization, remote package URLs, and app distribution remain separate approval gates.

## Product boundary

The separate `career-workbench` repository owns:

- SwiftUI and navigation
- local database and migrations
- document pickers, security-scoped URLs, and extraction adapters
- Keychain and user consent
- provider clients and failure isolation
- application lifecycle and platform diagnostics

The binding package owns none of those concerns. It remains a deterministic local adapter over versioned JSON contracts.
