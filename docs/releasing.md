# Release preparation checklist

This checklist defines the reviewed release process. Public `v0.1.0` and `v0.1.1` release workflows remain immutable historical evidence. Future npm versions use one stable OIDC-only workflow. Implementation agents must not create tags or GitHub Releases, dispatch publication, authenticate/query/publish npm, or use release credentials. Those actions belong to the parent maintainer after exact-head review and CI.

The current release maintenance owner is [Revaz Zakalashvili](https://github.com/revazi). Ownership and governance are documented in [`../MAINTAINERS.md`](../MAINTAINERS.md).

## 1. Approve scope and platforms

- choose one reviewed commit from synchronized `main`
- choose a SemVer version and update manifests/changelog intentionally
- record the supported target triples and maintenance owner
- confirm both licenses and security documentation are included
- obtain explicit approval to create and upload release artifacts
- for npm, review exact lockstep `@revazi/career@X.Y.Z` plus all eight internal packages, public access, stable-workflow trusted-publisher ownership, and the explicit absent independent-binary-signature policy

Completed `v0.1.1` publication established all nine package names. Future publication is OIDC-only through the stable workflow; no bootstrap credential is part of routine release policy. No platform is supported merely because it compiles.

## 2. Verify a clean source tree

```bash
git switch main
git pull --ff-only
test -z "$(git status --porcelain)"
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --all-features --locked
node --test npm/tests/launcher.test.js
scripts/test-npm-cli-packages.sh
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Run every CLI golden command listed in [`../AGENTS.md`](../AGENTS.md), preserve the capability and operation-catalog goldens, export every embedded schema, emit every self-contained bundle, and run `scripts/verify-managed-adapter-contracts.sh target/debug/career` with pinned `check-jsonschema` 0.34.1. The independent gate validates every bundle against the Draft 2020-12 metaschema and representative instances.

Repeat installation from a clean clone:

```bash
install_root="$(mktemp -d)"
cargo install --path crates/career-cli --locked --root "$install_root"
"$install_root/bin/career" --version
"$install_root/bin/career" capabilities --format json-compact
"$install_root/bin/career" operations --format json-compact
"$install_root/bin/career" schema bundle \
  --id career.job_match_input.v1 --format json-compact
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

Windows uses exact `career.exe`; npm packages remain tarballs with archive mode `0644` and no POSIX runtime-mode claim. Do not cross-compile an artifact and label it supported without native execution.

Before packaging, run the built binary directly:

```bash
"dist/$artifact/career" --version
"dist/$artifact/career" capabilities --format json-compact
"dist/$artifact/career" operations --format json-compact
"dist/$artifact/career" schema export \
  --id career.job_match.v1 \
  --format json-compact
"dist/$artifact/career" schema bundle \
  --id career.job_match_input.v1 \
  --format json-compact
```

Create one archive per target using the same reviewed build environment and naming convention:

```bash
tar -C dist -czf "dist/${artifact}.tar.gz" "$artifact"
tar -tzf "dist/${artifact}.tar.gz"
```

Record the runner image, Rust version, target triple, source commit, and exact command in the release notes. Bit-for-bit reproducibility across different archive implementations is not currently claimed.

### npm publication candidate and protected workflow

Exact `@revazi/career@0.1.1` and its eight internal native packages completed protected publication and all-eight native public acceptance in run `31346152236`. The release-specific `npm-publish.yml` and `npm-publish-v0.1.1.yml` workflows are historical evidence only. Do not copy, edit, or dispatch them for a later version.

Routine npm releases use only `.github/workflows/npm-release.yml`. It derives `X.Y.Z` from the selected annotated `vX.Y.Z` tag; stable SemVer without a prerelease/build suffix is required. The source gate requires a clean exact fetched `origin/main`, the explicit reviewed SHA, an annotated tag resolving to that SHA, and exact lockstep versions in root/workspace Cargo metadata, `Cargo.lock`, the launcher, all eight native templates, and launcher optional dependencies. Filenames, candidate metadata, provenance refs, package acquisition, and acceptance expectations derive from that version rather than a workflow constant.

The stable workflow repeats all final-source evidence on exact native hosts: macOS ARM64/x64, GNU Linux ARM64/x64, digest-pinned Alpine 3.22/musl 1.2.5 x64/ARM64, and MSVC Windows x64/ARM64. It assembles eight native packages in catalog order and the public launcher ninth. Every package is byte/integrity/provenance checked, and every public package is reacquired and executed on its exact native target class. Skipped, cross-compiled, emulated, synthetic, or metadata-only evidence blocks release support.

Keep registry-free verification available:

```bash
npm_output="$(mktemp -d)"
trap 'rm -rf "$npm_output"' EXIT
scripts/prepare-npm-cli-packages.sh --output-dir "$npm_output"
node --test npm/tests/launcher.test.js
scripts/test-npm-cli-packages.sh
scripts/test-npm-publication.sh
```

Source templates remain `private: true`; generated public manifests, binaries, provenance, and tarballs remain outside the checkout. Never hand-edit a tarball. Publication uses exact Node 22.19.0/npm 11.6.2 and rustc/Cargo 1.97.1, with Rust 1.85 checked separately as the MSRV.

#### Stable OIDC publishing setup

All nine package names now exist. Routine releases therefore have no bootstrap mode, npm token input, secret fallback, or token-authenticated npmrc. The publish job grants only `contents: read` and `id-token: write`, checks out only the reviewed driver, and invokes npm trusted publishing with `--access public --provenance --ignore-scripts`.

Configure each npm package's GitHub trusted publisher once for:

- repository: `revazi/career-core`
- workflow file: `npm-release.yml`
- environment: `npm-production`
- permission: publish

Use exact npm CLI 11.15.0 for the separately authenticated `npm trust` maintenance operation; publication and public acceptance remain pinned to npm 11.6.2. Verify the previously reviewed npm CLI integrity, install it into a temporary prefix with scripts disabled, and use interactive account authentication plus 2FA:

```bash
test "$(node --version)" = "v22.19.0"
expected_trust_npm_integrity='sha512-+k0tk7lRnpMUPnC7kTuU/yrV/mnFoPhJQ75VfLtZ6fwbzOVXaPsTE/Il9Pn1DHi482byMyqkHv/XsQ76mNjXLw=='
actual_trust_npm_integrity="$(
  npm view npm@11.15.0 dist.integrity --json |
    node -e 'const fs=require("node:fs"); process.stdout.write(JSON.parse(fs.readFileSync(0,"utf8")))'
)"
test "$actual_trust_npm_integrity" = "$expected_trust_npm_integrity"
trust_npm_root="$(mktemp -d)"
trap 'rm -rf "$trust_npm_root"' EXIT
npm install --global --prefix "$trust_npm_root" --ignore-scripts npm@11.15.0
trust_npm="$trust_npm_root/bin/npm"
test "$($trust_npm --version)" = "11.15.0"
unset NODE_AUTH_TOKEN NPM_TOKEN
"$trust_npm" login

for package in \
  @revazi/career-darwin-arm64 \
  @revazi/career-darwin-x64 \
  @revazi/career-linux-x64-gnu \
  @revazi/career-linux-arm64-gnu \
  @revazi/career-linux-x64-musl \
  @revazi/career-linux-arm64-musl \
  @revazi/career-win32-x64-msvc \
  @revazi/career-win32-arm64-msvc \
  @revazi/career
do
  "$trust_npm" trust github "$package" \
    --repo revazi/career-core \
    --file npm-release.yml \
    --environment npm-production \
    --allow-publish
  "$trust_npm" trust list "$package"
done
```

Review every result. No `NPM_TOKEN` or `NODE_AUTH_TOKEN` may be present during publication.

Protect release tags through a repository ruleset for `v*`. Configure `npm-production` with required maintainer approval and one deployment-tag rule `v*`; do not add one environment rule per release. Environment approval is defense in depth, not a substitute for the workflow's annotated-tag/SHA/main/version checks.

#### Routine npm release

After the reviewed version bump is merged and exact-head CI is green:

```bash
git switch main
git pull --ff-only
test -z "$(git status --porcelain)"
version="X.Y.Z"
git tag -a "v$version" -m "Career Core v$version"
git push origin "v$version"
reviewed_sha="$(git rev-list -n 1 "v$version")"
gh workflow run npm-release.yml \
  --ref "v$version" \
  -f reviewed_sha="$reviewed_sha"
```

The workflow is idempotent only for exact candidate bytes. Existing matching versions with valid SLSA provenance are verified and skipped; conflicts fail closed. Missing package names fail because routine OIDC publishing requires all nine trusted-publisher identities to exist. Native packages always publish before the launcher.

Each `npm publish` subprocess is actively bounded to five minutes. Registry integrity/provenance visibility is logged and checked up to 61 times at ten-second intervals per package. A successful npm response is not completion until exact SHA-512 integrity and `https://slsa.dev/provenance/v1` registry attestations are visible. The overall publish job remains bounded at 110 minutes for nine serial packages and bounded retries.

The release is complete only after all eight no-secret public acceptance jobs pass using exact `@revazi/career@X.Y.Z`. npm registry provenance, package-contained SHA-256 consistency, and the explicitly absent independent native-binary signature remain distinct. The parent maintainer may then create the annotated-tag GitHub Release with dated notes and no custom binary assets.

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

- verify local review and CI are green for the exact source commit
- merge the reviewed version change to `main`, then create annotated `vX.Y.Z` only at that exact commit
- run only `npm-release.yml` with the selected tag and exact reviewed SHA
- require all eight no-secret public-registry acceptance jobs to pass before declaring npm complete
- keep all nine trusted publishers bound to `npm-release.yml` and keep token credentials absent
- only then create the GitHub Release for annotated `vX.Y.Z` with dated notes identifying exact `@revazi/career@X.Y.Z`, source SHA, npm provenance, supported hosts/glibc floor, and known limitations
- attach no custom binary asset, checksum archive, signature, notarization, crate publication, or formula to that GitHub Release
- update installation status only for the exact package/version that passed public acceptance

## 6. Record provenance

Release notes must include:

- version, source commit, and tag
- core and public schema versions
- supported platforms and target triples
- Rust toolchain and runner images
- candidate SHA-256/SHA-512 integrity and npm registry SLSA provenance results
- known limitations and compatibility impact
- confirmation that the binary performs no implicit network requests

Stop and discard artifacts if the source tree, lockfile, version, tests, schema export, archive contents, or checksums differ from the approved inputs.
