# Phase 7 assisted resume-variant fixtures

These fixtures are synthetic and were created for `career-core`. They contain no real personal data and are not adapted from the Django reference repository.

- `complete-analysis-suggestion-review.input.json` submits one source-targeted external advisory suggestion bound to the current failed measurable-impact action.
- `complete-analysis-replacement-review.input.json` submits one source-targeted exact external replacement bound to the current failed measurable-impact action.
- `complete-analysis-suggestion-review.expected.json` preserves the freshly rerun deterministic analysis and exposes one canonical assisted/non-authoritative advisory suggestion with mandatory factuality limits.
- `complete-analysis-replacement-review.expected.json` preserves the freshly rerun deterministic analysis and exposes one canonical assisted/non-authoritative before/proposed-after replacement with mandatory factuality limits.
- `complete-variant-review.input.json` proposes two bounded line-targeted changes with exact resume and vacancy evidence.
- `complete-variant-review.expected.json` preserves the baseline and exposes the two canonical non-authoritative review changes.
- `selected-variant-materialization.input.json` selects only `change-0002` from the same exact proposal.
- `selected-variant-materialization.expected.json` changes only that selected target and preserves all unselected baseline text.

The fixtures demonstrate structural/evidence validation, review-only suggestion/replacement isolation, and deterministic variant materialization. They do not demonstrate or claim semantic factuality certification.
