# Current phase

## Most recently completed sub-phase

**Phase 4B — Conservative deterministic matching**

## Status

Complete. PR review and merge are pending. Phase 5 distribution work has not started.

## Approved scope

- bounded `career.job_match_input.v1` containing original resume and job inputs
- baseline-only `career.job_match.v1`
- six deterministic categories with fixed 30/25/15/10/10/10 weights
- integer round-half-to-even aggregation
- normalized exact and reviewed same-technology skill equivalence only
- confidence-aware raw/published scores and missing-item statuses
- bounded source/derived evidence, strengths, gaps, and metrics
- deterministic recommendation thresholds and blocker gates
- `career job match --input <path|-> --format json|text`
- public schemas, synthetic risky fixtures, reference projection, docs, agent guidance, and CI goldens

## Explicitly out of scope

- vacancy URL fetching or company research
- embeddings, fuzzy similarity, transferable-skill inference, or LLM equivalence
- provider calls, prompts, API keys, interpretation, merge, fallback, caching, or observability
- accepting assisted or caller-supplied normalized documents as scoring input
- persistence, application tracking, cover letters, or resume mutation
- Swift bindings, SwiftUI, MCP, release binaries, or package-manager distribution

## Reference scope

Reference policies verified against `../resume-ai/docs/current-phase.md`:

- `job_match_deterministic_v2`
- `conservative_skill_equivalence_v1`
- `job_match_recommendation_safety_v1`

Primary read-only sources:

- `accounts/services/job_match_scoring.py`
- `accounts/services/job_match_skill_equivalence.py`
- `accounts/services/job_match_recommendation.py`
- `accounts/services/job_match_prompt_context.py` for bounds only
- `accounts/services/job_match_response.py` for evidence-shape ideas only
- `accounts/test_job_match_skill_equivalence.py`
- `accounts/test_job_match_recommendation.py`
- `accounts/test_job_match_fixtures.py`
- `accounts/test_fixtures/job_match/`
- matching-focused tests in `accounts/test_services.py`

LLM interpretation, validation, provider fallback, merge, prompts, cache, observability, Django models/APIs, credits, and persistence are excluded.

## Implemented so far

- Added typed match input, category, item, metric, confidence, finding, recommendation, warning, and result contracts.
- Matching independently reruns `resume_normalization_v1` and `job_normalization_v1`; assisted documents cannot enter scores.
- Ported all six reference category rules and fixed weights with integer half-even rounding.
- Added exact normalized skill matching plus all reviewed conservative alias groups and explicit version handling.
- Added complete-word domain signals to avoid Django substring false positives such as `ui` inside `building` or `requirements`.
- Added canonical skill aliases to domain and keyword membership without fuzzy expansion.
- Added uncertainty bounds of 50–75 for low/unknown resume or job confidence and normalization truncation.
- Added stable confirmed, partial, likely-missing, and unverified item statuses.
- Added bounded top strengths/gaps and suppression of broad inferred gaps for uncertain jobs.
- Added deterministic recommendation thresholds, a conservative 50-point apply-after core floor, and required-skill/core-evidence/unassessed-qualification blockers.
- Added `career job match` JSON/text CLI paths, a bounded 1,048,576-byte composite envelope, and marked `job.match` available.
- Added strict input/output schemas, complete/vague full goldens, and compact alias/adjacent/weak regression projections.
- Added an executed Django reference projection for the complete fixture and explicit intentional differences.
- Added focused core, CLI, schema, parity, determinism, safety, limits, and privacy tests.

## Verification completed locally

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features --locked` — 103 tests passed
- `cargo build --workspace --all-features --locked`
- rustdoc with warnings denied
- capability JSON/text checks expose `job.match` as available
- all prior resume and job-normalization JSON goldens remain byte-equivalent
- complete and vague job-match JSON goldens remain byte-equivalent; three compact risky projections pass; vague-job text output is provisional and suppresses top gaps
- all schemas pass Draft 2020-12 metaschema validation
- all five match inputs, both full match outputs, and representative prior outputs pass `check-jsonschema`
- selected complete fixture matches the executed Django projection for all six raw/category scores, overall score, weights, exact skill details, experience metrics, seniority, keywords, and education/certification signals
- the compact reference projection regenerates byte-for-byte from the Django scorer
- all 17 equivalence groups and 43 aliases exactly match `conservative_skill_equivalence_v1`
- exact aliases, explicit versions, close non-equivalents, vague jobs, weak resumes, partial core evidence, unassessed qualifications, Unicode, prompt-like text, limits, and nested typed errors are covered
- dependency tree contains no network, provider, TLS, async-runtime, or telemetry stack
- relative Markdown links, README capability JSON, CI YAML, and Agent Skill checks pass
- credential-pattern scan reports no findings
- Fallow changed-code and security checks report no findings; Fallow does not currently analyze Rust source for health metrics
- `git diff --check`
- clean-clone formatting, Clippy, 103-test, locked-build, rustdoc, capabilities, all prior goldens, and complete/vague match goldens pass
- GitHub Actions PR run `30021879854` — passed, including Rust 1.85 compatibility, 103 tests, locked build, and all CLI golden checks

## Next phase after approval

Phase 5 will harden agent-facing CLI/distribution only after Phase 4B is fully verified, reviewed, merged, and separately approved.
