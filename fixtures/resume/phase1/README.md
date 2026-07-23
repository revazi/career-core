# Phase 1 resume fixtures

All source text in this directory is synthetic and contains no real personal data.

## Provenance

```text
source_repository: resume-ai
source_path: accounts/services/resume_normalization.py
reference_version: resume_normalization_v7
changes: retained exact-line alias semantics for four expected sections; replaced all document content with minimal synthetic examples
```

```text
source_repository: resume-ai
source_path: accounts/test_services.py (ResumeSectionAliasTests)
reference_version: resume_normalization_v7
changes: adapted document-order and prose non-matching behavior into JSON CLI fixtures
```

## Fixtures

- `complete-sections.input.json`: all four expected aliases with following content and caller metadata
- `prompt-like-sparse.input.json`: prose and instruction-like lines that must not become headers, plus an empty recognized skills header

Each `.expected.json` file is canonical pretty JSON emitted by the Phase 1 CLI and reviewed as a golden public-contract fixture.
