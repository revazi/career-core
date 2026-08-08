# Installation and distribution

`career` is local-first software. The project does not provide a remote execution service or an installer that pipes network content into a shell.

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

## External integrations

The optional native Pi package and bundled Agent Skill are maintained in the separate [`pi-career`](https://github.com/revazi/pi-career) repository. Career Core is not a Pi package; use the external repository's reviewed installation and security guidance rather than registering this checkout.

## Maintainer-only runtime inputs for `pi-career`

`pi-career` may track reviewed native `career` binaries so its users need neither a separately installed CLI nor a Rust toolchain. Career Core provides a bounded preparation mechanism for that repository, not a public installation channel:

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

Homebrew, Cargo registry publication, and other package-manager channels are not currently supported. Each channel requires an explicit maintenance and update plan before it can be documented as available.

Maintainers preparing an approved release must follow [`releasing.md`](releasing.md).

## Support and maintenance

Career Core is primarily maintained by [Revaz Zakalashvili](https://github.com/revazi). Use the repository's [issue tracker](https://github.com/revazi/career-core/issues) for sanitized support requests, [`../SECURITY.md`](../SECURITY.md) for private vulnerability reporting, and [`../MAINTAINERS.md`](../MAINTAINERS.md) for ownership and governance details.
