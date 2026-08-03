# Reviewed external resume-analysis suggestions v1

## Scope

`career.resume_analysis_suggestion_review.v1` is a review-only assisted boundary for presenting bounded external suggestions beside the authoritative `career.resume_analysis.v1` baseline.

```bash
career resume analysis-suggestions-review --input <path|->
```

The core performs no model/provider call. It does not create a candidate resume, accept a selection, mutate source text, export content, or materialize a rewrite. The existing Phase 2 enrichment contract remains parser-gap recovery only and is unchanged.

## Contracts

- `career.resume_analysis_suggestion_proposal.v1` — up to three untrusted suggestion items
- `career.resume_analysis_suggestion_review_input.v1` — original `career.resume_input.v1`, expected current analysis policy, and proposal
- `career.resume_analysis_suggestion_review.v1` — rerun deterministic baseline analysis, canonical retained suggestions, payload-free discards, and mandatory warnings

CLI and Swift reject the composite input before parsing above 262,144 UTF-8 bytes. Core limits are deliberately smaller:

| Value | Limit |
|---|---:|
| Proposed suggestions | 3 |
| Total proposal string scalars | 4,000 |
| Source target | 500 scalars |
| Suggestion text | 600 scalars |
| Source evidence excerpts | 1–2 |
| Source evidence excerpt | 240 scalars |

## Review procedure

The caller supplies `expected_analysis_policy_version: "resume_analysis_v1"` and the original resume source. The core validates the envelope and proposal bounds, then independently reruns `resume_analysis_v1` from that original `career.resume_input.v1`.

Each item must contain:

- one canonical `basis_check_id`
- 1-based inclusive `start_line` and `end_line`
- an exact nonempty `source_target` at that exact physical line range
- one or two unique exact `source_evidence` excerpts occurring in the current resume source
- bounded suggestion text without unsupported controls

A retained item must bind to one current canonical `improvement_action` whose basis check remains failed. The output copies that action's priority, area, action text, and `confirmed` or `provisional` status; proposal-supplied status, action text, provider IDs, and score claims are never accepted.

At most one suggestion can bind to any action. Every member of an exact duplicate/intersecting target group is discarded, and every member of a duplicate action group is discarded. The core never chooses an arbitrary winner. Retained items sort by canonical action priority, source position, and original input index, then receive core-owned `suggestion-0001`-style identifiers.

Malformed individual items are discarded with only stable `input_index` and `code` values. Discards and errors do not echo proposal target, evidence, or suggestion payloads. A structurally invalid JSON envelope remains a normal typed CLI JSON error.

## Authority and limitations

The review result returns `baseline_analysis` as the freshly rerun, unchanged `career.resume_analysis.v1` output. Its score, checks, evidence, warnings, findings, and improvement actions remain authoritative and byte-equivalent to calling `resume analyze` on the same original input.

Retained suggestions are `assisted_non_authoritative`. Exact target/evidence occurrence proves only occurrence. It does **not** establish factual entailment, verify generated wording, or certify a rewrite. Human review is required.

No assisted suggestion can enter deterministic analysis or job matching. `resume analyze` and `job match` continue to accept original inputs and independently rerun deterministic baselines.

## Determinism and privacy

Identical versioned input produces byte-equivalent output. Ordering uses canonical vectors and ordered sets only. The operation reads no filesystem, clock, locale, randomness, network, provider, prompt, credential, database, telemetry, or UI state.

Fixtures under `fixtures/resume/phase7/` are synthetic and created for this repository. This review-only contract is not Django parity work and makes no provider or generated-content parity claim.
