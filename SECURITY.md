# Security Policy

## Supported versions

`career-core` is pre-release software. Security fixes are applied to the latest revision on `main` until the first stable release policy is documented.

## Reporting a vulnerability

Use GitHub private vulnerability reporting when the repository is public. Do not include sensitive resume content, job descriptions, credentials, or API keys in a report. If private reporting is unavailable, contact the maintainer before sharing exploit details publicly.

## Security model

The core library processes untrusted career-document text and structured data locally. It must:

- perform no implicit network requests
- execute no instructions embedded in documents
- avoid logging source payloads
- reject oversized or invalid structured input at public boundaries
- return typed errors instead of panicking on user-controlled data
- keep output bounded and deterministic
- treat external normalization proposals as untrusted data requiring exact shape, limits, eligible targets, and source grounding
- preserve deterministic output when assisted validation fails
- treat external analysis-suggestion targets, evidence, and text as untrusted; rerun the analysis, require current failed-action/source binding, discard ambiguous items without payload echoes, and never claim occurrence proves factual entailment or rewrite certification
- treat resume-variant targets, evidence, generated replacements, and selected identifiers as untrusted; discard ambiguous/overlapping changes and never claim evidence occurrence proves factual entailment
- materialize only revalidated explicitly selected canonical changes while preserving the exact baseline and every unselected range
- score only the deterministic normalization baseline and never assisted proposal or variant values
- label parser uncertainty and general ATS-readiness limitations without claiming proprietary ranking behavior
- treat job fields not detected by lexical classification as unverified rather than confirmed absent
- require caller-supplied job text and never fetch vacancy URLs from the core
- rerun deterministic resume and job normalization for matching rather than accepting assisted or caller-forged baselines
- allow only normalized exact or reviewed same-technology aliases; related technologies remain non-equivalent
- bound uncertain match scores and label missing/partial evidence as unverified
- prevent recommendation guidance from exceeding deterministic score, confidence, core-category, and gap gates

The CLI may read only input paths explicitly supplied by the caller and never makes a provider call. Public schemas are reviewed files embedded at compile time; schema discovery does not search caller paths, load plugins, generate code, or use the network. Compact and pretty machine modes preserve stdout/stderr separation and bounded payload-free errors.

The Swift adapter accepts owned JSON strings only, applies pre-parse UTF-8 byte limits, and maps failures into bounded generated Swift errors. It opens no files and performs no platform, persistence, credential, provider, or network behavior. Project-authored Rust contains no unsafe block; FFI allocation and C ABI behavior are isolated to exact UniFFI `0.30.0` generated/runtime code. XCFrameworks and checksums are local ignored artifacts until a separately approved publication review includes [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

Hosts that send resumes to an external model must obtain user consent, keep API keys outside core contracts and logs, bound provider context, isolate failures, and label accepted values as assisted.
