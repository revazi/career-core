# npm CLI distribution v1

Phase 9 distributes the unchanged native `career` CLI through one user-facing npm package without changing Core algorithms, schemas, capability/operation discovery, output ordering, warnings, evidence, uncertainty, assisted authority, the 33,554,432-byte successful machine-output ceiling, or Swift behavior.

## Consumer surface

The complete npm consumer surface is:

- package: `@revazi/career`
- version: `0.1.0`
- bin: `career`
- release tag: annotated `v0.1.0`
- launcher runtime: Node `>=22`

Users and external consumers install or address only `@revazi/career`. The Darwin ARM64 and Linux x64 glibc packages named in launcher metadata are internal exact-version optional-dependency implementation details. Consumers must not install, invoke, resolve, or pin those packages directly.

The exact one-off command is:

```bash
npx --yes --package=@revazi/career@0.1.0 career <args>
```

Use the exact version, never `latest`. npm/npx acquisition may contact the npm registry; the installed launcher and native CLI perform no network requests.

As observed by the maintainer on 2026-08-09, `npm view` returned `E404` for all three names, `npm whoami` returned `E401`, and the GitHub repository had no Actions secrets, variables, or environments. The `E404` results are availability evidence only; the `E401` means `@revazi` ownership remains unproven until successful authentication. Scope ownership is an external publication prerequisite.

## Source and publication manifests

Every checked-in package template retains `private: true`, including after publication. Directly running `npm publish` against a source template fails safely.

Only `scripts/prepare-npm-publication-native.sh` and `scripts/assemble-npm-publication-candidate.sh` may create publishable manifests, and only in an output directory outside the checkout. They require:

- exact version `0.1.0`, ref `refs/tags/v0.1.0`, and an explicit reviewed 40-character SHA;
- an annotated—not lightweight—`v0.1.0` tag resolving to the reviewed `HEAD`;
- the exact fetched `origin/main` commit equal to `HEAD`;
- canonical credential-free HTTPS origin `https://github.com/revazi/career-core` with optional `.git` suffix;
- a clean tracked and untracked worktree;
- exact private source manifests and lockstep versions;
- exact Node `22.19.0`, npm `11.6.2`, rustc `1.97.1`, and Cargo `1.97.1` for publication candidates; and
- one approved matching native host: `macos-14` ARM64 or `ubuntu-22.04` x64.

Candidate staging removes the `private` property only in the external copy and adds exactly:

```json
{
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

Candidate manifests have exact reviewed property sets, no lifecycle scripts, no `dependencies`, `devDependencies`, or `peerDependencies`, and no runtime code dependency other than the launcher's exact two `optionalDependencies`.

Private local testing remains available through `scripts/prepare-npm-cli-packages.sh --allow-dirty`. It preserves `private: true`, records `git_dirty: true`, keeps tag/ref null, and always emits `publication_candidate: false`.

## Exact target selection

| Node host | Local evidence | Rust target | Support |
|---|---|---|---|
| `darwin` / `arm64` | n/a | `aarch64-apple-darwin` | supported |
| `linux` / `x64` | bounded `process.report` version at least `2.35` | `x86_64-unknown-linux-gnu` | supported |

The Linux package contract and provenance encode `minimum_glibc_version: "2.35"`. It is built on pinned `ubuntu-22.04`, records exact `glibc 2.35`, and fails candidate preparation if `readelf` finds a required GLIBC symbol version above `2.35`. Older, malformed, musl, unknown, or unavailable libc evidence returns `CAREER_NPM_UNSUPPORTED_LIBC` before package resolution or launch. No broader Linux compatibility is claimed.

Every other platform/architecture returns `CAREER_NPM_UNSUPPORTED_PLATFORM`. There is no fallback target.

After selection, the launcher resolves only:

```text
require.resolve(<selected-internal-package>/package.json, { paths: [launcherDirectory] })
```

It never searches `PATH`, downloads a binary, runs an installer, or substitutes another package.

## Native package and provenance

Each internal native package contains exactly:

```text
package.json                       0644
career                             0755
provenance.json                    0644
LICENSE-MIT                        0644
LICENSE-APACHE                     0644
THIRD_PARTY_NOTICES.md             0644
```

`career.npm_native_package.v1` records exact platform, architecture, Rust target, binary/provenance filenames, mode, 16 MiB maximum, and Linux minimum glibc 2.35.

`career.npm_native_provenance.v1` has an exact top-level and nested property set:

```text
schema_version
package
  name, version, platform_key, node_platform, node_arch, rust_target,
  minimum_glibc_version
source
  repository, git_sha, git_ref, git_tag, git_dirty, publication_candidate
build
  command, profile, locked, rustc_version, cargo_version
  runner
    os, arch, image, libc
executable
  file_name, binary_format, mode, size_bytes, sha256
integrity
  npm_registry_integrity, package_contained_sha256, independent_signature
```

A public candidate requires clean source, `publication_candidate: true`, exact `refs/tags/v0.1.0`/`v0.1.0`, exact rustc/Cargo 1.97.1, the approved runner, and Linux glibc 2.35 where applicable. Private non-candidates require null tag/ref.

The fixed integrity claims are deliberately separate:

```json
{
  "npm_registry_integrity": "external_to_launcher_runtime",
  "package_contained_sha256": "consistency_only",
  "independent_signature": "absent"
}
```

- npm tarball integrity and npm registry provenance attestations are registry acquisition evidence.
- The package-contained SHA-256 detects inconsistency or post-install mutation relative to metadata shipped in the same package.
- No independent native-binary signature, signing key, notarization, or registry-signature equivalence is claimed.

## Launcher package

The launcher tarball contains exactly:

```text
package.json                       0644
bin/career.js                      0755
README.md                          0644
LICENSE-MIT                        0644
LICENSE-APACHE                     0644
THIRD_PARTY_NOTICES.md             0644
```

The bounded package README documents only `@revazi/career`; it contains no direct internal-package installation instructions. Candidate assembly compares the launcher, README, licenses, and notices byte-for-byte with reviewed source before transfer to the publish job.

Before every execution the launcher validates exact launcher/native manifest and provenance property sets, package/version/target/minimum-libc/build identity, regular non-symlink metadata, exact modes, bounded sizes, Mach-O ARM64 or ELF64 x86-64 headers, SHA-256, and final file identity.

The verified executable is launched as:

```text
spawn(binaryPath, originalArgv, { shell: false, stdio: "inherit" })
```

argv strings remain literal; stdin/stdout/stderr, normal exit status, and SIGINT/SIGTERM/SIGHUP behavior remain native. No output framing is added.

## Ordered publication candidate

Final assembly emits exactly four flat regular files outside the checkout:

```text
10-revazi-career-darwin-arm64-0.1.0.tgz
20-revazi-career-linux-x64-gnu-0.1.0.tgz
30-revazi-career-0.1.0.tgz
publication-manifest.json
```

`career.npm_publication_candidate.v1` binds the exact source repository/SHA/ref/tag, clean candidate state, Node/npm versions, public access, required npm provenance, absent independent signature, package roles, strict order, filenames, SHA-256, and SHA-512 SRI for all three tarballs. Extra files, directories, FIFOs, symlinks, nested artifact directories, duplicate tar paths, non-regular tar members, unsafe modes, manifest extras, and byte/integrity drift fail closed.

The two native packages are always processed before the user-facing launcher. The launcher cannot publish unless both exact native versions already exist with the candidate's exact registry integrity and valid npm SLSA provenance.

## Authentication, provenance, retry, and partial state

`.github/workflows/npm-publish.yml` is manual-only and environment-gated by `npm-production`. Preparation jobs have read-only contents permission. The minimal publish job adds only `id-token: write`, checks out only the reviewed publish driver, installs no project dependency, and executes no package lifecycle script.

Publication uses exact Node 22.19.0/npm 11.6.2 and `npm publish --access public --provenance --ignore-scripts`. Trusted publishing requires npm 11.5.1 or newer and Node 22.14 or newer; normal local package tests remain compatible with reviewed npm 10.9.3.

Two modes are explicit and never fall back:

- `bootstrap=true`: uses only a temporary granular `NPM_TOKEN` supplied by the protected environment. It is for first publication and an exact interrupted-bootstrap recovery. Every package name must be absent or already contain exact `0.1.0` with the candidate's exact integrity and valid npm registry provenance. A name that exists without that exact reviewed version, or with conflicting integrity/provenance, blocks all publication.
- `bootstrap=false` (default): requires every package name already to exist so package-level GitHub trusted publishers can be configured. `NODE_AUTH_TOKEN` must be absent; a temporary auth-free npm userconfig is used and npm authenticates only through GitHub OIDC.

Bootstrap uses a temporary mode-0600 npm userconfig containing the literal `${NODE_AUTH_TOKEN}` placeholder, never the token value. OIDC mode uses a separate auth-free mode-0600 userconfig. Both are removed on exit.

Known transient publish failures receive at most three attempts. A dropped response is accepted only when the exact version appears with exact candidate integrity and valid registry provenance. Registry attestations are polled boundedly and must include:

```text
dist.attestations.provenance.predicateType = https://slsa.dev/provenance/v1
```

plus a bounded HTTPS attestations URL on `registry.npmjs.org`. Missing or malformed attestations fail publication completion. This npm registry provenance is not an independent native-binary signature.

An interrupted bootstrap is rerunnable with the same exact candidate/token: already published exact packages are verified and skipped, missing native packages continue first, and the launcher remains last. A conflicting or otherwise partial package-name state requires incident review; it is never repaired by changing bytes or silently incrementing a version.

## Public registry acceptance

The workflow is not complete after `npm publish`. A final no-secret matrix on `macos-14` ARM64 and `ubuntu-22.04` x64 waits boundedly for registry propagation, acquires only exact `@revazi/career@0.1.0` through the documented npx/package path, confirms npm selected the correct internal optional package, and runs version, capability, operation, schema, resume-analysis, and job-match parity. No source executable or PATH fallback is available.

Only after both jobs pass does the parent maintainer create the GitHub Release for annotated `v0.1.0`, with dated notes linking the exact npm package/version and acceptance result. That Release contains no custom binary assets, crate publication, signature, or notarization.

## Stable launcher errors

The existing bounded `CAREER_NPM_*` error table remains stable. In particular:

| Code | Meaning |
|---|---|
| `CAREER_NPM_UNSUPPORTED_PLATFORM` | platform/architecture is not approved |
| `CAREER_NPM_UNSUPPORTED_LIBC` | glibc 2.35+ was not locally confirmed |
| `CAREER_NPM_LAUNCHER_MANIFEST_INVALID` | launcher identity/metadata/lockstep is invalid |
| `CAREER_NPM_PLATFORM_PACKAGE_MISSING` | selected internal optional package is unavailable locally |
| `CAREER_NPM_PLATFORM_MANIFEST_INVALID` | native manifest is absent, malformed, unsafe, or contains forbidden code/dependencies |
| `CAREER_NPM_PLATFORM_PACKAGE_MISMATCH` | resolved native identity is wrong |
| `CAREER_NPM_VERSION_MISMATCH` | launcher/native/provenance versions differ |
| `CAREER_NPM_TARGET_MISMATCH` | target/minimum-libc mapping differs |
| `CAREER_NPM_PROVENANCE_MISSING` | provenance is absent |
| `CAREER_NPM_PROVENANCE_INVALID` | provenance property/build/release/trust claim is invalid |
| `CAREER_NPM_PROVENANCE_PACKAGE_MISMATCH` | provenance package identity differs |
| `CAREER_NPM_PROVENANCE_TARGET_MISMATCH` | provenance target/minimum-libc mapping differs |
| `CAREER_NPM_DIRTY_PROVENANCE` | a dirty/non-candidate build is marked publishable |
| `CAREER_NPM_BINARY_MISSING` | `career` is absent |
| `CAREER_NPM_BINARY_TYPE_INVALID` | binary is symlink/non-regular/unopenable |
| `CAREER_NPM_BINARY_MODE_MISMATCH` | mode is not exactly 0755 |
| `CAREER_NPM_BINARY_SIZE_MISMATCH` | size differs or exceeds 16 MiB |
| `CAREER_NPM_BINARY_TYPE_MISMATCH` | native target header differs |
| `CAREER_NPM_BINARY_HASH_MISMATCH` | package-contained SHA-256 differs |
| `CAREER_NPM_BINARY_CHANGED` | file identity changed during verification |
| `CAREER_NPM_LAUNCH_FAILED` | verified executable could not start |

Messages remain bounded, single-line, path/payload-free diagnostics.
