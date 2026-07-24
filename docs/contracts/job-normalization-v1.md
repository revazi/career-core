# Deterministic job-description normalization v1

## Status

`career.job_normalization.v1` is the Phase 4A deterministic job-description contract. Its Rust policy is `job_normalization_v1`; its selected deterministic reference is Django `job_description_normalization_v6`.

Invoke it with:

```bash
career job normalize --input <path|-> [--format json|json-pretty|json-compact|text]
```

The operation accepts caller-supplied plain text only. It does not fetch vacancy URLs, open documents, call a provider, apply an external proposal, or perform resume-to-job matching. Matching is exposed separately through `career.job_match.v1`.

## Input

`career.job_input.v1` contains:

- `schema_version`: exactly `career.job_input.v1`
- `text`: plain job-description text supplied by the caller
- `metadata.document_id`: optional caller-controlled correlation identifier

Metadata never affects normalization or confidence.

Whitespace-only text is invalid. LF and CRLF are supported. Limits are measured as Unicode scalar values unless stated otherwise:

| Limit | Value |
|---|---:|
| Source text | 50,000 characters |
| Source lines | 2,000 |
| Individual source line | 2,000 characters |
| Document identifier | 128 characters |
| CLI JSON input | 262,144 bytes |
| Source excerpt | 160 characters |
| Required skills | 50 |
| Preferred skills | 50 |
| Required qualifications | 50 |
| Preferred qualifications | 50 |
| Each requirement group | 50 |
| Responsibilities | 50 |
| Seniority signals | 20 |
| Stored unmatched-line excerpts | 10 |

Limits are checked before regex classification. Reaching a structured-list or unmatched-excerpt limit produces explicit truncation metadata and a warning.

## Output fields

`deterministic_document` contains source-grounded values for:

- title
- company
- required and preferred skills
- required and preferred non-skill qualifications
- explicit seniority signals
- experience requirements
- education requirements
- certification requirements
- responsibilities

Every non-empty value includes a one-based physical source line, bounded source excerpt, and transformation:

- `verbatim`
- `list_item_cleaned`
- `label_prefix_removed`
- `delimiter_split`
- `normalized_case`

Values are de-duplicated case-insensitively while preserving first source order. Empty fields remain empty; normalization does not infer facts or rewrite prose.

Field statuses are emitted in canonical output order. `not_detected` means only that deterministic classification did not recover the field. It never confirms that the employer has no such requirement.

## Section recognition

Headers are matched case-insensitively against the entire normalized line. Normalization trims whitespace, removes colon characters, lowercases, and collapses whitespace. Prose containing a header phrase does not match.

### Required

`requirements`, `minimum qualifications`, `basic qualifications`, `required qualifications`, `required skills`, `candidate requirements`, `essential qualifications`, `essential skills`, `key qualifications`, `key requirements`, `must have`, `must haves`, `must-have skills`, `role requirements`, `what we are looking for`, `what we're looking for`, `what you bring`, `what you need`, `your qualifications`, `qualifications`, `who you are`

### Preferred

`preferred qualifications`, `preferred skills`, `nice to have`, `bonus points`, `preferred experience`, `additional qualifications`, `bonus skills`, `desired qualifications`, `desired skills`, `good to have`, `highly desired`, `nice-to-have`, `nice-to-have skills`, `preferred`, `preferred qualifications and skills`, `what would be a plus`

### Responsibilities

`responsibilities`, `what you'll do`, `what you will do`, `about the role`, `job responsibilities`, `day-to-day responsibilities`, `duties`, `duties and responsibilities`, `job duties`, `key duties`, `key responsibilities`, `position responsibilities`, `role and responsibilities`, `role responsibilities`, `the role`, `what you'll be doing`, `what you’ll do`, `your impact`, `your responsibilities`

### Noise boundaries

`benefits`, `about us`, `about the company`, `about our company`, `company overview`, `compensation`, `compensation and benefits`, `diversity and inclusion`, `equal opportunity`, `our company`, `our culture`, `our mission`, `our values`, `perks`, `perks and benefits`, `who we are`, `why join us`, `why work with us`

Noise-section content cannot create skills, seniority, experience, education, certification, or responsibility signals.

## Classification

The first non-empty line is retained as the title candidate. Confidence separately records whether it is plausible. The second non-empty line supplies company only when it starts with `Company:` or `At `.

Skills are extracted only from required/preferred section context or explicit inline prefixes. Candidate lists split on commas, slashes, pipes, semicolons, or bullets. Bounded skill-context forms include:

- experience with / in / of / using
- proficiency with / in / of / using
- knowledge with / in / of / using
- expertise with / in / of / using
- familiarity with / in / of / using

Requirement sentences, responsibility actions, long prose fragments, section labels, degree/certification/year phrases, and action gerunds are rejected as skills. Required and preferred groups remain separate.

Non-skill qualifications retain requirement sentences under their section. Requirement-like lines misplaced under Responsibilities are preserved as required qualifications rather than responsibilities.

Responsibilities are accepted only within a responsibility section. Explicit requirements and skill-list lines are excluded.

Explicit patterns recover:

- seniority: junior, mid, senior, lead, principal, staff
- experience: `N years`, `N+ years`, or `N-M years`
- education: bachelor, master, PhD, doctorate, or degree phrases
- certification: certification, certified, certificate, AWS Certified, PMP, or CPA phrases

These matches are lexical evidence, not semantic equivalence or proof that a condition is mandatory.

## Metadata

Matched headers preserve document order and physical source line numbers. Metadata also contains:

- at most ten bounded unmatched-line excerpts
- total unmatched-line count
- unmatched-excerpt truncation status
- structured-output truncation status

Title, detected company, recognized headers, classified requirements/responsibilities, explicit inline groups, and noise lines do not become unmatched diagnostics.

## Parse confidence

Six integer signals sum to 100:

| Signal | Maximum | Rule summary |
|---|---:|---|
| `text_quality` | 15 | text length 0/2/4/6/8, line count 0/2/3/4, readable ratio 0/1/2/3 |
| `title_detection` | 15 | plausible title 15, weaker bounded candidate 8, otherwise 0 |
| `skills_detection` | 30 | 0 skills=0, 1=10, 2–3=18, 4–5=25, 6+=30 |
| `responsibilities_detection` | 20 | 0=0, 1=10, 2–3=16, 4+=20 |
| `requirement_detection` | 10 | 2 points each for seniority, experience, education, certification, and qualification groups |
| `structure_quality` | 10 | classified-line ratio plus recognized sections, capped at 3 for prose-heavy layouts |

Labels:

- `high`: total at least 75, title score 15, skills at least 18, responsibilities at least 10, and structure at least 6
- `medium`: total at least 45 and text quality at least 6
- `low`: any other valid non-empty input
- `unknown`: reserved by the output enum; empty input is rejected at the Rust boundary

Ratio evidence is represented as deterministic integer per-mille values. No floating-point value is authoritative.

## Warnings

Every output includes `limited_normalization_scope`. Additional warnings report:

- no recognized section headers
- unclassified lines
- low/unknown provisional confidence
- bounded output truncation

Consumers must retain provisional-confidence warnings. Future matching must not convert missing low-confidence classifications into confirmed job requirements or resume gaps.

## Reference compatibility

The following deterministic `job_description_normalization_v6` behavior is ported:

- all four explicit section-header groups
- title and company candidates
- required/preferred skills and qualifications
- responsibility/requirement separation
- noise exclusion
- seniority, experience, education, and certification patterns
- stable case-insensitive de-duplication
- matched/unmatched metadata
- all six confidence signals, weights, gates, and labels

For `complete-normalization` and `prose-heavy`, every normalized field, matched/unmatched metadata record, confidence label/score, and six signal scores is tested against `fixtures/job/phase4a/job-description-normalization-v6-reference.json`, generated by executing the reference normalizer on identical source text.

Intentional differences:

- Rust adds source spans, transformations, field statuses, typed warnings, and explicit list bounds.
- Rust reports physical source line numbers; Django numbers filtered non-empty lines. Selected fixtures contain no blank lines, so the compared records are equal.
- Rust rejects empty input rather than returning an unknown empty document.
- Rust removes `At ` company prefixes case-insensitively; Django's intended check is case-insensitive but its replacement is case-sensitive.
- Provider fallback, fallback merge, persistence, API fields, and notes are excluded.
- Phase 4A normalization parity does not by itself establish job-matching parity; matching has a separate contract and reference matrix.

This establishes selected deterministic fixture parity, not provider, API, or matching parity.

## Determinism and safety

Identical valid input and core version produce byte-equivalent typed and pretty-JSON output. The core reads no files, URLs, environment variables, clocks, locale, randomness, models, or network state. Prompt-like source text is data and cannot alter policy.
