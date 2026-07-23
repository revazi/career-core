# Resume evaluation v1 contract

## Status

Phase 1 contract for the first deterministic vertical slice. It measures recognized core-section coverage only. It is not a complete resume-quality or ATS score.

## Input

`career.resume_input.v1` is a JSON object containing:

- `schema_version`: exactly `career.resume_input.v1`
- `text`: plain text already extracted by the caller
- `metadata.document_id`: optional caller-controlled correlation identifier

Metadata never influences scoring.

## Limits

Limits are measured as Unicode scalar values unless stated otherwise:

| Limit | Value |
|---|---:|
| Source text | 50,000 characters |
| Non/empty source lines combined | 2,000 lines |
| Individual source line | 2,000 characters |
| Document identifier | 128 characters |
| CLI JSON input | 262,144 bytes |
| Evidence header excerpt | 120 characters |

Whitespace-only source text is invalid. LF and CRLF line endings are supported. Limits are checked before section detection.

## Header detection

Phase 1 recognizes exact normalized aliases for four sections:

- summary
- experience
- education
- skills

Header normalization:

1. trim leading/trailing whitespace
2. lowercase using Unicode lowercase conversion
3. remove colon characters
4. collapse runs of whitespace to one ASCII space

The entire normalized line must equal a configured alias. Prose containing words such as “experience” or “professional profile” does not match.

Known Projects and Certifications aliases are recognized only as content boundaries during Phase 1. They stop content from being attributed to the preceding core section but do not produce checks or detected-section output yet.

A section is `detected_with_content` when at least one later non-empty line appears before the next recognized header boundary. A recognized header with no such line is `detected_without_content`. The first occurrence supplies header evidence; repeated occurrences may contribute content.

## Scoring

The policy version is `resume_section_coverage_v1`.

Each expected section has equal weight:

| Detection status | Check score | Passed |
|---|---:|---|
| `detected_with_content` | 100 | yes |
| `detected_without_content` | 0 | no |
| `not_detected` | 0 | no |

The evaluation score is the integer arithmetic mean of the four check scores. Because every check is either 0 or 100 and there are four checks, possible values are 0, 25, 50, 75, and 100. No floating-point arithmetic or rounding is used.

Checks are always emitted in canonical order: summary, experience, education, skills. Detected-section records preserve first appearance order in the source.

## Evidence and warnings

Recognized headers produce bounded evidence containing section, one-based line number, and a truncated source-header excerpt. Missing sections do not fabricate evidence.

Every result includes `limited_evaluation_scope`. Additional warnings distinguish:

- no recognized section headers
- some expected headers not detected
- recognized headers without following content

“Not detected” never means the underlying content is confirmed absent.

## Determinism

For the same `career-core` version and byte-identical valid input, typed output and pretty JSON field/list ordering are stable. The output does not include clocks, random identifiers, host paths, locale data, or source text beyond bounded recognized-header evidence.

## Reference provenance

Header aliases and exact-line matching are intentionally adapted from:

```text
source_repository: resume-ai
source_path: accounts/services/resume_normalization.py
reference_version: resume_normalization_v7
changes: limited Phase 1 checks to summary, experience, education, and skills; retained Projects and Certifications aliases only as conservative content boundaries; omitted parsing and fallbacks
```

Related source tests:

```text
source_repository: resume-ai
source_path: accounts/test_services.py (ResumeSectionAliasTests)
reference_version: resume_normalization_v7
changes: converted to Rust unit/integration fixtures with synthetic text
```

This phase claims a bounded rule port, not full normalization or scoring parity.
