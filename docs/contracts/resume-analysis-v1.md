# Deterministic resume analysis v1

## Status and compatibility

`career.resume_analysis.v1` is the Phase 3 full deterministic resume-readiness contract. Its Rust policy version is `resume_analysis_v1`; the scoring-rule reference is Django `deterministic_v2`.

This is an additive operation:

```bash
career resume analyze --input <path|-> [--format json|json-pretty|json-compact|text]
```

The earlier `career resume evaluate` command and `career.resume_evaluation.v1` section-coverage output remain unchanged. Callers must not substitute one contract for the other.

## Input and authoritative baseline

The command accepts `career.resume_input.v1` plain text. It validates and normalizes that text with `resume_normalization_v1`, then scores only the resulting `deterministic_document`.

The score source is always:

```text
deterministic_normalization_baseline
```

External proposals and `assisted_document` values cannot be supplied to this operation and cannot change its confidence, checks, evidence, or score. A host may display assisted normalization separately, but authoritative analysis must call `resume analyze` with the original input.

Input limits are inherited from the resume input and normalization contracts:

| Limit | Value |
|---|---:|
| Source text | 50,000 Unicode scalar values |
| Source lines | 2,000 |
| Individual source line | 2,000 Unicode scalar values |
| Document identifier | 128 Unicode scalar values |
| CLI JSON input | 262,144 bytes |
| Checks | exactly 18 |
| Evidence records per check | 2 |
| Evidence value | 240 Unicode scalar values |
| Source excerpt | 160 Unicode scalar values |
| Strengths | 3 |
| Weaknesses | 3 |
| Improvement actions | 3 |
| Warnings | 6 |

## Categories and aggregation

The six category weights sum to 100:

| Category | Weight |
|---|---:|
| `format_ats` | 20 |
| `content_strength` | 25 |
| `experience_impact` | 20 |
| `skills_coverage` | 15 |
| `presentation` | 10 |
| `completeness` | 10 |

Each category score is the arithmetic mean of its adjusted check scores. Category means and the final weighted mean use deterministic round-half-to-even integer arithmetic. Individual ratio checks truncate toward zero, matching the reference policy. No floating-point value is authoritative.

A check passes at 60 or above. Every raw, adjusted, category, and overall score is an integer in `0..=100`.

## Check catalog

Checks are always emitted in this canonical order:

| Check ID | Category | Raw scoring rule |
|---|---|---|
| `contact_email_present` | `completeness` | 100 when normalized email is present, otherwise 0 |
| `contact_name_present` | `completeness` | 100 when normalized name is present, otherwise 0 |
| `contact_phone_or_link_present` | `completeness` | 100 when a phone or profile link is present, otherwise 0 |
| `summary_present` | `content_strength` | 100 when normalized summary is non-empty, otherwise 0 |
| `experience_section_present` | `completeness` | 100 when at least one experience entry exists, otherwise 0 |
| `education_section_present` | `completeness` | 100 when at least one education entry exists, otherwise 0 |
| `skills_section_present` | `skills_coverage` | 100 when at least one skill exists, otherwise 0 |
| `experience_bullets_present` | `experience_impact` | 100 when any experience bullet exists, otherwise 0 |
| `experience_bullet_length_quality` | `experience_impact` | percentage of bullets containing 5–35 whitespace-separated words |
| `experience_date_ranges_present` | `presentation` | percentage of experience entries containing a date range |
| `experience_chronology_consistency` | `presentation` | percentage of entries whose date range contains a 19xx/20xx year, `present`, or `current` signal |
| `measurable_impact_in_bullets` | `experience_impact` | percentage of bullets containing a digit or `%` |
| `skills_count_quality` | `skills_coverage` | 0 skills = 0; 1–2 = 35; 3–4 = 60; 5–7 = 80; 8+ = 100 |
| `weak_phrasing_penalty` | `content_strength` | 0 weak-phrase hits = 100; 1 = 80; 2 = 60; 3 = 40; 4+ = 20 |
| `keyword_repetition_penalty` | `presentation` | top significant-word frequency ratio: ≤5%=100, ≤8%=80, ≤12%=60, ≤16%=40, otherwise 20; fewer than 10 words scores 100 |
| `ats_section_header_clarity` | `format_ats` | percentage of summary, experience, education, and skills fields populated by deterministic normalization |
| `ats_line_density` | `format_ats` | average non-empty-line density: 3–16 words=100, 2–20=75, 1–25=50, otherwise 25 |
| `parse_quality_proxy` | `format_ats` | high=100, medium=75, low=50, unknown=25 |

Weak phrases are the explicit, case-insensitive list:

- `responsible for`
- `worked on`
- `helped with`
- `various`
- `etc`
- `duties included`

“Chronology consistency” retains the reference check identifier and rule. It recognizes basic date signals; it does not prove chronological ordering or employment duration.

## Parser uncertainty

The analysis reuses normalization field statuses for contact, summary, experience, education, and skills:

- `detected`: deterministic normalized content exists
- `likely_missing`: sufficient structure exists to treat an empty field as a conclusive gap
- `not_detected`: limited parser confidence makes absence inconclusive

When a field is `not_detected`, each associated missing-data check below 50 is raised to the fixed score floor of 50. The result retains both `raw_score` and `score`, sets `score_adjusted: true`, and reports `outcome: inconclusive`. The check remains failed because 50 is below the passing threshold.

An explicitly detected but empty section is `likely_missing`; its related primary check remains at 0 and is conclusive. Once a field is detected, its content-quality checks are not upgraded merely because the overall parse confidence is low.

The confidence context lists uncertain and likely-missing fields, deterministic fallbacks, all adjusted check IDs, and the complete five-signal parse-confidence record.

## Evidence and findings

Every check contains at least one bounded evidence record. Evidence may contain:

- a normalized field and its one-based source span
- a deterministic count or ratio
- parser-confidence data
- a field-status adjustment

No evidence is fabricated for absent source content. Evidence IDs, check IDs, category order, finding order, and warning order are stable.

Top strengths follow category score descending order and use the first passing canonical check as their reason. A strength is `provisional` when another check in the same category is inconclusive.

Top weaknesses follow category score ascending order. Conclusive failed checks are preferred over inconclusive checks. If a category has only inconclusive failures, its title and status are provisional.

Improvement actions are static deterministic mappings from the selected weakness check. Each action includes its basis check ID and finding status. Provisional actions instruct callers to verify extracted text first. Actions never introduce employers, skills, metrics, dates, credentials, or other facts absent from the source/check evidence.

## ATS-readiness boundary

`format_ats` represents general text-extraction and structural readiness signals only. It does not reproduce or predict any proprietary applicant-tracking system.

Every result warns that it does not evaluate:

- proprietary ATS ranking or employer-specific weighting
- knockout questions, recruiter searches, or hiring outcomes
- visual layout, columns, tables, text boxes, fonts, or headers/footers
- PDF/DOCX conversion fidelity or OCR quality

Those document-ingestion diagnostics belong in a future adapter and must remain separately labeled.

## Reference compatibility matrix

The following `deterministic_v2` behavior is ported:

| Reference behavior | Status |
|---|---|
| 18 check identifiers and canonical check order | ported |
| six categories and 20/25/20/15/10/10 weights | ported |
| check scoring tiers and ratios | ported |
| pass threshold of 60 | ported |
| category means and weighted overall score | ported |
| Python-compatible round-half-to-even results | ported |
| low/unknown confidence score floor of 50 | ported |
| detected / likely-missing / not-detected adjustment behavior | ported |
| deterministic summary strongest/weakest tie order | ported |
| top-strength and top-weakness category ordering | ported, with additional provisional strength labeling |
| LLM-generated improvement guidance | intentionally excluded |
| Django IDs, timestamps, persistence, credits, and API envelopes | intentionally excluded |
| scoring of LLM-merged normalization fields | intentionally excluded; Rust scores the deterministic baseline only |
| source spans, raw scores, outcomes, and basis IDs | Rust-specific explainability additions |

For `complete-analysis` and `messy-analysis`, all 18 check scores, pass states, detection statuses, explanations, six category scores, and overall scores are tested against `fixtures/resume/phase3/deterministic-v2-reference.json`, a compact projection generated by executing the reference scorer with equivalent normalized facts.

This establishes scoring-rule parity and selected fixture parity. It does not claim end-to-end Django parser, provider, persistence, or API-response parity.

## Determinism

For identical valid input and the same core version, typed output and pretty JSON are byte-equivalent. The operation reads no clock, locale, environment variable, filesystem, randomness, model, or network state. Output ordering uses fixed enums and vectors rather than hash iteration.

## Reference provenance

```text
source_repository: resume-ai
source_path: accounts/services/resume_scoring.py
reference_version: deterministic_v2
changes: ported pure scoring and confidence-aware adjustment rules to bounded Rust types; added source evidence, raw scores, outcomes, deterministic actions, and baseline-only authority
```

```text
source_repository: resume-ai
source_path: accounts/services/resume_analysis_response.py; accounts/services/resume_analysis_preview.py
reference_version: deterministic_v2 response behavior
changes: ported deterministic strength/weakness ordering; replaced LLM actions with bounded check-derived actions; omitted persistence identifiers and timestamps
```

```text
source_repository: resume-ai
source_path: accounts/test_services.py (ResumeScoringServiceTests); accounts/test_normalization_fixtures.py
reference_version: deterministic_v2
changes: reduced to synthetic Rust unit and golden fixtures with explicit equivalent-normalization comparisons
```
