# Accepted decisions

These decisions remain active until an explicit reviewed change updates this file.

## D-001 — Rust core

The authoritative portable engine is written in Rust. Python and Go are not embedded in Apple applications. SwiftUI remains a future consumer rather than the source of scoring truth.

## D-002 — Dual permissive license

Source is offered under `MIT OR Apache-2.0`. Package manifests and source distributions must preserve that expression and both license files.

## D-003 — CLI before agent-specific protocols

A documented JSON CLI is the universal coding-agent interface. Agent Skills documentation teaches agents to call it. MCP, editor extensions, and provider-specific integrations are deferred until a concrete consumer cannot use the CLI.

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

Phase 4A exposes bounded `career.job_input.v1`, `career.job_normalization.v1`, and `career job normalize` without matching, URL fetching, or provider fallback. Phase 4B may consume the reviewed deterministic job-normalization contract only after 4A merges. `job.match` remains planned until its independent scoring, equivalence, evidence, uncertainty, and recommendation gates pass.
