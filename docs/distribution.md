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

The script uses a temporary Cargo root, executes capability/schema discovery plus representative resume, job, and Phase 7 operations from a temporary working directory, and removes the temporary installation on exit.

## External integrations

The optional native Pi package and bundled Agent Skill are maintained in the separate [`pi-career`](https://github.com/revazi/pi-career) repository. Career Core is not a Pi package; use the external repository's reviewed installation and security guidance rather than registering this checkout.

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
