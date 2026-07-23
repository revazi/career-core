# Architecture

## Dependency direction

```text
career-cli ───────► career-core
future Swift API ─► career-core
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

## Core boundary

The library accepts bounded typed values and returns typed deterministic results. It may:

- normalize plain text
- calculate confidence and scoring signals
- analyze only deterministic normalization baselines with versioned checks and integer aggregation
- compare explicit evidence
- emit provider-neutral enrichment eligibility and target contracts
- validate source-grounded external proposals as untrusted typed input
- merge accepted proposal values only into eligible empty fields while preserving the baseline
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

## Future Swift boundary

Swift bindings belong in an adapter crate introduced only in the Swift phase. Prefer a small stable facade over exposing internal Rust types directly. The Swift adapter must translate core errors and owned contract types without moving platform behavior into the core.

No FFI dependency or `unsafe` exception is approved during the current phases. A future Swift host owns Keychain access, consent settings, provider clients, and network calls; it submits only the proposal contract to the core.
