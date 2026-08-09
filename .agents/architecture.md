# Architecture

## Dependency direction

```text
career-cli ─────────► career-core
career-swift ───────► career-core
npm launcher ──────► verified native career-cli ─► career-core
external adapters ─► installed career-cli ─► career-core
future bindings ───► career-core

career-core ─X─► adapters, UI, persistence, network, LLMs
```

The repository root is the default `career-core` library package. Workspace packages are adapters. An adapter may translate inputs and outputs but must not duplicate or override authoritative scoring.

The CLI also owns additive managed-adapter discovery: `career.operation_catalog.v1`, deterministic embedded schema bundles, and successful machine-output enforcement. These remain CLI metadata/transport behavior and do not enter the pure library. Generic raw agents discover capabilities and schemas explicitly. A separately reviewed managed adapter may cache only non-sensitive operation/schema discovery per verified Core version; source documents, complete results, handles, projections, and persistence remain outside this repository.

The Phase 9 `@revazi/career` npm launcher is a distribution adapter around the unchanged native CLI and the only npm consumer surface. Platform packages are internal exact-version optional-dependency implementation details. Public `0.1.0` remains the completed two-target release. Candidate `0.1.1` uses one exact ordered eight-target catalog for package identities, Rust/Node/libc mappings, executable names, formats/architectures, size ceilings, provenance requirements, and target-specific file invariants. The launcher accepts only the reviewed catalog bytes, resolves only one exact package, requires positive glibc or musl evidence rather than treating non-glibc as musl, and validates Mach-O, ELF, or PE architecture before direct execution. Unix requires a regular non-symlink exact-mode file; Windows requires a regular non-symlink `career.exe` and exact PE32+ machine mapping instead of a Unix mode claim.

Catalog, metadata, cross-compilation, and synthetic fixtures are not support evidence. A candidate target remains blocked until the final public package executes on its exact native OS and architecture without emulation. The launcher has no scoring authority, network/provider behavior, install hook, PATH fallback, runtime dependency, or verification bypass.

Checked-in templates remain private. The Darwin/GNU evidence boundary executes offline extracted private packages on exact native ARM64/x64 macOS and GNU Linux hosts, then writes bounded external `career.npm_native_inspection.v1` evidence for source/catalog/host/version/header/linkage/import/interpreter/GLIBC-symbol/mode/size/SHA-256 checks. The musl boundary uses a digest-pinned Node 22.19.0 Alpine 3.22 image on architecture-matched native x64/ARM64 hosted runners and additionally proves exact musl 1.2.5 plus target-specific static linkage: x64 ELF `DYN` static PIE with `NOW PIE` flags, AArch64 ELF `EXEC`, and neither with an interpreter or shared-library imports. The Windows boundary uses native `windows-2025` x64 and `windows-11-arm` ARM64 runners, a Windows-specific private staging/parity path, and bounded pure-Python PE32+/machine/import inspection; it requires `career.exe` as a regular non-symlink file and records no Unix runtime mode. The preparation-only workflow uploads and publishes nothing. The `0.1.1` external candidate boundary requires clean annotated `v0.1.1` at exact fetched `origin/main`, reviewed SHA, exact Node/npm and Rust toolchains, approved native runners, strict tar/file/mode/source-byte allowlists, ordered eight-native-before-launcher assembly, and registry-free adversarial verification. Protected `npm-publish-v0.1.1.yml` repeats native execution/inspection from final source, publishes through `npm-production`, and requires no-secret public acceptance on all eight targets. npm registry provenance, package-contained SHA-256 consistency, and the absent independent native-binary signature remain distinct.

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

Optional external analysis suggestions use a separate review-only boundary:

```text
career.resume_input.v1
        │
        ▼
resume_analysis_v1 deterministic baseline
        │ host/provider proposes bounded source-targeted suggestions
        ▼
career-core reruns analysis + validates current action/source occurrence
        │
        ├─ unchanged authoritative analysis baseline
        └─ separately labeled assisted review suggestions
```

Exact target/evidence occurrence does not prove semantic entailment, factual truth, or rewrite safety. This boundary never produces a candidate resume, selection, source mutation, export, or materialization result.

Optional exact analysis replacements use a separate review-only boundary:

```text
career.resume_input.v1
        │
        ▼
resume_analysis_v1 deterministic baseline
        │ host/provider proposes exact source-targeted replacements
        ▼
career-core reruns analysis + validates current action/source occurrence
        │
        ├─ unchanged authoritative analysis baseline
        └─ canonical assisted before/proposed-after diff values
```

The replacement boundary is distinct from advisory analysis suggestions. It emits no candidate resume, selection, application, materialization, export, persistence, or source mutation. Exact target/evidence occurrence does not prove semantic entailment, factual truth, or rewrite safety.

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
- rerun analysis and validate bounded external suggestions against current failed canonical actions and exact source occurrence without changing the baseline
- rerun analysis and validate bounded exact external replacements against current failed canonical actions and exact source occurrence for non-authoritative diff display only
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
- The npm launcher must not add, remove, or rewrite native stdout/stderr bytes; packaged/native output remains byte-equivalent.
- Successful CLI machine JSON is completely serialized before a 33,554,432-byte (32 MiB) ceiling check and any stdout write; it is never silently truncated.
- Schema bundles retain the requested embedded root and rewrite only recursively known sibling references to root-local JSON Pointers; unknown, remote, or unresolved references fail closed.
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
