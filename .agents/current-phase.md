# Current phase

## Active sub-phase

**Phase 4A — Deterministic job-description normalization**

## Status

Implementation and full local verification are complete on `feature/job-normalization-v1`. Clean-clone verification, GitHub Actions, and review are pending. Phase 4B matching has not started.

## Approved scope

- bounded `career.job_input.v1`
- source-grounded `career.job_normalization.v1`
- title and company candidates
- required/preferred skills and qualifications
- responsibilities and explicit seniority, experience, education, and certification signals
- exact whole-line section aliases and conservative inline classification
- noise exclusion, stable de-duplication, and bounded matched/unmatched metadata
- six-signal job parse confidence and provisional warnings
- canonical field statuses where `not_detected` never confirms absence
- `career job normalize --input <path|-> --format json|text`
- public schemas, synthetic fixtures, reference projection, docs, agent guidance, and CI goldens

## Explicitly out of scope

- `career job match` or any resume-to-job score
- skill equivalence or recommendation logic
- URL fetching or company research
- provider fallback, prompts, API keys, or external-proposal job enrichment
- persistence, application tracking, or cover letters
- Swift bindings, SwiftUI, MCP, or provider-specific adapters

## Reference scope

The selected deterministic reference is `job_description_normalization_v6` from:

- `../resume-ai/accounts/services/job_description_schema.py`
- `../resume-ai/accounts/services/job_description_normalization.py`
- job normalization classes in `../resume-ai/accounts/test_services.py`
- `../resume-ai/accounts/test_fixtures/normalization/prose_heavy_job_description.txt`
- deterministic fixture assertions in `../resume-ai/accounts/test_normalization_fixtures.py`

Provider fallback execution, fallback merge, Django models/API fields, URL fetching, and matching are excluded.

## Implemented so far

- Added typed job input, normalized document, source span, confidence, status, metadata, warning, and error aliases.
- Ported all required, preferred, responsibility, and noise section aliases.
- Ported deterministic skill, qualification, responsibility, seniority, experience, education, and certification classification.
- Ported matched/unmatched metadata and all six confidence signals with integer ratio evidence.
- Added list/input limits, stable output ordering, source provenance, truncation metadata, and conservative warnings.
- Added the `career job normalize` JSON/text CLI path.
- Added complete and prose-heavy synthetic goldens plus a compact Django reference projection.
- Added public schemas, contract tests, selected parity tests, and user/agent documentation.

## Verification completed locally

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features --locked` — 79 tests passed
- `cargo build --workspace --all-features --locked`
- rustdoc with warnings denied
- capability JSON/text checks expose `job.normalize` as available and retain `job.match` as planned
- all prior resume JSON goldens remain byte-equivalent
- complete and prose-heavy job JSON goldens match byte-for-byte; job text output preserves provisional warnings
- all schemas pass Draft 2020-12 metaschema validation
- job inputs/outputs and representative resume outputs validate with `check-jsonschema`
- all four section alias sets exactly match `job_description_normalization_v6`
- the compact reference projection regenerates byte-for-byte from the Django normalizer
- every selected normalized field, matched/unmatched metadata record, confidence label/score, and six signal scores matches the reference
- high/medium/low confidence, sections, inline requirements, noise exclusion, Unicode, physical spans, prompt-like text, unmatched diagnostics, limits, and typed errors are covered
- dependency tree contains no network, provider, TLS, async-runtime, or telemetry stack
- relative Markdown links, README capability JSON, CI YAML, and Agent Skill checks pass
- credential-pattern scan reports no findings
- Fallow changed-code and security checks report no findings; Fallow does not currently analyze Rust source for health metrics
- `git diff --check`

Clean-clone verification and GitHub Actions, including Rust 1.85 compatibility, remain pending.

## Next sub-phase after approval

Phase 4B will add conservative deterministic matching only after Phase 4A is fully verified, reviewed, merged, and explicitly approved.
