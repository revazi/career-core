# Phase 2 resume normalization fixtures

These fixtures are synthetic and contain no real personal data.

## Provenance

### `complete-normalization`

```text
source_repository: career-core
source_path: fixtures/resume/phase2/complete-normalization.input.json
reference_version: resume_normalization_v7 concepts only
changes: original synthetic labeled resume covering every supported deterministic structure
```

### `messy-unlabeled`

```text
source_repository: resume-ai
source_path: accounts/test_fixtures/normalization/messy_resume.txt and messy_resume_valid_proposal.json
reference_version: resume_normalization_v7 / resume_normalization_llm_fallback_v1 / resume_normalization_fallback_merge_v1
changes: converted to versioned career-core input contracts; retained synthetic names and reduced metadata; external proposal is explicit caller input rather than a provider response fetched by the core
```

The messy fixture demonstrates a low-confidence deterministic baseline, conservative deterministic education recovery, source-grounded proposal validation, and assisted-only merging for empty Summary, Experience, and Skills fields. The deterministic baseline and confidence remain unchanged in the enrichment result.
