# Assisted resume variant v1

## Scope

The Phase 7 variant boundary deterministically reviews and materializes explicitly selected external resume changes. It performs no model/provider call and does not certify generated prose as factually true.

Operations:

```text
career resume variant-review
career resume variant-materialize
```

Capabilities:

```text
resume.variant.review
resume.variant.materialize
```

## Contracts

- `career.resume_variant_proposal.v1` — at most 50 untrusted line-targeted changes
- `career.resume_variant_review_input.v1` — exact resume/vacancy inputs plus proposal
- `career.resume_variant_review.v1` — canonical retained changes, discard diagnostics, and assisted preview
- `career.resume_variant_materialization_input.v1` — same exact review input plus selected canonical IDs
- `career.resume_variant.v1` — exact baseline plus selected assisted candidate

Review and materialization composite JSON envelopes are limited to 1,048,576 bytes by CLI and Swift adapters before parsing.

## Change validation

Each proposed change includes:

- one bounded section label
- 1-based inclusive start/end lines
- exact nonempty original target text
- bounded proposed replacement text
- one-to-five unique exact resume evidence excerpts
- one-to-five unique exact vacancy evidence excerpts

The original target must match exactly at the supplied line range. The core never searches another occurrence or repairs a range. Replacement and target strings are each limited to 10,000 Unicode scalar values; the complete proposal is limited to 100,000 string scalars. Unsupported control characters are rejected.

Invalid individual changes receive bounded discard codes. Any duplicate or intersecting target causes every member of that overlap set to be discarded. Remaining changes are ordered by source position and receive `change-0001`-style core identifiers. Provider identifiers are not accepted.

The all-change preview must still satisfy the ordinary 50,000-scalar, 2,000-line, and 2,000-scalar-per-line resume bounds.

## Materialization

Materialization repeats review from the complete original input. The selected list must contain one-to-50 unique canonical identifiers from that repeated review.

Selected non-overlapping replacements are applied in stable reverse source order. Unselected ranges and all bytes outside selected target ranges remain unchanged. The final candidate must satisfy ordinary resume bounds.

The output includes:

- byte-equivalent original `baseline_resume`
- exact `assisted_resume_text`
- canonical `selected_changes`
- `assisted_non_authoritative` authority
- three mandatory warnings

## Authority and limitations

Exact evidence occurrence establishes only that an excerpt occurs in a supplied source. It does not establish semantic entailment, factual completeness, or truth of generated wording. Human review remains mandatory.

Assisted variants cannot enter `resume analyze` or `job match`; those operations accept only original input and rerun deterministic normalization. The core does not persist, export, name, transmit, or automatically accept a variant.

## Determinism and privacy

Identical versioned input produces byte-equivalent canonical output. Ordering uses source positions and canonical vectors, never hash iteration. Errors and discard diagnostics contain bounded codes and paths without source, replacement, evidence, prompt, or provider payloads.
