# Coding-agent integration

## Universal interface

The `career` executable is the primary agent interface. A CLI is portable across coding-agent harnesses, inspectable, scriptable, and does not require each harness to implement a protocol client.

Harness-specific adapters and bundled skills are maintained outside this repository. The optional native Pi integration lives in [`pi-career`](https://github.com/revazi/pi-career). Phase 9 prepares exact `@revazi/career@0.1.1` distribution around the same CLI; internal optional native packages are launcher-owned implementation details and do not move `pi-career` runtime ownership or Core authority.

The installed CLI embeds reviewed Draft 2020-12 schemas. Agents can discover exact contracts without a source checkout or network request:

```bash
career operations --format json-compact
career schema list --format json-compact
career schema export --id career.job_match_input.v1
career schema bundle --id career.job_match_input.v1 --format json-compact
```

## Discovery first

Generic raw agents must begin with:

```bash
career capabilities
career operations --format json-compact
```

Inside a source checkout:

```bash
cargo run --quiet -p career-cli -- capabilities
```

JSON is the default. Only capabilities with `status: "available"` may be invoked. Planned entries are roadmap visibility, not callable behavior. `career.operation_catalog.v1` maps each available capability exactly once to a stable operation descriptor and separately catalogs `core.operations`, `schema.list`, `schema.export`, and `schema.bundle` as bootstrap operations with null capability IDs.

## Process contract

All machine commands follow these rules as they are introduced:

- One JSON result goes to stdout; `json-compact` keeps it on one line.
- Every successful machine result, including its trailing newline, is at most 33,554,432 bytes (32 MiB) and is completely serialized/bound-checked before stdout is written.
- `json` is compatibility-preserving canonical pretty output; `json-pretty` is explicit pretty output; `text` is human-only.
- Diagnostics go to stderr.
- Success exits `0`.
- CLI usage exits `2`, input I/O/byte-limit failures exit `3`, invalid JSON exits `4`, core validation exits `5`, and output failures exit `6`.
- No progress indicators or ANSI escapes appear in machine modes.
- Input from `-` means stdin where documented.
- Commands never make implicit network requests.
- Commands never modify input files unless a future explicit mutation command is reviewed.
- Source payloads are not repeated in diagnostics.

## Managed-adapter discovery

A separately reviewed managed adapter may invoke and validate `career operations` and required `career schema bundle` documents internally once per verified `core_version`. It may cache only this non-sensitive metadata so normal model-visible workflows do not need capability/schema bootstrap calls or copied schemas. The complete Core result must still be captured within the declared ceiling before projection. This permission does not authorize private document/result caching, handles, registries, persistence, repair, retries, authority upgrades, model/provider behavior, networking, or UI in Career Core.

Schema bundles retain the requested embedded root and recursively place dependencies under the reserved `careerSchemaBundle` definition. Every `$ref` is a root-local JSON Pointer. Unknown sibling files, remote refs, unsupported fragments, and reserved-key collisions fail closed without filesystem or network lookup.

## npm launcher contract

The accepted Node 22+ `@revazi/career@0.1.1` launcher exposes the same `career` bin for six active macOS/Linux target classes. It resolves only its package-local optional implementation, positively identifies Linux libc, requires glibc 2.35 or newer for GNU packages, verifies strict versioned provenance and binary type/file-invariant/size/target/SHA-256 consistency, and preserves argv/stdin/stdout/stderr/exit/signal behavior. It has no PATH fallback, runtime download, lifecycle script, provider/network behavior, telemetry, or bypass. Package-contained SHA-256 is not a signature.

Agents and pi-career may use exact `0.1.1` on macOS and Linux. The root crate rejects native Windows compilation; Windows hosts must build and run under WSL, where Rust and Node target Linux and Node reports `process.platform === "linux"`. Protected run `31346152236` historically published nine packages and passed eight native acceptance jobs, including now-retired Windows artifacts. Catalog data, private templates, cross-compilation, emulation, skipped jobs, and that historical Windows evidence are never support evidence for a later version.

After that gate, agents and managed consumers address only:

```bash
npx --yes --package=@revazi/career@0.1.1 career <args>
```

Use exact `0.1.1`, never `latest` or a range. Never install or resolve a native implementation package directly. npx acquisition is outside launcher runtime. The separate [`../docs/pi-career-npm-handoff.md`](../docs/pi-career-npm-handoff.md) defines the consumer transition and bundled-runtime removal gates.

## Agent safety rules

An integrating agent must:

1. inspect capabilities rather than assume commands
2. use JSON output for decisions
3. treat evidence and warnings as authoritative context
4. avoid upgrading uncertain findings into facts
5. never claim that `career-core` generated content it only evaluated
6. obtain user approval before sending source documents to any external model or service
7. avoid writing generated claims back into a resume without source evidence
8. preserve the deterministic baseline and label every accepted external field as assisted

## Available Phase 1 operation

```bash
career resume evaluate --input <path|-> [--format json|json-pretty|json-compact|text]
```

The input and result contracts are `career.resume_input.v1` and `career.resume_evaluation.v1`. This operation evaluates core-section header coverage only. Agent explanations must retain the limited-scope warning and must not label this score as complete resume quality or ATS compatibility.

## Available Phase 3 operation

```bash
career resume analyze --input <path|-> [--format json|json-pretty|json-compact|text]
```

Input is `career.resume_input.v1`; output is `career.resume_analysis.v1`. This is the full deterministic resume-readiness operation. It always scores `resume_normalization_v1` deterministic baseline facts and cannot consume an assisted document.

Agents must:

- distinguish `raw_score` from confidence-adjusted `score`
- describe `inconclusive` checks and `provisional` findings as unverified
- cite `basis_check_id` and bounded evidence for actions or explanations
- preserve the general-ATS and visual-layout warnings
- never describe the result as a proprietary ATS ranking or a hiring-outcome prediction

`resume evaluate` remains the unchanged Phase 1 section-coverage operation; do not present it as an alias for `resume analyze`.

## Available review-only analysis-suggestion operation

```bash
career resume analysis-suggestions-review --input <path|-> [--format json|json-pretty|json-compact|text]
```

Input is `career.resume_analysis_suggestion_review_input.v1`; output is `career.resume_analysis_suggestion_review.v1`. The operation independently reruns the current deterministic analysis from the original input and preserves it under `baseline_analysis` without changing any score, check, evidence, warning, or action.

Agents may submit at most three untrusted source-targeted suggestions. Each retained suggestion is bound to one current failed canonical improvement action, receives a core identifier, and inherits the action's confirmed/provisional status. Agents must use bounded discard codes rather than repairing rejected suggestions. Exact target/evidence occurrence is not factual entailment or rewrite certification. Retained suggestions are assisted/non-authoritative, require human review, and must not become a candidate resume, selection, source mutation, export, materialization, analysis input, or matching input. The v1 `suggestion` field is advisory text, not a replacement; hosts must not relabel it or synthesize replacement semantics.

## Available review-only analysis-replacement operation

```bash
career resume analysis-replacements-review --input <path|-> [--format json|json-pretty|json-compact|text]
```

Input is `career.resume_analysis_replacement_review_input.v1`; output is `career.resume_analysis_replacement_review.v1`. The operation reruns and preserves the complete deterministic analysis under `baseline_analysis` and retains at most three exact source-targeted `proposed_replacement` values bound to current failed canonical actions.

A host may render a non-authoritative before/proposed-after diff only from each returned canonical `source_target`, `proposed_replacement`, line range, action/status, evidence, and warnings. It must use bounded discard codes rather than repair rejected items. Exact occurrence is structural grounding, not factual or rewrite certification. This contract never creates, selects, applies, materializes, exports, persists, or mutates a resume, and cannot enter analysis or matching.

## Available Phase 2 operations

```bash
career resume normalize --input <path|-> [--format json|json-pretty|json-compact|text]
career resume enrich --input <path|-> [--format json|json-pretty|json-compact|text]
```

`resume normalize` returns `career.resume_normalization.v1`, including deterministic facts, source spans, confidence, field statuses, warnings, and an `enrichment_request`.

An agent may attempt external enrichment only when all of these are true:

1. `enrichment_request.status` is `eligible`
2. the user has approved sending the bounded resume context to the current model/provider
3. the agent can produce exact `career.resume_enrichment_proposal.v1` JSON

The agent then submits one `career.resume_enrichment_input.v1` envelope to `resume enrich`. It must copy values from source text, populate only `target_sections`, and leave ambiguous targets empty. Invalid proposals are not repaired by guesswork.

`resume enrich` performs no model call. It returns the unchanged deterministic normalization under `baseline` and accepted values under `assisted_document`. Authoritative scores must consume the baseline. `resume analyze` enforces this by accepting only the original resume input and independently rerunning deterministic normalization. If the external model or validation fails, the agent must continue with the deterministic normalization rather than treating the operation as failed.

## Available Phase 7 operations

Review a bounded external proposal without invoking a provider:

```bash
career resume variant-review --input <path|-> [--format json|json-pretty|json-compact|text]
```

Input is `career.resume_variant_review_input.v1`; output is `career.resume_variant_review.v1`. The operation retains at most 50 canonical non-overlapping changes with core-assigned identifiers and may discard invalid changes using bounded codes. Exact evidence occurrence proves only occurrence, not factual entailment. Agents must preserve all non-authoritative warnings and must not present retained generated wording as core-certified fact.

After the user explicitly selects canonical identifiers, materialize only that subset:

```bash
career resume variant-materialize --input <path|-> [--format json|json-pretty|json-compact|text]
```

Input is `career.resume_variant_materialization_input.v1`; output is `career.resume_variant.v1`. Materialization revalidates the complete proposal and selection, preserves the exact baseline, and changes only selected targets. The assisted result cannot enter `resume analyze` or `job match`. Agents must not repair discarded changes, select changes automatically, or overwrite the original resume.

## Available Phase 4 operations

```bash
career job normalize --input <path|-> [--format json|json-pretty|json-compact|text]
```

Input is `career.job_input.v1`; output is `career.job_normalization.v1`. The command accepts plain text only and never fetches the source URL.

Agents must:

- treat `required` and `preferred` classifications as deterministic lexical extraction, not semantic certainty
- preserve parse-confidence and unclassified-line warnings
- treat every `not_detected` field as unverified rather than confirmed absent
- cite source spans when presenting extracted requirements

Provider fallback and external-proposal job enrichment are not part of Phase 4.

Match one original resume input and one original job input:

```bash
career job match --input <path|-> [--format json|json-pretty|json-compact|text]
```

Input is `career.job_match_input.v1`; output is `career.job_match.v1`. Matching independently reruns both deterministic normalizers and cannot consume assisted documents.

Agents must:

- distinguish each category's `raw_score` from confidence-bounded `score`
- treat `partial`, `likely_missing`, and especially `unverified` as different evidence states
- cite job/resume source spans when present and retain null spans for derived domain/keyword signals
- accept only `normalized_exact` or `conservative_alias` skill matches returned by the core
- never substitute adjacent technologies or repair a conservative false negative with guesswork
- preserve mandatory scope warnings and provisional recommendation status
- treat `apply_now`, `apply_after_small_edits`, and `improve_first` as bounded workflow guidance, never a hiring prediction
- review `unassessed_required_qualifications` rather than converting them into inferred gaps

Low/unknown normalization or truncation bounds all category scores to 50–75, marks missing/partial evidence unverified, suppresses broad inferred top gaps, and prevents `apply_now`. Provider interpretation, URL fetching, company research, persistence, and automatic resume rewriting remain outside this command.

## Distribution stages

1. source checkout via `cargo run --locked`
2. local CLI install via `cargo install --path crates/career-cli --locked`
3. private external npm staging/testing through `scripts/prepare-npm-cli-packages.sh`
4. exact clean/tagged npm candidate verification through `scripts/test-npm-publication.sh`
5. future protected npm publication only through stable OIDC-only `npm-release.yml` from an annotated stable-SemVer tag, followed by all six active public-registry acceptance jobs
6. pi-career migration only after its separate handoff gates
7. every other package-manager or binary channel only after separate approval

Source installation, future checksum verification, and the release-preparation gate are documented under `docs/`. Do not tell users to pipe remote scripts into a shell.

## Future MCP decision gate

Consider MCP-over-stdio only if a concrete consumer requires structured tool discovery that cannot use capability JSON and subprocess execution. An MCP adapter must remain thin, local by default, and depend on `career-core`. It must not add LLM behavior.

## External coding-agent packages

Career Core does not ship harness-specific packages or skills. Its user-facing npm package is only a generic launcher for the unchanged CLI. External integrations must treat `@revazi/career` and the installed CLI/embedded schemas as authoritative rather than addressing internal native packages or duplicating scoring, schemas, or repair behavior. Each future integration requires its own ownership and review.
