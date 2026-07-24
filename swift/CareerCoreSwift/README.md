# CareerCoreSwift

`CareerCoreSwift` is the local Swift Package wrapper for the deterministic `career-core` Rust library. It is a binding boundary, not the `career-workbench` application.

## Supported build slices

The local XCFramework contains:

- Apple Silicon macOS: `arm64-apple-macos`
- iOS device: `arm64-apple-ios`
- Apple Silicon iOS simulator: `arm64-apple-ios-simulator`

The package declares macOS 13 and iOS 16 as minimum deployment versions. Intel macOS and Intel iOS-simulator artifacts are not currently included.

## Prerequisites

- Rust 1.85 or newer installed through `rustup`
- Rust targets `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- a full Xcode installation with the iOS platform installed
- Swift 5.9 package tooling or newer

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

## Build and verify

From the repository root:

```bash
scripts/build-swift-xcframework.sh
scripts/verify-swift.sh
```

The build script:

1. builds pinned Rust static libraries for the three Apple targets
2. runs pinned UniFFI `0.30.0` in library mode
3. regenerates `Sources/CareerCore/CareerCore.swift`
4. creates the local `Artifacts/CareerCoreFFI.xcframework`
5. writes version/target metadata and verifies `Artifacts/CareerCoreFFI.sha256`

The XCFramework, metadata, and checksum manifest are ignored build outputs. Generated `CareerCore.swift` is reviewed source and must remain byte-equivalent after regeneration.

## Swift API

The generated module exports narrow JSON functions:

```swift
let capabilities = try capabilitiesJson()
let evaluation = try resumeEvaluateJson(inputJson: resumeInput)
let analysis = try resumeAnalyzeJson(inputJson: resumeInput)
let normalization = try resumeNormalizeJson(inputJson: resumeInput)
let enrichment = try resumeEnrichJson(inputJson: enrichmentInput)
let job = try jobNormalizeJson(inputJson: jobInput)
let match = try jobMatchJson(inputJson: matchInput)
```

Inputs and outputs use the same versioned JSON contracts and canonical pretty bytes as the Rust CLI. This keeps the FFI surface stable and prevents generated Swift bindings from mirroring the complete internal Rust type graph. A future application may wrap these strings in app-owned `Codable` types.

Failures throw `CareerSwiftError`:

- `InvalidJson`
- `InputTooLarge`
- `InvalidInput` with core code, message, and field path
- `OutputSerialization`

Messages are bounded and do not include source documents. Single-document envelopes are limited to 262,144 UTF-8 bytes and match envelopes to 1,048,576 bytes before JSON parsing.

## Smoke target

```bash
swift run --package-path swift/CareerCoreSwift career-core-smoke
```

The smoke executable prints canonical `career.capabilities.v1` JSON. Swift tests execute every stable operation against the reviewed Rust fixtures and compare exact output bytes.

## Local application integration

Run the build script first, then add `swift/CareerCoreSwift` as a local package dependency in the future `career-workbench` Xcode project and import `CareerCore`.

The application—not this package or `career-core`—owns SwiftUI, persistence, document access, Keychain, provider clients, consent, and lifecycle behavior.

No package or XCFramework is published by Phase 6.
