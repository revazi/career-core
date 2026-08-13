# npm CLI distribution v2

This contract describes the active Career Core npm source and future release boundary. The checked-in `@revazi/career` launcher supports macOS and Linux only. Native Windows support is retired.

Public `@revazi/career@0.1.0` and `@revazi/career@0.1.1` bytes are immutable. Protected run `31346152236` historically published `0.1.1` with eight native packages, including two Windows packages, and passed all-eight public acceptance. Those Windows artifacts are historical release evidence, not an active or future support promise.

## Consumer and host boundary

The only consumer package is exact `@revazi/career`, bin `career`; internal native packages are never consumer interfaces. npm/npx acquisition may use the network. The installed launcher and native CLI perform no network requests or telemetry.

Active launcher metadata declares npm `os` values `darwin` and `linux`. The root `career-core` crate rejects `target_os = "windows"` at compile time, so the library, CLI, and adapters cannot be built natively for Windows. Windows users must build and run Career Core and pi-career through WSL. Inside WSL, Rust and Node target Linux, Node reports `process.platform === "linux"`, and the normal Linux architecture/libc policy applies. Native `win32` execution is unsupported and no `career.exe` route exists in active source.

This support-boundary change does not alter Core algorithms, v1 operations or schemas, limits, ordering, evidence, warnings, uncertainty, assisted authority, CLI output bytes, or Swift behavior. macOS, Linux, and existing Apple iOS/Swift targets remain unaffected.

## Exact ordered active target catalog

The reviewed source catalog is [`../../npm/career/targets.json`](../../npm/career/targets.json), schema `career.npm_target_catalog.v1`. Its exact bytes are SHA-256 pinned in the launcher and independently checked by candidate policy code.

| Order | Platform key | Rust target | Node OS/CPU/libc | Internal package | Format/architecture | GNU floor |
|---:|---|---|---|---|---|---|
| 10 | `darwin-arm64` | `aarch64-apple-darwin` | `darwin` / `arm64` | `@revazi/career-darwin-arm64` | Mach-O 64 / AArch64 | n/a |
| 20 | `darwin-x64` | `x86_64-apple-darwin` | `darwin` / `x64` | `@revazi/career-darwin-x64` | Mach-O 64 / x86-64 | n/a |
| 30 | `linux-x64-gnu` | `x86_64-unknown-linux-gnu` | `linux` / `x64` / glibc | `@revazi/career-linux-x64-gnu` | ELF64 / x86-64 | `2.35` |
| 40 | `linux-arm64-gnu` | `aarch64-unknown-linux-gnu` | `linux` / `arm64` / glibc | `@revazi/career-linux-arm64-gnu` | ELF64 / AArch64 | `2.35` |
| 50 | `linux-x64-musl` | `x86_64-unknown-linux-musl` | `linux` / `x64` / musl | `@revazi/career-linux-x64-musl` | ELF64 / x86-64 | n/a |
| 60 | `linux-arm64-musl` | `aarch64-unknown-linux-musl` | `linux` / `arm64` / musl | `@revazi/career-linux-arm64-musl` | ELF64 / AArch64 | n/a |
| 70 | launcher | n/a | Node `>=22`; npm OS `darwin`, `linux` | `@revazi/career` | reviewed JavaScript | n/a |

`optionalDependencies` object order and `career_launcher.platform_packages` array order must exactly match rows 10–60. Missing, extra, duplicate, reordered, or non-lockstep entries fail verification.

## Selection and binary invariants

Linux selection requires positive bounded evidence. A valid `process.report` GNU libc version selects the exact GNU package only when it meets the catalog floor. An architecture-matched musl marker selects the exact musl package. Conflicting, missing, malformed, or unknown evidence returns `CAREER_NPM_UNSUPPORTED_LIBC`; “not glibc” never means musl.

Every active target uses executable `career` and requires a bounded regular non-symlink file with exact mode `0755`. Size, Mach-O or ELF format/architecture, and SHA-256 must match versioned provenance before direct execution with `shell: false` and inherited stdio. Resolution revalidates the pathname immediately before spawn. Package-contained SHA-256 is consistency evidence, npm registry integrity/SLSA provenance is external acquisition evidence, and independent native signatures remain absent.

Unknown platform, architecture, libc, catalog, package identity, provenance, file invariant, format, size, or digest fails closed with bounded diagnostics. The launcher has no runtime download, network request, install hook, PATH fallback, telemetry, provider behavior, or verification bypass.

## Evidence and release blockers

Synthetic headers and non-host tarballs test policy only; they do not establish support. Final release packages must execute on exact native hosts. Darwin/GNU evidence uses native macOS ARM64/x64 and Ubuntu 22.04 GNU x64/ARM64 environments. Musl evidence uses the immutable multi-architecture Node 22.19.0 Alpine 3.22 image on architecture-matched x64/ARM64 Linux runners and requires exact musl 1.2.5 plus the reviewed static ELF form.

Future stable versions use OIDC-only `.github/workflows/npm-release.yml`. The workflow assembles six native packages in catalog order and the launcher seventh and last, verifies exact bytes/integrity/provenance, and requires six no-secret public acceptance jobs. If any runner, package, integrity record, provenance record, or acceptance job is unavailable, that version remains blocked.

Version-specific `.github/workflows/npm-publish.yml` and `.github/workflows/npm-publish-v0.1.1.yml` remain immutable historical evidence. They must not be edited to imply that the old release omitted Windows, and they must not be used as templates for future releases.
