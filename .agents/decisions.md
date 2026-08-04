# Accepted decisions

These decisions remain active until an explicit reviewed change updates this file.

## D-001 — Rust core

The authoritative portable engine is written in Rust. Python and Go are not embedded in Apple applications. SwiftUI remains a future consumer rather than the source of scoring truth.

## D-002 — Dual permissive license

Source is offered under `MIT OR Apache-2.0`. Package manifests and source distributions must preserve that expression and both license files.

## D-003 — CLI before agent-specific protocols

A documented JSON CLI is the universal coding-agent interface. Coding-agent documentation teaches agents to call it. A demonstrated harness may receive an explicitly approved thin native tool adapter only after the CLI contract is stable and without duplicating authority. Harness-specific packages belong outside this repository unless a separate reviewed decision changes that boundary. MCP, bridge protocols, editor integrations, and provider-specific behavior remain deferred until their own concrete consumer proves the CLI or an approved native adapter insufficient.

## D-004 — Deterministic core, provider-neutral assisted boundary

The core has no LLM client, API-key handling, provider dependency, hidden prompt, or network behavior. It may emit a bounded enrichment request and validate an explicit external proposal as untrusted data. A host may interpret core evidence or submit source-grounded assistance, but assisted fields cannot silently replace the deterministic baseline, authoritative scores, confidence, or evidence.

## D-005 — Django is reference, not dependency

`../resume-ai` provides source-grounded rules and regression examples. The Rust build and tests must stand alone.

## D-006 — Root package owns heavy lifting

The repository root package is `career-core`. Adapter packages live in the Cargo workspace and depend inward. CLI libraries never leak into the root package.

## D-007 — Capability discovery is truthful

Operations are advertised as `available` only after their contracts, implementation, limits, schemas, and tests satisfy the active phase. Planned entries may be discoverable but must not be callable.

## D-008 — Swift comes after stable core operations

Do not add UniFFI, XCFramework packaging, Swift code, or Apple build automation before deterministic resume and matching contracts are stable enough to justify the boundary.

## D-009 — No publishing without explicit approval

Creating GitHub releases, publishing crates, uploading binaries, and changing repository visibility require explicit maintainer approval.

## D-010 — Hosts orchestrate optional external enrichment

An agent or application may automatically attempt external normalization enrichment only after the user enables it, an eligible core request exists, and a provider is configured. The host owns consent, keys, prompts, provider calls, timeouts, and failure isolation. The core owns target selection, proposal contracts, source-grounding validation, conservative empty-field merging, provenance, and deterministic-score isolation. Provider failure always leaves the local deterministic result usable.

## D-011 — Full analysis is additive and baseline-only

`career.resume_evaluation.v1` and `career resume evaluate` retain their Phase 1 section-coverage semantics. Full deterministic readiness scoring is exposed separately as `career.resume_analysis.v1` and `career resume analyze`. Analysis reruns `resume_normalization_v1` from the original input and cannot accept assisted documents, so external proposals never alter authoritative checks, confidence, evidence, or scores.

## D-012 — Phase 4 stabilizes job normalization before matching

Phase 4A exposed bounded `career.job_input.v1`, `career.job_normalization.v1`, and `career job normalize` without matching, URL fetching, or provider fallback. Phase 4B consumed the reviewed deterministic job-normalization contract only after 4A merged. `job.match` remained planned until its independent scoring, equivalence, evidence, uncertainty, and recommendation gates passed.

## D-013 — Matching reruns deterministic baselines and gates recommendations

`career job match` accepts original `career.resume_input.v1` and `career.job_input.v1` documents inside `career.job_match_input.v1`. The core reruns both deterministic normalizers, so assisted or caller-forged normalized values cannot enter authoritative matching. Skill matches use normalized equality or the reviewed same-technology alias policy only. Low/unknown confidence or truncation bounds category scores and makes gaps provisional. Recommendation labels are generated from deterministic score, core-category, explicit-gap, and unassessed-qualification gates and cannot be upgraded by external prose.

## D-014 — CLI schemas are reviewed, embedded, and offline

The CLI embeds the repository's reviewed Draft 2020-12 schema files and exposes additive `schema list` and `schema export` commands. It does not infer schemas from Clap or Rust types at runtime and does not search the filesystem or network. Existing `--format json` remains canonical pretty output; `json-pretty` and one-line `json-compact` are additive spellings. Publishing binaries or packages remains a separate approval gate.

## D-015 — Swift uses a narrow pinned UniFFI JSON facade

`career-swift` depends inward on `career-core` and exposes each stable operation as owned versioned JSON input/output plus typed Swift errors. The app may decode those contracts into app-owned `Codable` models; the FFI ABI does not mirror all internal Rust records. Exact UniFFI `0.30.0` is pinned because `0.31+` requires Rust 1.87, above the project MSRV. No handwritten unsafe code is allowed; generated FFI behavior is isolated to the adapter. XCFramework publication, signing, SwiftUI, persistence, and provider behavior remain outside this boundary.

## D-016 — Analysis suggestions are review-only and baseline-bound

`career.resume_analysis_suggestion_review.v1` reruns `career.resume_analysis.v1` from original `career.resume_input.v1` and returns that unchanged result as the authoritative baseline. An external suggestion is retained only when its exact target/evidence occurs in current source and its `basis_check_id` maps to one current failed canonical improvement action; the core assigns identifiers and copies confirmed/provisional status. Exact occurrence does not certify factuality or a rewrite. The contract is intentionally limited to review presentation: it has no candidate resume, selection, source mutation, export, or materialization path. Its `suggestion` field is advisory text, not a replacement. Phase 2 enrichment remains parser-gap recovery only, and Swift only exposes this core-defined JSON surface.

## D-017 — Analysis replacements are explicit diff-only review values

`career.resume_analysis_replacement_review.v1` is a separate additive contract rather than a reinterpretation of advisory v1 suggestions. It reruns and returns the unchanged authoritative `career.resume_analysis.v1` baseline, and retains at most three exact source targets with bounded proposed replacements only when each binds to one current failed canonical action and exact source evidence. The core assigns identifiers/order/action priority/area/status, rejects unchanged targets, and discards every member of duplicate-action or overlapping-target groups. Exact occurrence remains structural grounding, not factuality or rewrite certification. The contract produces no candidate resume, selection, application, materialization, export, persistence, or source mutation; Swift exposes only the same core-owned JSON.

## D-018 — Pi adapter ownership externalized (supersedes repository-local placement)

The earlier decision to host the approved Pi consumer in this repository is superseded. The native Pi package and bundled Agent Skill now belong to the separate [`pi-career`](https://github.com/revazi/pi-career) repository. Career Core ships only its deterministic Rust, CLI/schema, generic installed-CLI acceptance, and Swift surfaces; this ownership change does not alter any of those contracts. Future Core adapters remain separately gated.
