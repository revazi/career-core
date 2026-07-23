# Current phase

## Active phase

**Phase 2 — Deterministic resume normalization and provider-neutral assisted boundary**

## Status

Implementation and clean-clone verification are complete locally; GitHub Actions and review are pending. Phase 3 has not started.

## Implemented

- `career.resume_normalization.v1` and `resume_normalization_v1` deterministic contracts.
- Bounded contact, summary, experience, education, skills, projects, and certifications structures.
- Source spans and explicit transformations for every non-empty normalized value.
- Shared explicit whole-line aliases for all six sections.
- Conservative experience blocks plus experience, skills, and education fallback heuristics.
- Five-signal integer parser confidence with unknown/low/medium/high labels and explicit gates.
- Detected, likely-missing, and not-detected field statuses without converting uncertainty into absence.
- Stable section metadata, fallback identifiers, truncation metadata, and warnings.
- `career resume normalize --input <path|-> --format json|text`.
- `career.resume_enrichment_proposal.v1`, input, result, and policy contracts.
- Low-confidence-and-empty-field enrichment eligibility in canonical target order.
- Strict proposal shape, character, list, entry, target, and source-grounding validation.
- Conservative empty-target-only merge with deterministic/external/not-available provenance.
- A complete immutable deterministic baseline plus a separately labeled assisted document.
- Preserved deterministic confidence and explicit warning that assisted fields are non-authoritative.
- `career resume enrich --input <path|-> --format json|text` with no provider or network behavior.
- Updated agent guidance for opt-in host orchestration, failure isolation, and baseline authority.
- Synthetic clean, messy, source-grounded proposal, Unicode, sparse, prompt-like, limit, and invalid-output coverage.
- Public schemas, contract documentation, CLI golden fixtures, and CI smoke checks.
- Capability discovery marks `resume.normalize` and `resume.enrich` available.

## Architecture decision

The root library still has no API-key, provider, prompt, model-call, or network behavior. An opted-in agent or application owns those concerns. The core emits eligible targets, validates an explicit proposal as untrusted data, and creates a separate assisted view. Future authoritative scoring consumes the deterministic baseline only.

## Reference scope

Behavior and bounds were intentionally adapted from:

- `../resume-ai/accounts/services/resume_normalization.py`
- `../resume-ai/accounts/services/resume_normalization_fallback_validation.py`
- `../resume-ai/accounts/services/normalization_fallback_merge.py`
- normalization tests and synthetic fixtures under `../resume-ai/accounts/`
- reference versions `resume_normalization_v7`, `resume_normalization_llm_fallback_v1`, and `resume_normalization_fallback_merge_v1`

Provider execution, prompts, Django persistence, and score coupling were not ported. The Rust baseline/assisted split, date-as-phone rejection, and education-date grouping are intentional differences. This is selected fixture/rule adaptation, not full Django policy parity.

## Verification completed locally

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features --locked` — 47 tests passed
- `cargo build --workspace --all-features --locked`
- rustdoc with warnings denied
- capability JSON and text smoke checks
- Phase 1 evaluation goldens remain unchanged
- Phase 2 deterministic and assisted JSON/text CLI checks pass
- deterministic normalization and enrichment goldens match byte-for-byte
- enrichment `baseline` exactly equals the independent deterministic messy-resume golden
- all schemas pass Draft 2020-12 metaschema validation
- input, normalization, proposal, enrichment-input, enrichment-result, and representative error documents validate with `check-jsonschema`
- all six Rust section alias sets exactly match `resume_normalization_v7`
- clean-clone formatting, Clippy, 47-test, locked-build, and all three CLI golden checks
- regex dependency family declares Rust 1.65 MSRV and permissive MIT/Apache/Unlicense-compatible licensing
- all relative Markdown links resolve
- README JSON and CI YAML parse
- Agent Skill metadata and size checks pass
- credential-pattern scan reports no findings
- Fallow changed-code and security checks report no findings; Fallow does not currently analyze Rust source for health metrics
- `git diff --check`

GitHub Actions, including the Rust 1.85 job, remains pending until the branch is pushed.

## Explicitly unavailable

- complete `deterministic_v2` resume evaluation and ATS-readiness checks
- job-description normalization and resume-to-job matching
- provider clients, model calls, prompt construction, or API-key handling
- automatic rewriting or mutation of source resumes
- PDF/DOCX ingestion or OCR
- Swift bindings and SwiftUI host orchestration
- MCP or provider-specific adapters

## Next phase after approval

Phase 3 will port explainable deterministic resume evaluation and ATS-readiness checks against the deterministic normalization contract. Do not begin Phase 3 until Phase 2 is green, reviewed, merged, and explicitly approved.
