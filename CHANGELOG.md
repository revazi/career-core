# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project intends to follow [Semantic Versioning](https://semver.org/) once public contracts begin shipping.

## [Unreleased]

### Fixed

- Allow one combined bounded npm registry integrity/provenance readiness window of up to ten minutes per sequential package, preventing successful publication from failing after the former 50-second visibility window.

## [0.1.0] - 2026-08-09

### Added

- Initial Rust library and workspace foundation.
- Versioned, network-free capability discovery.
- JSON-first `career` CLI for coding-agent discovery.
- Detailed project and agent workflow documentation.
- Public maintainer, contact, support, security, ownership, and release-governance documentation.
- Dual MIT/Apache-2.0 licensing.
- Versioned `career.resume_input.v1`, `career.resume_evaluation.v1`, and `career.error.v1` contracts.
- Bounded deterministic resume section-coverage evaluation with exact header aliases, evidence, and uncertainty warnings.
- `career resume evaluate` with file/stdin input, JSON/text output, and documented exit statuses.
- Synthetic reviewed golden fixtures and public JSON schemas for the Phase 1 vertical slice.
- Versioned deterministic resume normalization with source spans, confidence signals, field statuses, warnings, conservative fallbacks, and bounded structured facts.
- Provider-neutral external proposal contracts with strict source-grounding validation, empty-target-only merging, preserved deterministic baselines, and field provenance.
- `career resume normalize` and `career resume enrich` with file/stdin input and JSON/text output.
- Phase 2 schemas and synthetic clean, messy, assisted, Unicode, sparse, and adversarial regression coverage.
- Versioned `career.resume_analysis.v1` with 18 deterministic checks, six weighted categories, bounded evidence, confidence-aware score floors, provisional findings, and check-derived actions.
- `career resume analyze` with preserved Phase 1 behavior, JSON/text output, selected `deterministic_v2` scoring parity, and explicit proprietary-ATS/layout limitations.
- Versioned `career.job_input.v1` and `career.job_normalization.v1` contracts with source-grounded required/preferred fields, responsibilities, explicit requirement signals, confidence, metadata, statuses, and warnings.
- `career job normalize` with file/stdin input, JSON/text output, bounded synthetic fixtures, and selected deterministic `job_description_normalization_v6` fixture parity.
- Versioned `career.job_match_input.v1` and `career.job_match.v1` contracts with six weighted categories, source/derived evidence, confidence bounds, strengths, gaps, and deterministic recommendation gates.
- `career job match` with baseline-only normalization, normalized exact and reviewed same-technology aliases, close-non-equivalent/vague/weak fixtures, and selected `job_match_deterministic_v2` scoring parity.
- Additive `json-pretty` and one-line `json-compact` output modes while preserving canonical `json` output.
- Offline `career schema list` and `career schema export` commands backed by reviewed schemas embedded in the CLI binary.
- `career.schema_catalog.v1`, source-install guidance, shell-safe coding-agent examples, Linux/macOS compatibility CI, and a checksum-based release-preparation checklist.
- Pinned UniFFI `0.30.0` `career-swift` adapter with JSON-in/JSON-out functions for every stable deterministic operation and typed Swift errors.
- Local `CareerCoreSwift` package, Apple Silicon macOS/iOS/iOS-simulator XCFramework build, exact Swift/Rust fixture parity, smoke executable, and per-file artifact checksums.
- Versioned evidence-linked assisted resume-variant review/materialization contracts with 50-change bounds, canonical selection identifiers, exact baseline preservation, CLI/schema/Swift facade coverage, and mandatory non-authoritative factuality warnings.
- Review-only external resume-analysis suggestion contracts with fresh deterministic baseline reruns, action/check binding, exact source targets/evidence, core-assigned identifiers, confirmed/provisional status, payload-free discard diagnostics, and CLI/schema/Swift parity.
- Separate review-only external resume-analysis replacement contracts with fresh deterministic baseline reruns, action/check binding, exact source before/proposed-after values, no-change rejection, core-assigned identifiers, payload-free discards, and CLI/schema/Swift parity for non-authoritative diff display.
- Temporary-root installed-CLI acceptance covering capability/schema discovery and representative resume, job, and assisted-review commands outside the checkout.
- Separate maintainer-dispatched native runtime artifact preparation for reviewed `pi-career` imports, with fail-closed target checks, bounded provenance/digests, synthetic execution, and short retention; this is not a public Core release channel.
- Versioned `career.operation_catalog.v1` discovery with stable capability/operation mapping, bootstrap commands, exact CLI paths, input transports, schemas, and byte ceilings.
- Deterministic offline `career schema bundle` output with recursive embedded-reference rewriting and no unresolved or remote `$ref`.
- A 33,554,432-byte (32 MiB) complete successful machine-JSON ceiling enforced after serialization and before stdout writes, with exact-bound and overflow coverage.
- Versioned managed-adapter compatibility metadata for maintainer runtime artifacts, including catalog/schema digests, complete mappings, representative bundle digests, declared output bounds, deterministic result digests, and adversarial tamper tests.
- Independent installed-binary validation for all schema bundles plus Draft 2020-12 metaschema and representative-instance checks.
- Private source template for exact user-facing Node 22+ `@revazi/career@0.1.0`, with lockstep internal native optional packages for Apple Silicon macOS and x86-64 GNU/Linux glibc 2.35+.
- Fail-closed package-local launcher verification for glibc/target, manifest/provenance/version, regular non-symlink binary mode/size/type/SHA-256, and argv/stdin/stdout/stderr/exit/signal transparency.
- Versioned npm native provenance with explicit package-contained consistency-only hashing and absent independent-signature claims.
- External current-host npm package preparation, offline exact-allowlist tarball/extracted-install/all-operation parity tests, and a manual two-platform preparation-only workflow with no publication or artifact upload.
- Exact public-package-only `pi-career` package/bin/npx/provenance/error/transition handoff while retaining the old artifact path as transitional.
- Clean annotated `v0.1.0`/`origin/main` publication source gate, exact rustc/Cargo 1.97.1 native candidate builds, strict ordered three-tarball assembly, public launcher README/metadata, and `career.npm_publication_candidate.v1` integrity manifest.
- Registry-free candidate/adversarial/fake-npm tests covering manifest/file/mode/source-byte allowlists, dirty/missing/lightweight/moved tags, glibc 2.35, bootstrap/OIDC auth isolation, partial/idempotent recovery, conflicts, transients, launcher-last ordering, and npm registry attestations.
- Protected manual `npm-production` workflow using exact Node 22.19.0/npm 11.6.2, temporary-token bootstrap or default OIDC trusted publishing, npm provenance/public access, corrected immutable GitHub Action pins, and no-secret public registry acceptance on macOS ARM64 and Ubuntu 22.04 x64; the parent maintainer creates a dated no-custom-assets `v0.1.0` GitHub Release only after acceptance.

### Changed

- Externalized the native Pi package and bundled Agent Skill to the standalone [`pi-career`](https://github.com/revazi/pi-career) repository; Career Core no longer ships Pi package/runtime/test artifacts or a duplicate project skill.
- Labeled the existing `pi-career` native archive workflow transitional; removal remains gated on exact public `@revazi/career@0.1.0` acceptance and separate consumer adoption.
- Established `@revazi/career` as the only npm consumer surface; internal native packages are never direct user or `pi-career` dependencies.
