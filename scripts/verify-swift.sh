#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/swift-rustup-toolchain.sh"
career_swift_select_rustup_toolchain

PACKAGE_DIR="$ROOT_DIR/swift/CareerCoreSwift"
ARTIFACTS_DIR="$PACKAGE_DIR/Artifacts"
RUST_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

for command in cmp git nm python3 shasum swift xcodebuild xcrun; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "error: required command is unavailable: $command" >&2
    exit 2
  fi
done

cp "$PACKAGE_DIR/Sources/CareerCore/CareerCore.swift" \
  "$TEMP_DIR/CareerCore.before.swift"
"$ROOT_DIR/scripts/build-swift-xcframework.sh"
cmp "$TEMP_DIR/CareerCore.before.swift" \
  "$PACKAGE_DIR/Sources/CareerCore/CareerCore.swift"

git -C "$ROOT_DIR" diff --exit-code -- \
  swift/CareerCoreSwift/Sources/CareerCore/CareerCore.swift

swift test --package-path "$PACKAGE_DIR"
swift run --package-path "$PACKAGE_DIR" career-core-smoke \
  > "$TEMP_DIR/swift-capabilities.json"
"$CAREER_SWIFT_CARGO" run \
  --quiet \
  --locked \
  --manifest-path "$ROOT_DIR/Cargo.toml" \
  --package career-cli \
  -- capabilities \
  > "$TEMP_DIR/rust-capabilities.json"
cmp "$TEMP_DIR/swift-capabilities.json" "$TEMP_DIR/rust-capabilities.json"

python3 - \
  "$ARTIFACTS_DIR/CareerCoreFFI.xcframework/Info.plist" \
  "$ARTIFACTS_DIR/CareerCoreFFI.metadata.json" <<'PY'
from pathlib import Path
import json
import plistlib
import sys

path = Path(sys.argv[1])
with path.open("rb") as file:
    document = plistlib.load(file)

slices = {
    (
        item["SupportedPlatform"],
        item.get("SupportedPlatformVariant"),
        tuple(item["SupportedArchitectures"]),
    )
    for item in document["AvailableLibraries"]
}
expected = {
    ("macos", None, ("arm64",)),
    ("ios", None, ("arm64",)),
    ("ios", "simulator", ("arm64",)),
}
if slices != expected:
    raise SystemExit(f"unexpected XCFramework slices: {slices!r}")

metadata = json.loads(Path(sys.argv[2]).read_text())
if metadata != {
    "schema_version": "career.swift_artifact_metadata.v1",
    "package_version": "0.1.1",
    "uniffi_version": "0.30.0",
    "deployment_targets": {"macos": "13.0", "ios": "16.0"},
    "rust_targets": [
        "aarch64-apple-darwin",
        "aarch64-apple-ios",
        "aarch64-apple-ios-sim",
    ],
}:
    raise SystemExit(f"unexpected artifact metadata: {metadata!r}")
PY

(
  cd "$ARTIFACTS_DIR"
  shasum -a 256 -c CareerCoreFFI.sha256
)

cat > "$TEMP_DIR/CareerCoreLinkSmoke.swift" <<'SWIFT'
public func careerCoreLinkSmoke() throws -> String {
    try capabilitiesJson()
}
SWIFT

link_apple_target() {
  local sdk="$1"
  local swift_target="$2"
  local rust_target="$3"
  local output_path="$4"
  xcrun --sdk "$sdk" swiftc \
    -emit-library \
    -parse-as-library \
    -module-name CareerCoreLinkSmoke \
    -target "$swift_target" \
    -sdk "$(xcrun --sdk "$sdk" --show-sdk-path)" \
    -Xcc "-fmodule-map-file=$RUST_TARGET_DIR/career-swift/headers/module.modulemap" \
    "$PACKAGE_DIR/Sources/CareerCore/CareerCore.swift" \
    "$TEMP_DIR/CareerCoreLinkSmoke.swift" \
    "$RUST_TARGET_DIR/$rust_target/release/libcareer_swift.a" \
    -o "$output_path"
  nm -gU "$output_path" > "$output_path.symbols"
  grep -q 'uniffi_career_swift_fn_func_capabilities_json' \
    "$output_path.symbols"
}

link_apple_target \
  iphoneos \
  arm64-apple-ios16.0 \
  aarch64-apple-ios \
  "$TEMP_DIR/libCareerCoreLinkSmoke-ios.dylib"
link_apple_target \
  iphonesimulator \
  arm64-apple-ios16.0-simulator \
  aarch64-apple-ios-sim \
  "$TEMP_DIR/libCareerCoreLinkSmoke-ios-simulator.dylib"

run_apple_build() {
  local destination="$1"
  local sdk="$2"
  local derived_data="$3"
  local log_path="$4"
  if ! (
    cd "$PACKAGE_DIR"
    xcodebuild \
      -scheme CareerCore \
      -destination "$destination" \
      -sdk "$sdk" \
      -derivedDataPath "$derived_data" \
      CODE_SIGNING_ALLOWED=NO \
      build
  ) > "$log_path" 2>&1; then
    cat "$log_path" >&2
    return 1
  fi
}

run_apple_build \
  "generic/platform=iOS" \
  iphoneos \
  "$TEMP_DIR/ios-derived-data" \
  "$TEMP_DIR/ios-build.log"
run_apple_build \
  "generic/platform=iOS Simulator" \
  iphonesimulator \
  "$TEMP_DIR/ios-simulator-derived-data" \
  "$TEMP_DIR/ios-simulator-build.log"

printf 'Swift binding verification passed.\n'
