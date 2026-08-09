# Release preparation checklist

This checklist defines the reviewed release process. Public `v0.1.0` is complete and immutable. Proposed `v0.1.1` is an active multi-PR preparation only: implementation agents may perform explicitly approved branch/PR work but must not merge, tag, dispatch a protected publication workflow, authenticate to npm, release, or publish. The final `v0.1.1` release checklist and protected workflow remain deferred until every exact native target has execution evidence.

The current release maintenance owner is [Revaz Zakalashvili](https://github.com/revazi). Ownership and governance are documented in [`../MAINTAINERS.md`](../MAINTAINERS.md).

## 1. Approve scope and platforms

- choose one reviewed commit from synchronized `main`
- choose a SemVer version and update manifests/changelog intentionally
- record the supported target triples and maintenance owner
- confirm both licenses and security documentation are included
- obtain explicit approval to create and upload release artifacts
- for npm, prove control of `@revazi`, review exact `@revazi/career@0.1.0`, public access, bootstrap/trusted-publisher ownership, and the explicit absent independent-binary-signature policy

Maintainer observations from 2026-08-09—`npm view` `E404` for all three names, `npm whoami` `E401`, and no GitHub environment/secrets/variables—showed apparent name availability but did not prove `@revazi` ownership. Successful authenticated scope verification remains mandatory. No platform is implied merely because it can compile in CI.

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

Use `career.exe` and a ZIP archive if a future Windows target is separately approved. Do not cross-compile an artifact and label it supported without executing CLI smoke tests on that platform.

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

For proposed `0.1.1`, registry-free tests may assemble an exact nine-package synthetic policy candidate: eight ordered native tarballs followed by the launcher. Only the current host tarball executes in that local test; every synthetic non-host tarball is marked as policy-only evidence and cannot satisfy release acceptance. A real candidate remains blocked until native jobs produce and execute all eight exact packages. The protected workflow remains the completed `v0.1.0` workflow until the separately approved final publication PR prepares exact `v0.1.1` source/tag/SHA, runner, assembly, trusted-publisher/bootstrap, and public-acceptance gates.

Keep private local verification available:

```bash
npm_output="$(mktemp -d)"
trap 'rm -rf "$npm_output"' EXIT
scripts/prepare-npm-cli-packages.sh --output-dir "$npm_output"
node --test npm/tests/launcher.test.js
scripts/test-npm-cli-packages.sh
scripts/test-npm-publication.sh
```

Source templates must remain `private: true`. Never hand-edit a tarball. The completed `v0.1.0` candidate path required exact clean fetched `origin/main`, reviewed SHA, annotated unmoved tag, Node 22.19.0/npm 11.6.2, exact rustc/Cargo 1.97.1, and its two approved native runners. Proposed `v0.1.1` retains those source/toolchain principles but cannot finalize runner labels, GNU symbol records, musl linkage, Windows rules, or nine-package workflow inputs until the preceding native sub-gates pass. Rust 1.85 remains a separate MSRV check.

The following protected `.github/workflows/npm-publish.yml` dispatch inputs describe the completed `v0.1.0` path and are not authorization to dispatch proposed `v0.1.1`:

- `reviewed_sha`: exact 40-character tagged `origin/main` commit;
- `bootstrap=false`: default OIDC-only steady state;
- `bootstrap=true`: explicitly approved temporary granular-token first publication or interrupted-bootstrap recovery.

The `npm-production` environment must require maintainer approval. The workflow builds and executes both native packages, assembles exactly Darwin native, Linux native, then `@revazi/career`, validates package and provenance bytes, and publishes with `--access public --provenance --ignore-scripts`. The publish job has only read-only contents plus `id-token: write`; it checks out only the tracked publish driver. No project dependency install, crate publication, signing, notarization, GitHub Release, or custom release asset is allowed.

#### One-time first-publication bootstrap

Package-level trusted publishers cannot be configured before each package exists. Before the first dispatch:

1. Prove npm account/scope ownership independently of the prior `E404` results.
2. Create one shortest-lived granular npm token with only the minimum public-publish access available for these three package names/scope. Do not use it for interactive trusted-publisher configuration.
3. Create protected GitHub environment `npm-production` and add the temporary environment secret `NPM_TOKEN`; add no repository-level token fallback.
4. Dispatch exact `v0.1.0`/reviewed SHA with `bootstrap=true`.

Bootstrap accepts only an absent package name or exact already-published `name@0.1.0` with matching candidate integrity and valid npm SLSA provenance. This permits safe rerun after interruption. A name with no exact reviewed version, conflicting integrity, or missing/malformed registry provenance blocks publication. Native packages always finish before the launcher.

A successful `npm publish` response is not treated as complete until exact public integrity and valid SLSA provenance become visible. Each package gets one combined bounded readiness loop of 61 attempts at ten-second intervals (at most ten minutes of sleeps shared by integrity and provenance); the protected publish job allows 40 minutes for all three sequential packages plus setup and bounded transient retries. This bound reflects the 3–5 minute propagation observed during the `v0.1.0` bootstrap while still failing closed on absence, conflict, malformed provenance, or timeout.

Immediately after successful bootstrap, delete the GitHub environment secret and revoke/delete the granular token in npm. For example, after confirming the workflow result:

```bash
gh secret delete NPM_TOKEN --env npm-production --repo revazi/career-core
```

Token revocation in the npm account is also mandatory; the GitHub command alone does not revoke it.

#### Configure steady-state trusted publishing

Trusted-publisher setup is post-bootstrap only. Use exact Node 22.19.0 and exact npm CLI 11.15.0 with supported interactive npm authentication and account 2FA; do not use a bypass-2FA granular access token for `npm trust`. This is a narrow reviewed maintainer-only exception: publication and public acceptance remain pinned to npm 11.6.2, while official `npm trust` is unavailable before npm 11.15.0 and is never executed in the publication workflow. It performs no Career Core package build or publication.

The reviewed npm 11.15.0 registry integrity observed on 2026-08-09 is:

```text
sha512-+k0tk7lRnpMUPnC7kTuU/yrV/mnFoPhJQ75VfLtZ6fwbzOVXaPsTE/Il9Pn1DHi482byMyqkHv/XsQ76mNjXLw==
```

Record that value and the successful comparison in the dated release notes. Install the trust-only CLI into a temporary prefix, with scripts disabled, only after exact verification:

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

"$trust_npm" trust github @revazi/career-darwin-arm64 \
  --repo revazi/career-core \
  --file npm-publish.yml \
  --environment npm-production \
  --allow-publish
"$trust_npm" trust github @revazi/career-linux-x64-gnu \
  --repo revazi/career-core \
  --file npm-publish.yml \
  --environment npm-production \
  --allow-publish
"$trust_npm" trust github @revazi/career \
  --repo revazi/career-core \
  --file npm-publish.yml \
  --environment npm-production \
  --allow-publish

"$trust_npm" trust list @revazi/career-darwin-arm64
"$trust_npm" trust list @revazi/career-linux-x64-gnu
"$trust_npm" trust list @revazi/career
```

Review each list result for repository `revazi/career-core`, workflow file `npm-publish.yml`, environment `npm-production`, and publish permission. Keep `NPM_TOKEN` absent, then use only default `bootstrap=false`. OIDC mode creates no auth-token npmrc and fails if `NODE_AUTH_TOKEN` is present or any package name lacks the configured trusted-publisher prerequisite.

Publication success additionally requires exact registry integrity plus bounded validation of `dist.attestations.provenance.predicateType == https://slsa.dev/provenance/v1` and the registry attestations URL. npm registry provenance, npm registry signatures, package-contained SHA-256, and the explicitly absent independent native-binary signature are distinct.

The workflow is not complete until no-secret public-registry acceptance passes on both approved runners using only exact:

```bash
npx --yes --package=@revazi/career@0.1.0 career --version
```

The smokes confirm the selected internal optional package and deterministic version/discovery/schema/resume/job parity without a source or PATH binary fallback.

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
- merge the reviewed change to `main`, then create annotated `v0.1.0` only at that exact commit
- run only the protected manual npm workflow with the exact reviewed SHA/mode
- require both no-secret public-registry acceptance jobs to pass before declaring npm complete
- only after both public acceptance jobs pass, create the GitHub Release for annotated `v0.1.0` with dated notes identifying exact `@revazi/career@0.1.0`, source SHA, npm provenance, supported hosts/glibc floor, and known limitations
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
