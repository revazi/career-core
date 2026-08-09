# Proposed npm CLI distribution v2 (`0.1.1`)

This contract describes the release-blocked Phase 9 matrix extension. Public `@revazi/career@0.1.0` and its v1 npm metadata remain immutable. Nothing in this document claims a new supported platform or authorizes publication.

## Consumer and compatibility boundary

The only future consumer surface remains exact `@revazi/career@0.1.1`, bin `career`, after separately authorized publication. Internal platform packages are never consumer interfaces. npm/npx package acquisition may use the network; the installed launcher and native CLI perform no network requests or telemetry.

Patch `0.1.1` changes exact Core version metadata from `0.1.0` to `0.1.1`. `career operations` retains the same schema and descriptor set with exact `core_version: "0.1.1"`. Existing v1 operations, schemas, limits, ordering, algorithms, evidence, warnings, uncertainty, and assisted authority remain stable.

## Exact ordered target catalog

The reviewed source catalog is [`../../npm/career/targets.json`](../../npm/career/targets.json), schema `career.npm_target_catalog.v1`. Its exact bytes are SHA-256 pinned in the launcher and independently checked by candidate policy code.

| Order | Platform key | Rust target | Node OS/CPU/libc | Internal package | Executable | Format/architecture | Proposed GNU floor |
|---:|---|---|---|---|---|---|---|
| 10 | `darwin-arm64` | `aarch64-apple-darwin` | `darwin` / `arm64` | `@revazi/career-darwin-arm64` | `career` | Mach-O 64 / AArch64 | n/a |
| 20 | `darwin-x64` | `x86_64-apple-darwin` | `darwin` / `x64` | `@revazi/career-darwin-x64` | `career` | Mach-O 64 / x86-64 | n/a |
| 30 | `linux-x64-gnu` | `x86_64-unknown-linux-gnu` | `linux` / `x64` / glibc | `@revazi/career-linux-x64-gnu` | `career` | ELF64 / x86-64 | `2.35` |
| 40 | `linux-arm64-gnu` | `aarch64-unknown-linux-gnu` | `linux` / `arm64` / glibc | `@revazi/career-linux-arm64-gnu` | `career` | ELF64 / AArch64 | `2.35` |
| 50 | `linux-x64-musl` | `x86_64-unknown-linux-musl` | `linux` / `x64` / musl | `@revazi/career-linux-x64-musl` | `career` | ELF64 / x86-64 | n/a |
| 60 | `linux-arm64-musl` | `aarch64-unknown-linux-musl` | `linux` / `arm64` / musl | `@revazi/career-linux-arm64-musl` | `career` | ELF64 / AArch64 | n/a |
| 70 | `win32-x64-msvc` | `x86_64-pc-windows-msvc` | `win32` / `x64` | `@revazi/career-win32-x64-msvc` | `career.exe` | PE32+ / x86-64 | n/a |
| 80 | `win32-arm64-msvc` | `aarch64-pc-windows-msvc` | `win32` / `arm64` | `@revazi/career-win32-arm64-msvc` | `career.exe` | PE32+ / AArch64 | n/a |
| 90 | launcher | n/a | Node `>=22` | `@revazi/career` | bin `career` | reviewed JavaScript | n/a |

`optionalDependencies` object order and `career_launcher.platform_packages` array order must exactly match rows 10–80. Missing, extra, duplicate, reordered, or non-lockstep entries fail verification.

## Catalog requirements

Every target records exact platform/package identity, Rust and Node mappings, libc family, executable name, binary format and architecture, native runner OS/architecture, maximum binary size, archive/runtime file invariants, proposed minimum glibc version, and provenance requirements. The binary ceiling is currently 16 MiB for every target.

Required provenance states that native execution, format inspection, dynamic-import inspection, and SHA-256 verification are mandatory. Cross-compilation and emulation are explicitly not release evidence.

## Linux selection

Linux selection requires positive bounded evidence:

- a valid `process.report` GNU libc runtime version selects the exact GNU package only when it meets the catalog floor;
- an exact musl loader/shared-object marker selects the exact musl package;
- conflicting, missing, malformed, or unknown evidence returns `CAREER_NPM_UNSUPPORTED_LIBC`.

“Not glibc” never means musl. GNU and musl packages cannot substitute for one another. Native CI must record the highest imported GLIBC symbol and all dynamic imports for GNU artifacts. Musl CI must record whether each artifact is static or its exact interpreter/import set.

## File and binary invariants

All platforms require a bounded regular non-symlink file whose size and SHA-256 exactly match provenance before direct execution with `shell: false` and inherited stdio. Resolution performs that verification once, then launch immediately reopens with no-follow where the host exposes it, rechecks identity/format/size/SHA-256, holds the verified descriptor through the synchronous spawn call, and rejects a pathname replaced between resolution and launch.

Node does not expose one portable `fexecve`/Windows handle-based process API. This consistency check therefore does not claim protection from an actively malicious same-user process that can also rewrite the installed launcher itself; npm integrity and registry provenance remain the external acquisition boundary. The revalidation closes ordinary package replacement between resolution and launch and minimizes the remaining platform process-creation interval without overstating package-contained SHA-256 as a signature.

Unix targets additionally require exact mode `0755`. Windows does not pretend POSIX execute bits are authoritative: it requires exact filename `career.exe`, the reviewed Windows regular-file invariant, and PE32+ machine `0x8664` for x64 or `0xaa64` for ARM64. Candidate tar members use exact reviewed archive modes (`0755` Unix, `0644` Windows).

Internal `career.npm_launcher.v2`, `career.npm_native_package.v2`, and `career.npm_native_provenance.v2` make these differences explicit. Package-contained SHA-256 remains consistency-only, npm registry integrity/provenance remains external acquisition evidence, and independent signatures remain absent.

## Evidence and release blockers

Synthetic headers and non-host tarballs test fail-closed policy only. They do not execute and cannot support a platform claim. The preparation-only native evidence workflow uses exact `macos-14` ARM64, `macos-15-intel` x64, `ubuntu-22.04` x64, and `ubuntu-24.04-arm` with native Ubuntu 22.04 ARM64 userland for the Darwin/GNU sub-gate. The musl sub-gate uses immutable multi-architecture `node:22.19.0-alpine3.22@sha256:d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9` on architecture-matched native `ubuntu-22.04` x64 and `ubuntu-24.04-arm` runners. Outer Docker, inner kernel, Rust host, and musl loader architecture must agree without `--platform` or emulation; exact musl 1.2.5 and the observed target-specific static ELF form are mandatory: x64 is `DYN` static PIE with `NOW PIE` flags and a relocation-only dynamic section, while AArch64 is `EXEC` with no dynamic section; neither may have an interpreter, shared-library import, or imported GLIBC symbol. The Windows sub-gate uses native `windows-2025` x64 and `windows-11-arm` ARM64 runners. Runner/process/Python/Node/Rust architecture must agree; bounded PE parsing requires PE32+, exact `0x8664` or `0xaa64` machine, executable-image characteristics, section/RVA-safe import traversal, and reviewed system-DLL basenames. Windows records exact `career.exe`, archive mode `0644`, and regular non-symlink policy but no Unix runtime mode. Every job executes offline extracted packages and emits bounded `career.npm_native_inspection.v1` evidence for source/catalog binding, host architecture, version, format, linkage/imports, interpreter, libc, size, target-specific mode, and SHA-256. Evidence remains in logs and is neither uploaded nor published.

A target becomes release-eligible only when its final packaged binary executes on the exact native OS and architecture in CI without emulation and passes version, discovery, managed-adapter, representative operation, package, format, linkage, size, file-invariant, and SHA-256 checks. Intermediate evidence proves runner/build policy but all targets must be reproven from final release source.

The protected `v0.1.1` publication workflow remains deferred until all eight native targets have such evidence. If any required native runner is unavailable, that target and the release remain blocked.
