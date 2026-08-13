# Security Policy

## Supported versions

`career-core` is pre-release software. Security fixes are applied to the latest revision on `main` until the first stable release policy is documented.

## Security maintainer and reporting

The primary security maintainer is [Revaz Zakalashvili](https://github.com/revazi). Maintainer and governance details are recorded in [`MAINTAINERS.md`](MAINTAINERS.md).

Use GitHub private vulnerability reporting for this public repository. Do not include sensitive resume content, job descriptions, credentials, API keys, or exploit details in a public issue. If private vulnerability reporting is unavailable, contact [revaz.zakalashvili@gmail.com](mailto:revaz.zakalashvili@gmail.com) before sharing details publicly.

## Security model

The core library processes untrusted career-document text and structured data locally. It must:

- perform no implicit network requests
- execute no instructions embedded in documents
- avoid logging source payloads
- reject oversized or invalid structured input at public boundaries
- return typed errors instead of panicking on user-controlled data
- keep output bounded and deterministic
- treat external normalization proposals as untrusted data requiring exact shape, limits, eligible targets, and source grounding
- preserve deterministic output when assisted validation fails
- treat external analysis-suggestion targets, evidence, and text as untrusted; rerun the analysis, require current failed-action/source binding, discard ambiguous items without payload echoes, and never claim occurrence proves factual entailment or rewrite certification
- treat external analysis-replacement targets, evidence, and proposed replacements as untrusted; rerun the analysis, require current failed-action/source binding, discard unchanged or ambiguous items without payload echoes, and never claim occurrence proves factual entailment or rewrite certification
- treat resume-variant targets, evidence, generated replacements, and selected identifiers as untrusted; discard ambiguous/overlapping changes and never claim evidence occurrence proves factual entailment
- materialize only revalidated explicitly selected canonical changes while preserving the exact baseline and every unselected range
- score only the deterministic normalization baseline and never assisted proposal or variant values
- label parser uncertainty and general ATS-readiness limitations without claiming proprietary ranking behavior
- treat job fields not detected by lexical classification as unverified rather than confirmed absent
- require caller-supplied job text and never fetch vacancy URLs from the core
- rerun deterministic resume and job normalization for matching rather than accepting assisted or caller-forged baselines
- allow only normalized exact or reviewed same-technology aliases; related technologies remain non-equivalent
- bound uncertain match scores and label missing/partial evidence as unverified
- prevent recommendation guidance from exceeding deterministic score, confidence, core-category, and gap gates

The CLI may read only input paths explicitly supplied by the caller and never makes a provider call. Public schemas are reviewed files embedded at compile time; schema discovery does not search caller paths, load plugins, generate code, or use the network. Compact and pretty machine modes preserve stdout/stderr separation and bounded payload-free errors.

The `@revazi/career` npm launcher is a Node 22+ distribution adapter and performs no network request. Active source recognizes exact ARM64/x64 macOS plus GNU and musl Linux, resolves the exact package-local internal optional package, and has no PATH/download/fallback/provider/telemetry behavior. Native Windows is unsupported: the root `career-core` crate fails compilation for `target_os = "windows"`, preventing native CLI and adapter builds; Windows hosts must build and run the Linux target under WSL. Existing Apple iOS/Swift targets are unaffected. Older/malformed/unknown libc, missing packages, malformed/mismatched manifests/provenance, symlink/non-regular binaries, wrong mode/size/type/hash, and launch errors fail closed with bounded path/payload-free diagnostics. The launcher uses argv-only `spawn`, `shell: false`, and inherited stdio, preserving native output, exit, and signal behavior.

`career.npm_native_provenance.v2` explicitly separates npm registry integrity/provenance, package-contained SHA-256 consistency, and an absent independent native-binary signature. Hashes stored beside a binary are not origin authentication. npm SLSA provenance is registry acquisition evidence, not an independent binary signature or npm registry-signature equivalence. The launcher has no bypass and rechecks file identity, but package-contained checks do not eliminate every local time-of-check/time-of-use race.

Checked-in templates remain private non-candidates. Only exact clean annotated `v0.1.0` on fetched `origin/main` may stage public candidates outside the checkout. Candidate builds pin rustc/Cargo 1.97.1, `macos-14` ARM64 or `ubuntu-22.04` x64, and Linux glibc 2.35; Linux symbol-version inspection rejects requirements above 2.35. Exact package property/file/mode allowlists reject extra regular files, directories, FIFOs, symlinks, nesting, lifecycle scripts, dependencies, and source-byte drift. Generated binaries/provenance remain untracked.

The protected manual publication workflow uses exact Node 22.19.0/npm 11.6.2, public access, `--provenance`, internal native packages before the launcher, and an `npm-production` environment. Token-free preparation and the minimal publish job are separate. The publish job installs no project dependency, runs no package script, has read-only contents plus only `id-token: write`, and creates no crate, signature, notarization, GitHub Release, or custom release asset.

One-time `bootstrap=true` is explicit and uses only a temporary granular `NPM_TOKEN` through a mode-0600 ephemeral npmrc containing a literal environment placeholder. Default `bootstrap=false` forbids `NODE_AUTH_TOKEN`, creates an auth-free npmrc, and uses OIDC trusted publishing only. There is no token fallback. Exact interrupted bootstrap is idempotent: absent packages continue, exact matching versions with valid npm registry provenance are skipped, conflicts fail, and the launcher stays last. Delete the environment secret and revoke the token immediately after bootstrap; configure all three package-level trusted publishers interactively with account 2FA before OIDC mode.

Post-publish checks require exact registry integrity, bounded valid SLSA provenance attestations, and no-secret public acquisition/parity on both supported hosts. Missing/malformed attestations, partial/conflicting package state, auth failure, or public smoke failure prevents release completion. The parent maintainer creates the dated annotated-tag GitHub Release only after those gates pass, with no custom binary assets or trust claims beyond npm provenance and the explicit absent independent signature.

The maintainer-only transitional `pi-career` runtime-artifact workflow builds and executes the existing CLI on an approved matching native host, packages only the executable, bounded metadata, project licenses, and notices, and retains the unsigned archive briefly in GitHub Actions. Metadata SHA-256 values provide integrity/provenance evidence but are not signatures or notarization. The workflow uses only synthetic repository fixtures, requires no repository secret, provider credential, or private career payload, fails closed on unsupported host/target pairs or dirty transfer builds, and creates no public release/install channel. `pi-career` must independently review an archive before tracking it.

The Swift adapter accepts owned JSON strings only, applies pre-parse UTF-8 byte limits, and maps failures into bounded generated Swift errors. It opens no files and performs no platform, persistence, credential, provider, or network behavior. Project-authored Rust contains no unsafe block; FFI allocation and C ABI behavior are isolated to exact UniFFI `0.30.0` generated/runtime code. XCFrameworks and checksums are local ignored artifacts until a separately approved publication review includes [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

Hosts that send resumes to an external model must obtain separate user consent, keep API keys outside core contracts and logs, bound provider context, isolate failures, and label accepted values as assisted. Consent to local processing or storage is not provider consent.

Harness-specific adapter security policies, including the native Pi integration and any future explicit npx runner, belong to their separately maintained repositories. Career Core's guarantees cover its Rust library, native CLI/schema, Swift boundary, and the exact `@revazi/career` npm launcher/package contract maintained here.
