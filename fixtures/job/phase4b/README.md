# Phase 4B job-match fixtures

All resumes, organizations, vacancies, identifiers, and contact details in this directory are synthetic.

## Fixtures

- `complete-match` proves all six deterministic category rules, high-confidence scoring, exact skill matching, bounded strengths/gaps, and apply-now safety gates.
- `alias-match` proves only reviewed same-technology aliases can satisfy skill evidence.
- `close-non-equivalent` adapts adjacent-technology risks and proves related tools remain gaps.
- `vague-job` proves low-confidence job parsing bounds every category to 50–75, suppresses inferred top gaps, and makes recommendation guidance provisional.
- `weak-resume-narrow-job` proves required gaps and weak core categories cannot produce overconfident guidance.

The complete and vague `.expected.json` files are reviewed full canonical outputs from `job_match_v1`; repeated CLI runs must remain byte-equivalent. `risky-expectations.json` keeps the alias, adjacent-technology, and weak-resume regression projections compact while public-contract tests execute their full inputs and compare scores, gap counts, match types, and recommendation labels.

## Reference projection

`job-match-deterministic-v2-reference.json` was produced by executing the read-only Django `job_match_deterministic_v2` scorer against normalized facts equivalent to `complete-match.input.json` and the identical source texts.

```text
source_repository: resume-ai
source_paths:
  - accounts/services/job_match_scoring.py
  - accounts/services/job_match_skill_equivalence.py
  - accounts/services/job_match_recommendation.py
  - accounts/test_job_match_skill_equivalence.py
  - accounts/test_job_match_recommendation.py
  - accounts/test_job_match_fixtures.py
  - accounts/test_fixtures/job_match/close_non_equivalent_skills.json
  - accounts/test_fixtures/job_match/vague_job_requirements.json
  - accounts/test_fixtures/job_match/weak_resume_narrow_job.json
reference_versions:
  - job_match_deterministic_v2
  - conservative_skill_equivalence_v1
  - job_match_recommendation_safety_v1
rust_policy_versions:
  - job_match_v1
  - job_match_recommendation_v1
changes: synthetic reduction; raw input envelope; source spans; deterministic-only recommendation; no model, merge, persistence, or user data
```

The projection records one intentional safety difference: Django's direct substring domain rule detects `ui` inside words such as `building` and `requirements`; Rust requires complete normalized phrases. The selected category score is unchanged because the false `frontend` signal appears on both sides in the reference result.

Fixture parity is limited to the enumerated deterministic rules and evidence. It does not establish parity for LLM interpretation, provider fallback, merge, prompts, caching, observability, Django APIs/models, or application persistence.
