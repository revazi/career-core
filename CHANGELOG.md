# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project intends to follow [Semantic Versioning](https://semver.org/) once public contracts begin shipping.

## [Unreleased]

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

### Changed

- Externalized the native Pi package and bundled Agent Skill to the standalone [`pi-career`](https://github.com/revazi/pi-career) repository; Career Core no longer ships Pi package/runtime/test artifacts or a duplicate project skill.
