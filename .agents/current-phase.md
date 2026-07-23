# Current phase

## Most recently completed phase

**Phase 1 — Versioned contracts and first resume vertical slice**

## Status

Complete. Phase 2 has not started.

The private repository is available at `https://github.com/revazi/career-core`.

## Implemented

- Bounded `career.resume_input.v1` plain-text input with optional caller document identifier.
- Typed `career.resume_evaluation.v1` checks, section detections, evidence, warnings, and integer score.
- Typed `career.error.v1` core validation failures and bounded CLI failures.
- Documented limits for source characters, lines, line length, metadata, evidence, and CLI JSON bytes.
- Exact normalized aliases for Summary, Experience, Education, and Skills adapted from `resume_normalization_v7`.
- Projects and Certifications aliases retained only as conservative content boundaries.
- Explicit `detected_with_content`, `detected_without_content`, and `not_detected` statuses.
- `resume_section_coverage_v1` scoring: four equally weighted 0/100 checks and an integer mean.
- Mandatory warning that the score is section coverage, not complete quality or ATS compatibility.
- Stable evidence IDs, one-based line numbers, bounded header excerpts, and canonical check ordering.
- `career resume evaluate --input <path|-> --format json|text` with stdin support.
- Distinct CLI exit statuses for usage, I/O/byte limits, JSON, core validation, and output failures.
- Strict unknown-field rejection and no source-payload echo in errors.
- Synthetic complete and adversarial prompt-like golden fixtures with Django provenance.
- Public input, output, and error JSON schemas plus a detailed contract document.
- Updated capability discovery and Agent Skills guidance marking `resume.evaluate` available.
- CI golden-output/schema smoke checks and Rust 1.85 compatibility check.

## Reference scope

Behavior was intentionally adapted from:

- `../resume-ai/accounts/services/resume_normalization.py`
- `../resume-ai/accounts/test_services.py` (`ResumeSectionAliasTests`)
- reference version `resume_normalization_v7`

This is a bounded rule port. Full resume normalization and `deterministic_v2` scoring parity have not been evaluated or claimed.

## Verification completed locally

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features --locked` — 25 tests passed
- `cargo build --workspace --all-features --locked`
- capability JSON and text smoke checks
- complete resume-evaluation JSON golden comparison
- sparse resume-evaluation text smoke check
- all public schema documents parse as JSON
- input, golden output, and representative error documents validate with `check-jsonschema`
- clean-clone formatting, Clippy, 25-test, locked-build, and golden CLI verification
- Rust core/boundary alias sets exactly match the selected `resume_normalization_v7` reference sets
- all relative Markdown links resolve
- `git diff --check`
- GitHub Actions PR run `29997860390` — passed, including Rust 1.85 compatibility and golden CLI checks

## Explicitly unavailable

- full resume normalization and evaluation parity
- contact, experience-entry, education-entry, or skills-item extraction
- job-description normalization
- resume-to-job matching
- PDF/DOCX ingestion
- LLM or network behavior
- Swift bindings
- MCP or provider-specific agent adapters

## Next phase after approval

Phase 2 will implement bounded deterministic resume normalization with structured facts, confidence, provenance, and golden fixtures. Do not begin Phase 2 until Phase 1 is marked complete and the maintainer explicitly requests it.
