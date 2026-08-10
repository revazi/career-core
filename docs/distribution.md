# Installation and distribution

`career` is local-first software. The project does not provide a remote execution service or an installer that pipes network content into a shell. Exact `@revazi/career@0.1.1` is the completed public npm release for eight native targets after protected run `31346152236` passed publication and all eight public-registry acceptance jobs.

## Run from a source checkout

Requirements: Rust 1.85 or newer and the reviewed lockfile.

```bash
git clone https://github.com/revazi/career-core.git
cd career-core
cargo run --quiet --locked -p career-cli -- capabilities
```

Check out a reviewed commit or release tag when reproducibility matters.

## Install locally from a checkout

```bash
cargo install --path crates/career-cli --locked
career --version
career capabilities --format json-compact
career operations --format json-compact
career schema bundle --id career.job_match_input.v1 --format json-compact
```

Cargo normally installs to `$HOME/.cargo/bin`. To use an isolated prefix:

```bash
install_root="$HOME/.local"
cargo install --path crates/career-cli --locked --root "$install_root"
"$install_root/bin/career" capabilities --format json-compact
```

This installation builds only the CLI and its inward dependency on `career-core`. It adds no provider, telemetry, TLS, or network-runtime dependency.

Verify the same installed boundary outside the checkout with:

```bash
scripts/verify-installed-cli.sh
```

The script uses a temporary Cargo root, executes capability/operation/schema discovery, emits every self-contained bundle from a temporary working directory outside the checkout, checks complete mapping/local refs/bounds independently, runs representative resume, job, and Phase 7 operations, and removes the temporary installation on exit.

## npm package

The only user-facing npm package is exact `@revazi/career@0.1.1`, exposing bin `career`:

```bash
npx --yes --package=@revazi/career@0.1.1 career --version
npx --yes --package=@revazi/career@0.1.1 career capabilities --format json-compact
```

Always use an exact reviewed version, never `latest` or a range. npm/npx acquisition may contact npm; launcher runtime remains offline. Native platform packages are internal lockstep optional dependencies. Users and consumers must not install, invoke, or pin them directly.

### `0.1.1` native matrix

Source version `0.1.1` contains one reviewed ordered catalog and private templates for Darwin ARM64/x64, GNU Linux x64/ARM64, musl Linux x64/ARM64, and MSVC Windows x64/ARM64. The launcher positively distinguishes glibc, musl, and unknown libc; unknown never falls back to musl. It validates exact package/version/target/provenance, regular non-symlink files, target-specific Unix or Windows invariants, bounded size, Mach-O/ELF/PE architecture, and SHA-256 before direct execution.

Protected `v0.1.1` run `31346152236` recompiled, executed, inspected, packaged, published, reacquired, and executed all eight final-tag native packages on their exact target classes. Synthetic fixtures still prove policy only; cross-compilation, emulation, package metadata, and skipped jobs are not support evidence for later releases.

GNU builds use native Ubuntu 22.04 userland and reject imports above the glibc 2.35 floor. Musl builds use immutable Node 22.19.0 Alpine 3.22 on architecture-matched runners and require exact musl 1.2.5 plus the observed target-specific static form: x64 ELF `DYN` static PIE with `NOW PIE` flags or AArch64 ELF `EXEC`, both without an interpreter or shared-library imports. Windows builds use native `windows-2025` x64 and `windows-11-arm` ARM64, exact process/toolchain architecture agreement, `career.exe` regular non-symlink semantics without a Unix mode claim, and bounded PE32+/machine/reviewed-system-DLL import evidence. See [`contracts/npm-cli-distribution-v2.md`](contracts/npm-cli-distribution-v2.md).

Checked-in templates remain `private: true` even after release. Private current-host tests are prepared outside the checkout with:

```bash
package_dir="$(mktemp -d)"
trap 'rm -rf "$package_dir"' EXIT
scripts/prepare-npm-cli-packages.sh --output-dir "$package_dir"
```

`--allow-dirty` is local-test-only and records a private dirty non-candidate. Stable candidate scripts derive the release version from exact annotated `vX.Y.Z` source, require the reviewed `origin/main` SHA, Node 22.19.0/npm 11.6.2, rustc/Cargo 1.97.1, exact native runners, and external empty outputs. `npm-release.yml` supplies exactly eight ordered native tarballs, assembles the launcher ninth and last, and publishes through OIDC only after `npm-production` approval. Candidate commands themselves do not query or publish to npm.

Run registry-free package/candidate/publish-driver verification with local reviewed Node 22.19.0 and npm 10.9.3 or publication npm 11.6.2:

```bash
node --test npm/tests/launcher.test.js
scripts/test-npm-cli-packages.sh
scripts/test-npm-publication.sh
```

The package-contained SHA-256 is consistency evidence only. npm tarball integrity and npm SLSA registry provenance are separate acquisition evidence. No independent native-binary signature exists. Future releases use stable OIDC-only `npm-release.yml`; version-specific workflows remain historical. See [`contracts/npm-cli-distribution-v2.md`](contracts/npm-cli-distribution-v2.md) and [`releasing.md`](releasing.md).

## External integrations

The optional native Pi package and bundled Agent Skill are maintained in the separate [`pi-career`](https://github.com/revazi/pi-career) repository. Career Core is not a Pi package; use the external repository's reviewed installation and security guidance rather than registering this checkout. The future npm runner contract and transition gate are in [`pi-career-npm-handoff.md`](pi-career-npm-handoff.md).

## Transitional maintainer-only runtime inputs for `pi-career`

Until the npm handoff removal gate passes, `pi-career` may track reviewed native `career` binaries so its users need neither a separately installed CLI nor a Rust toolchain. Career Core retains this transitional bounded preparation mechanism for that repository, not a public installation channel:

- `.github/workflows/pi-career-runtime-artifacts.yml` runs only by explicit `workflow_dispatch` and is separate from normal Core CI.
- `scripts/prepare-pi-career-runtime-artifact.sh` provides the same native preparation and verification locally.
- Approved targets are limited to `x86_64-unknown-linux-gnu` on native `ubuntu-latest` x86_64 and `aarch64-apple-darwin` on native `macos-14` Apple Silicon. The script fails closed on every other host/target pairing.
- Each locked release build is executed for version, capability/operation/schema discovery, operation-catalog schema export, four representative recursive bundles, deterministic resume analysis, and deterministic job matching against synthetic goldens before packaging.
- The archive contains only `career`, `metadata.json`, `LICENSE-MIT`, `LICENSE-APACHE`, and `THIRD_PARTY_NOTICES.md`.
- Metadata retains exact source SHA, dirty state, Rust/Cargo and host/target details, executable size/hash, license/notice records, existing contract digests, and representative native results.
- Its versioned `career.pi_career_managed_adapter_compatibility.v1` record additionally proves operation-catalog schema/output digests, the complete available capability-to-operation mapping, representative bundle IDs/digests, every declared operation output bound, and representative deterministic output digests. Shell tests reject tampered catalog, mapping, bundle, bound, and representative-output metadata.
- Workflow artifacts have three-day retention and are unsigned maintainer handoff inputs. A separate `pi-career` change must review and track them before use.

On a clean reviewed checkout, the local equivalent requires an output directory outside the repository:

```bash
artifact_dir="$(mktemp -d)"
trap 'rm -rf "$artifact_dir"' EXIT
scripts/prepare-pi-career-runtime-artifact.sh --output-dir "$artifact_dir"
```

Artifact names contain the platform key, target triple, and full Core commit. This workflow does not create a tag, GitHub Release, installer, public download, signature, notarization, npm/crate publication, package-manager channel, or support claim for any other architecture. Do not direct Core end users to workflow artifacts.

## Uninstall a local CLI

Remove a default Cargo installation with:

```bash
cargo uninstall career-cli
```

For an isolated `cargo install --root`, remove only that user-controlled install root after verifying it contains no other tools. CLI removal does not claim to erase shell history, backups, or separately saved results.

## Release binaries

No release binary is published until a maintainer explicitly approves a release. When binaries become available, download the archive and `SHA256SUMS` from the same GitHub release page. Do not pipe an installer into a shell.

Select the exact downloaded archive entry first:

```bash
asset="career-vVERSION-REVIEWED-RUST-TARGET.tar.gz"
grep -F "  $asset" SHA256SUMS > "$asset.sha256"
test "$(wc -l < "$asset.sha256" | tr -d ' ')" = "1"
```

Verify on Linux:

```bash
sha256sum --check "$asset.sha256"
```

Verify on macOS:

```bash
shasum -a 256 -c "$asset.sha256"
```

Inspect the archive before extracting it, then place `career` in a directory already under your control and on `PATH`. Confirm the exact version and offline capability document:

```bash
career --version
career capabilities --format json-compact
```

Checksums detect accidental corruption and substitution relative to the release manifest; they are not a substitute for verifying the GitHub release source and tag.

## Package managers

Exact accepted `@revazi/career@0.1.1` is the sole approved package-manager consumer surface. Protected npm workflows publish no crate, Homebrew formula, GitHub Release asset, signature, or other channel. Every other package-manager channel requires separate maintenance, ownership, security, and publication approval.

Maintainers preparing an approved release must follow [`releasing.md`](releasing.md).

## Support and maintenance

Career Core is primarily maintained by [Revaz Zakalashvili](https://github.com/revazi). Use the repository's [issue tracker](https://github.com/revazi/career-core/issues) for sanitized support requests, [`../SECURITY.md`](../SECURITY.md) for private vulnerability reporting, and [`../MAINTAINERS.md`](../MAINTAINERS.md) for ownership and governance details.
