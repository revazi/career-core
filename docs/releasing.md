# Release preparation checklist

This checklist defines a repeatable release process but does not authorize publication. Creating a tag, GitHub release, crate publication, or binary upload requires explicit maintainer approval.

## 1. Approve scope and platforms

- choose one reviewed commit from synchronized `main`
- choose a SemVer version and update manifests/changelog intentionally
- record the supported target triples and maintenance owner
- confirm both licenses and security documentation are included
- obtain explicit approval to create and upload release artifacts

No platform is implied merely because it can compile in CI.

## 2. Verify a clean source tree

```bash
git switch main
git pull --ff-only
test -z "$(git status --porcelain)"
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Run every CLI golden command listed in [`../AGENTS.md`](../AGENTS.md), export every embedded schema, and validate the schema catalog plus all exported documents with an independent Draft 2020-12 validator.

Repeat installation from a clean clone:

```bash
install_root="$(mktemp -d)"
cargo install --path crates/career-cli --locked --root "$install_root"
"$install_root/bin/career" --version
"$install_root/bin/career" capabilities --format json-compact
rm -rf "$install_root"
```

## 3. Build on each approved native runner

Set explicit values rather than relying on host inference:

```bash
version="REVIEWED-SEMVER"
target="REVIEWED-RUST-TARGET"
artifact="career-v${version}-${target}"

cargo build --release --locked -p career-cli --target "$target"
mkdir -p "dist/$artifact"
cp "target/$target/release/career" "dist/$artifact/career"
cp LICENSE-MIT LICENSE-APACHE README.md THIRD_PARTY_NOTICES.md "dist/$artifact/"
```

Use `career.exe` and a ZIP archive if a future Windows target is separately approved. Do not cross-compile an artifact and label it supported without executing CLI smoke tests on that platform.

Before packaging, run the built binary directly:

```bash
"dist/$artifact/career" --version
"dist/$artifact/career" capabilities --format json-compact
"dist/$artifact/career" schema export \
  --id career.job_match.v1 \
  --format json-compact
```

Create one archive per target using the same reviewed build environment and naming convention:

```bash
tar -C dist -czf "dist/${artifact}.tar.gz" "$artifact"
tar -tzf "dist/${artifact}.tar.gz"
```

Record the runner image, Rust version, target triple, source commit, and exact command in the release notes. Bit-for-bit reproducibility across different archive implementations is not currently claimed.

### Swift artifact preparation

If Swift/XCFramework publication is separately approved, run on a reviewed Apple Silicon macOS runner with the documented Xcode and Rust toolchains:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
scripts/verify-swift.sh
```

Inspect `CareerCoreFFI.xcframework/Info.plist`, verify exactly the approved slices, regenerate checked-in Swift source without a diff, and include `THIRD_PARTY_NOTICES.md`. Do not upload the ignored local XCFramework directly from an unreviewed working tree. Define the package URL, archive format, checksum, tag alignment, signing policy, and supported deployment targets in the separately approved release plan.

## 4. Generate and verify checksums

After collecting all approved archives in one clean `dist/` directory:

```bash
(
  cd dist
  : > SHA256SUMS
  for asset in career-v*.tar.gz career-v*.zip; do
    test -f "$asset" || continue
    shasum -a 256 "$asset" >> SHA256SUMS
  done
  test -s SHA256SUMS
  shasum -a 256 -c SHA256SUMS
)
```

Shell glob order keeps the manifest stable for the same asset names. Review `SHA256SUMS`; it must contain only intended release assets and no source documents, logs, temporary files, or credentials.

## 5. Publish only after final approval

- verify CI is green for the exact source commit
- review archive contents and execute each binary on its supported platform
- create the annotated tag and GitHub release only after approval
- upload archives and `SHA256SUMS` together
- publish no crate or package-manager formula unless separately approved
- verify public downloads against the uploaded checksum manifest
- update installation documentation only for artifacts that actually exist

## 6. Record provenance

Release notes must include:

- version, source commit, and tag
- core and public schema versions
- supported platforms and target triples
- Rust toolchain and runner images
- SHA-256 checksums
- known limitations and compatibility impact
- confirmation that the binary performs no implicit network requests

Stop and discard artifacts if the source tree, lockfile, version, tests, schema export, archive contents, or checksums differ from the approved inputs.
