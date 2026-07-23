# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project intends to follow [Semantic Versioning](https://semver.org/) once public contracts begin shipping.

## [Unreleased]

### Added

- Initial Rust library and workspace foundation.
- Versioned, network-free capability discovery.
- JSON-first `career` CLI for coding-agent discovery.
- Detailed project and agent workflow documentation.
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
