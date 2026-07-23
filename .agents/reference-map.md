# Django reference map

## Location and rule

Expected sibling path:

```text
../resume-ai
```

If the repository is elsewhere, ask for its path rather than hard-coding an absolute path. Treat it as read-only. `career-core` must build and test when the reference repository is absent.

## Current reference versions

At repository bootstrap, the Django project documents these relevant policies:

| Area | Reference version |
|---|---|
| Resume normalization | `resume_normalization_v7` |
| Job-description normalization | `job_description_normalization_v6` |
| Resume deterministic scoring | `deterministic_v2` |
| Job-match deterministic scoring | `job_match_deterministic_v2` |
| Conservative skill equivalence | `conservative_skill_equivalence_v1` |
| Recommendation safety | `job_match_recommendation_safety_v1` |
| Prompt context limits | `job_match_prompt_controls_v1` |

LLM interpretation, merge, fallback, cache, and observability versions are **not** core parity targets. They may contain useful limits or fixtures, but their behavior must not enter authoritative Rust scoring.

Before a parity task, verify versions against `../resume-ai/docs/current-phase.md`; do not assume this table stays current.

## Resume normalization

Primary source:

- `accounts/services/resume_normalization.py`
- `accounts/services/resume_parsing.py`
- `accounts/services/resume_extraction.py` only for understanding boundaries; file extraction remains outside core

Tests and fixtures:

- `accounts/test_normalization_fixtures.py`
- `accounts/test_fixtures/normalization/`
- normalization-focused cases in `accounts/test_services.py`

Exclude from the core:

- `resume_normalization_fallback.py` LLM calls
- fallback prompts/provider handling
- Django model persistence

The deterministic pre-LLM result and confidence metadata are relevant. A fallback fixture may still be useful as adversarial source text, but expected LLM output is not.

## Resume evaluation

Primary source:

- `accounts/services/resume_scoring.py`
- `accounts/services/resume_analysis_response.py`
- `accounts/services/resume_analysis_preview.py`

Tests:

- deterministic portions of `accounts/test_analysis_flows.py`
- scoring-focused cases in `accounts/test_services.py`

Exclude:

- `resume_interpretation.py`
- prompts, provider output, and merge-only presentation fields
- credits and API response orchestration

## Job-description normalization

Primary source:

- `accounts/services/job_description_schema.py`
- `accounts/services/job_description_normalization.py`

Tests and fixtures:

- `accounts/test_normalization_fixtures.py`
- `accounts/test_fixtures/normalization/prose_heavy_job_description.txt`
- relevant service tests

Exclude:

- job-description LLM fallback execution and prompt code
- URL fetching from the core

## Deterministic matching

Primary source:

- `accounts/services/job_match_scoring.py`
- `accounts/services/job_match_skill_equivalence.py`
- `accounts/services/job_match_recommendation.py`
- `accounts/services/job_match_prompt_context.py` for bounded-context ideas only
- `accounts/services/job_match_response.py` for public evidence shape ideas only

Tests and fixtures:

- `accounts/test_job_match_skill_equivalence.py`
- `accounts/test_job_match_recommendation.py`
- `accounts/test_job_match_fixtures.py`
- `accounts/test_fixtures/job_match/`

High-risk fixtures to preserve semantically:

- `close_non_equivalent_skills.json`
- `vague_job_requirements.json`
- `weak_resume_narrow_job.json`
- `overbroad_tailoring_suggestions.json` for output-boundary behavior

Exclude:

- LLM interpretation, validation, fallback, merge, cache, and observability implementation
- prompt content as an authoritative source

## Fixture adaptation process

For every fixture copied or adapted, add a nearby provenance note containing:

```text
source_repository: resume-ai
source_path: <relative path>
reference_version: <policy version>
changes: <synthetic reductions/redactions/format conversion>
```

Never import real user documents. Reduce fixtures to the smallest text that proves the behavior while retaining risky edge cases.

## Parity language

Use precise labels:

- **ported**: the intended rule exists in Rust
- **fixture parity**: selected equivalent fixtures pass
- **policy parity**: all documented behaviors for a named policy version pass
- **not evaluated**: no parity conclusion is supported

Do not use “full parity” without an enumerated reference matrix and green golden suite.
