# Architecture

## Dependency direction

```text
career-cli ───────► career-core
career-swift ─────► career-core
future adapters ──► career-core

career-core ─X─► adapters, UI, persistence, network, LLMs
```

The repository root is the default `career-core` library package. Workspace packages are adapters. An adapter may translate inputs and outputs but must not duplicate or override authoritative scoring.

Authoritative resume analysis always follows the deterministic path:

```text
career.resume_input.v1
        │
        ▼
resume_normalization_v1 deterministic baseline
        │
        ▼
resume_analysis_v1 checks + integer aggregation + bounded evidence
```

`career.resume_evaluation.v1` remains the separate Phase 1 section-coverage contract. The full policy is exposed additively as `career.resume_analysis.v1`; existing command semantics are not silently replaced.

Job matching independently reproduces both deterministic baselines:

```text
career.job_match_input.v1
        │
        ├─ career.resume_input.v1 ─► resume_normalization_v1
        │
        └─ career.job_input.v1 ────► job_normalization_v1
                                          │
                                          ▼
                                     job_match_v1
                              scores + evidence + safe gates
```

Caller-supplied normalized or assisted documents cannot enter authoritative matching. The core never fetches a vacancy URL; hosts must provide bounded plain text.

Optional assisted normalization follows a split boundary:

```text
career-core deterministic normalization
        │ emits eligible targets
        ▼
host/agent provider call (explicit opt-in, outside core)
        │ submits an untrusted proposal
        ▼
career-core grounding validation + conservative merge
        │
        ├─ preserved deterministic baseline
        └─ separately labeled assisted document
```

Evidence-linked resume variants use a separate non-authoritative boundary:

```text
host/provider proposes bounded line-targeted changes
        │
        ▼
career-core exact target/evidence + structural review
        │ emits canonical selectable changes and assisted preview
        ▼
user selects canonical change identifiers outside core
        │
        ▼
career-core revalidation + deterministic materialization
        │
        ├─ byte-equivalent baseline
        └─ separately labeled assisted variant
```

Exact evidence occurrence does not prove semantic entailment or factual truth. Variant output is review/export data only and cannot enter authoritative analysis or matching.

## Core boundary

The library accepts bounded typed values and returns typed deterministic results. It may:

- normalize bounded resume and job-description plain text
- calculate confidence and scoring signals
- analyze only deterministic normalization baselines with versioned checks and integer aggregation
- compare deterministic resume/job baselines through exact and reviewed conservative equivalence
- emit bounded confidence-aware strengths, gaps, and recommendation gates
- emit provider-neutral enrichment eligibility and target contracts
- validate source-grounded external proposals as untrusted typed input
- merge accepted proposal values only into eligible empty fields while preserving the baseline
- validate bounded variant targets/evidence structurally and deterministically materialize explicitly selected changes into a separate assisted variant
- serialize public contract types through Serde

It may not:

- open files
- inspect environment variables
- read clocks, randomness, locale, or process state for scoring
- make network requests
- persist data
- call an LLM
- log source payloads
- depend on CLI or platform frameworks

When timestamps or identifiers are needed in a future contract, callers provide them; they do not influence authoritative scoring.

## Layering inside the core

Introduce modules only when their phase begins:

```text
contract/       public versioned input/output types
normalization/  pure source-to-structured transformations
evaluation/     resume checks and score aggregation
matching/       conservative evidence comparison
limits/         shared bounded-input validation
error/          typed public failures
```

Do not create empty modules or placeholder traits in advance.

## Determinism requirements

- Same core version + same input contract must produce byte-equivalent canonical JSON where documented.
- Use stable collection ordering. Prefer `Vec` and ordered maps at public boundaries.
- Normalize case and whitespace explicitly; never depend on host locale.
- Define integer score ranges and rounding rules before implementation.
- Include policy/schema versions in persisted or public results.
- Avoid hash iteration order in output.
- Keep evidence and warning limits explicit and tested.

## Error model

User-controlled invalid input must return typed errors with:

- stable machine code
- concise non-sensitive message
- optional bounded field path

Errors must not echo complete document text. Panics indicate programmer defects, never ordinary invalid input.

The CLI maps typed errors to documented nonzero exit codes and writes diagnostics to stderr. Machine results go to stdout only.

## Contract compatibility

Treat all of the following as public contracts once marked available:

- JSON property names and requiredness
- schema-version values
- enum representations
- score and confidence semantics
- check/evidence identifiers
- output ordering when documented
- CLI commands, flags, stdout framing, and exit codes

Additive compatible fields require schema/version review. Renames, removals, semantic changes, or stricter accepted input require a new contract version or major release.

## Input safety

Each operation must establish limits before accepting arbitrary documents, including:

- source character count
- line count and line length
- structured item count
- evidence count and evidence length
- recursion/nesting depth where JSON is accepted

Apply limits before expensive parsing. Untrusted text is never interpreted as instructions.

## Reference parity

The Django implementation is evidence for behavior, not architecture. Port pure rules and test cases. Do not port ORM models, HTTP serializers, service orchestration, credit handling, logs, or LLM fallback into the Rust core.

Parity must be declared per versioned operation and fixture set. Partial ports must use their own version and document differences.

LLM/provider execution is never a core parity target. Proposal bounds, grounding checks, and conservative merge rules may be adapted into provider-neutral core validation because they operate deterministically on explicit caller input. Assisted values remain separate from authoritative deterministic scoring.

Resume-analysis parity is defined at the scoring-rule and equivalent-normalized-fixture layers. Rust raw-text end-to-end output may differ when `resume_normalization_v1` intentionally differs from Django normalization. Public analysis results identify both the Rust policy and the selected Django reference policy.

## Swift boundary

`crates/career-swift` is an adapter depending inward on `career-core`. Its narrow UniFFI facade accepts and returns owned canonical JSON strings using the existing versioned schemas. This avoids making the complete internal Rust type graph an FFI ABI while allowing a future Swift host to decode app-owned `Codable` values.

The adapter maps malformed JSON, envelope limits, typed core failures, and serialization failures into generated Swift errors. It applies the same pre-parse byte ceilings as the CLI. It does not open files, inspect platform state, persist values, or perform network/provider behavior.

Project-authored Rust code contains no unsafe block. `career-swift` denies unsafe source; generated C ABI scaffolding and memory transport are isolated inside exact UniFFI `0.30.0` macros/runtime. The root `career-core` crate remains `#![forbid(unsafe_code)]` and has no FFI dependency.

The future Swift host owns SwiftUI, database access, document handling, Keychain, consent settings, provider clients, and network calls. It may submit only versioned original inputs or explicit proposal contracts; assisted values remain isolated from authoritative scoring.
