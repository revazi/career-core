# Deterministic job match v1

## Versions

- input schema: `career.job_match_input.v1`
- output schema: `career.job_match.v1`
- Rust policy: `job_match_v1`
- selected scoring reference: `job_match_deterministic_v2`
- skill-equivalence policy/reference: `conservative_skill_equivalence_v1`
- Rust recommendation policy: `job_match_recommendation_v1`
- recommendation reference: `job_match_recommendation_safety_v1`
- resume normalization: `resume_normalization_v1`
- job normalization: `job_normalization_v1`

Schemas:

- [`job-match-input-v1.schema.json`](../../schemas/job-match-input-v1.schema.json)
- [`job-match-v1.schema.json`](../../schemas/job-match-v1.schema.json)

## Scope

This operation compares one bounded resume and one bounded job description. It is deterministic, local, source-grounded, and conservative.

It is **not**:

- a hiring prediction or recruiter decision
- a proprietary ATS score or vendor reproduction
- fuzzy or embedding similarity
- an LLM interpretation
- proof that an unmentioned skill or qualification is absent
- document-layout, PDF, DOCX, OCR, or visual inspection
- vacancy URL fetching, company research, persistence, or application tracking

## Input

```json
{
  "schema_version": "career.job_match_input.v1",
  "resume": {
    "schema_version": "career.resume_input.v1",
    "text": "...",
    "metadata": { "document_id": "resume-1" }
  },
  "job": {
    "schema_version": "career.job_input.v1",
    "text": "...",
    "metadata": { "document_id": "job-1" }
  }
}
```

The operation accepts original plain-text inputs, not caller-supplied normalized or assisted documents. It independently reruns both deterministic normalization policies. Consequently, values accepted through `resume enrich` cannot alter authoritative match scores, evidence, gaps, or recommendations.

Nested validation errors use `resume.` or `job.` field-path prefixes. Existing resume and job limits still apply: 50,000 Unicode scalar values, 2,000 lines, 2,000 scalar values per line, and 128 scalar values per optional document identifier. The composite match CLI envelope is read with a separate 1,048,576-byte ceiling; existing single-document CLI commands retain their 262,144-byte ceiling.

## Pipeline

```text
career.job_match_input.v1
        │
        ├─ resume input ─► resume_normalization_v1 baseline
        │
        └─ job input ────► job_normalization_v1 baseline
                              │
                              ▼
                         job_match_v1
                              │
                              ├─ six category scores
                              ├─ source/derived evidence
                              ├─ confidence bounds
                              ├─ strengths and gaps
                              └─ deterministic recommendation gates
```

No provider, clock, filesystem, environment variable, random value, or process state participates.

## Category scores

All raw and published category scores are integers from 0 through 100. Categories and weights are fixed in this order:

| Category | Weight |
|---|---:|
| `skills_match` | 30 |
| `experience_match` | 25 |
| `seniority_fit` | 15 |
| `domain_fit` | 10 |
| `keyword_alignment` | 10 |
| `education_fit` | 10 |

The overall score is the weighted mean. Ratios and weighted means use deterministic round-half-to-even integer arithmetic.

### Skills match

Required and preferred job skills are compared against deterministic resume skills.

1. Normalize case, punctuation, and whitespace.
2. Accept normalized exact equality.
3. Otherwise accept only a reviewed same-technology alias with compatible explicit versions.
4. Never accept adjacent, competing, or merely transferable technologies.

Required and preferred group scores are matched count divided by total count. When both groups exist, the category uses 70% required and 30% preferred. If only one group exists, that group is authoritative. If neither exists, the raw score is zero; low/unknown parsing later makes that uncertainty visible.

The explicit alias groups are:

- AWS / Amazon AWS / Amazon Web Services
- GCP / Google Cloud / Google Cloud Platform
- JavaScript / JS
- TypeScript / TS
- PostgreSQL / Postgres
- Kubernetes / K8s
- Node / Node.js / NodeJS
- React / React.js / ReactJS
- Vue / Vue.js / VueJS
- Next.js / NextJS
- .NET / DotNet / Net
- C# / C Sharp / CSharp
- C++ / C Plus Plus / CPP
- Go / Golang
- Machine Learning / ML
- Artificial Intelligence / AI
- CI/CD / the two listed continuous-integration-and-delivery phrases

An explicit version on one side may match an unversioned name. Two differing explicit versions do not match. Thus Python 3.11 may match Python, while Python 2 does not match Python 3.

Examples that remain non-equivalent include Kubernetes/Docker, PostgreSQL/MySQL, React/Angular, AWS/Azure, Terraform/CloudFormation, Django/Flask, Java/JavaScript, C/C++, and Machine Learning/Artificial Intelligence.

### Experience match

The strongest explicit job year requirement is the maximum integer from forms such as `5 years`, `5+ years`, or `3-5 years`.

Resume years use explicit four-digit years in normalized experience date ranges. The estimate is latest explicit year minus earliest explicit year, with a minimum of one. `Present`, `Current`, or today's year is not invented; unclear date ranges produce an unknown estimate.

- no explicit job requirement: 100 with resume experience, otherwise 0
- explicit requirement but unclear resume dates: 50 with experience entries, otherwise 0
- estimate meets requirement: 100
- estimate is below requirement: proportional integer score

A below-target explicit estimate is a `partial_match`, not a fabricated confirmed absence.

### Seniority fit

Only explicit `junior`, `mid`, `senior`, `lead`, `staff`, or `principal` signals are compared. The fixed order is junior < mid < senior < lead < staff < principal.

- no job signal: 100 when the resume has a signal, otherwise 50
- job signal but no resume signal: 0
- same level: 100
- one level apart: 70
- more than one level apart: 30

### Domain fit

Six inspectable domain groups are detected from bounded source text and normalized facts: backend, frontend, data, DevOps/cloud, mobile, and product. Signals must appear as complete normalized words or phrases. Matched job-domain count divided by job-domain count produces the score. A job with no detected domain receives the neutral raw score 50.

Canonical reviewed skill aliases are included when detecting domains, so `ReactJS` can support the same frontend signal as `React`. Domain groups remain coarse derived signals, not skill equivalence.

### Keyword alignment

The policy extracts lowercase ASCII tokens matching `[a-z0-9+#]{3,}`, removes a fixed stopword list, ranks by descending frequency with first occurrence as the stable tie-break, and evaluates at most the first 20 job keywords.

Canonical reviewed resume-skill names are added only to the resume membership set. This prevents a confirmed AWS/Amazon Web Services alias from simultaneously appearing as an AWS keyword gap. No fuzzy, stemming, or semantic expansion occurs.

Keyword findings are lexical hints, not hard qualification claims. Low/unknown normalization suppresses keyword items from `top_gaps`.

### Education fit

Only explicit degree and certification terms from the selected reference policy are compared. No school prestige, field equivalence, credential level, or “equivalent experience” inference is attempted.

- no education/certification requirement: 100
- recognized explicit signal: 100 if supported, otherwise 0
- unrecognized requirement text: 50 when the corresponding resume entries exist, otherwise 0
- education and certification sub-scores are averaged when both groups exist

Generic required qualifications that cannot be assessed by these lexical categories are preserved separately and block `apply_now`.

## Category evidence

Each category result contains:

- canonical category and weight
- `raw_score`
- confidence-aware published `score`
- `score_adjusted`
- bounded explanation
- at most 100 comparison items
- at most 12 derived metrics

Item statuses are:

- `confirmed_match`: explicit normalized equality, reviewed alias, or supported derived signal
- `partial_match`: explicit evidence supports only part of a requirement
- `likely_missing`: a high/medium-confidence baseline does not clearly support the explicit signal
- `unverified`: parser confidence or truncation makes a missing/partial claim unsafe

Skill, education, certification, experience, and seniority items retain job source spans when available. Resume evidence retains resume source spans when available. Keyword and domain signals may be source-derived without one unique line and therefore use null spans.

## Confidence policy

`confidence_context` includes the full deterministic resume and job parse-confidence records and both normalization truncation flags.

A match is uncertain when any of these holds:

- resume confidence is `unknown` or `low`
- job confidence is `unknown` or `low`
- resume normalization output was truncated
- job normalization output was truncated

For an uncertain match:

- each published category score is clamped to 50–75
- every partial or missing item becomes `unverified`
- inferred keyword/domain items are omitted from `top_gaps`
- the recommendation is `provisional`
- `apply_now` is impossible

Confirmed source-grounded matches remain confirmed. The output always preserves raw category scores and lists adjusted categories.

## Strengths and gaps

`top_strengths` contains at most five de-duplicated confirmed matches. Ordering is stable: higher category score, canonical category order, then case-normalized item.

`top_gaps` contains at most five de-duplicated findings. Priority is:

1. required skills
2. experience
3. seniority
4. education/certification
5. preferred skills
6. domain
7. keywords

Gap status is `partial`, `likely_missing`, or `unverified`. Uncertain broad domain and keyword misses are suppressed so a vague job cannot create invented hard requirements.

## Recommendation policy

Labels are:

- `improve_first`
- `apply_after_small_edits`
- `apply_now`

The overall thresholds and apply-now core threshold are ported from `job_match_recommendation_safety_v1`, with an additional conservative apply-after core floor:

- overall below 60: `improve_first`
- any core category below 50: `improve_first`
- overall below 80: at most `apply_after_small_edits`
- apply-now core categories must each be at least 70: skills, experience, seniority, and education

`apply_now` is also blocked by:

- uncertain normalization
- any deterministic required-skill gap
- any partial/missing experience, seniority, education, or certification evidence
- any generic required qualification that deterministic lexical policy cannot safely assess

The core generates this recommendation directly from deterministic gates; it does not accept or upgrade an external recommendation. The recommendation status is `provisional` when normalization is uncertain or a required qualification remains unassessed. Reasons name bounded blockers and thresholds, never provider prose. Even `apply_now` is only workflow guidance and is not a hiring-outcome prediction.

Recommendation arrays contain at most ten required-skill blockers, ten core-evidence blockers, and ten unassessed qualifications.

## Mandatory warnings

Every output warns that:

1. the result is deterministic alignment only, not a hiring prediction, recruiter decision, proprietary ATS score, or guarantee
2. only caller-supplied plain text and explicit lexical evidence are evaluated; visual layout and unstated qualifications are not

Additional warnings cover uncertain normalization, truncated normalization, and unassessed required qualifications.

## Determinism and bounds

For identical versioned input and core version:

- normalization is rerun from the original inputs
- category and item order is fixed
- set-derived public values are sorted or emitted in policy order
- all scores use integer arithmetic
- strengths and gaps are limited to five each
- category items are limited to 100 and metrics to 12
- keyword targets are limited to 20
- recommendation blocker groups are limited to ten
- metric strings are limited to 240 Unicode scalar values
- source excerpts retain their normalization limits

Prompt-like text is data. It cannot change aliases, weights, thresholds, limits, warnings, or policy behavior.

## Reference parity and intentional differences

The selected complete fixture executes `job_match_deterministic_v2` against equivalent normalized facts and identical source texts. Rust matches all six raw/category scores, the overall score, scoring weights, exact required/preferred skill details, experience metrics, seniority signals, keyword lists, and education/certification signals.

Intentional differences:

- Rust requires complete normalized phrases for domains. Django direct substring checks can detect `ui` inside `building` and `requirements`; Rust rejects that false signal.
- Rust applies the reviewed alias policy directly to authoritative deterministic skill matching. Django deterministic scoring uses normalized equality and applies alias policy later while validating model proposals.
- Rust augments keyword membership with accepted canonical skill aliases to avoid contradictory alias and keyword findings.
- Rust treats uncertain resume normalization and normalization truncation conservatively in addition to Django's low/unknown job-confidence bounds.
- Rust does not read the current year. Only explicit years contribute to experience estimation.
- Rust emits source spans, typed statuses, bounded metrics, mandatory scope warnings, and deterministic strengths/gaps.
- Rust generates recommendation guidance directly from deterministic gates, adds a 50-point apply-after core floor plus core-evidence/unassessed-qualification blockers, and does not consume an LLM candidate.
- Generic qualifications are retained for review rather than semantically interpreted.

This establishes enumerated deterministic rule and selected fixture parity. It does not establish parity for LLM interpretation, provider fallback, merge, prompts, caching, observability, Django APIs/models, or persistence.
