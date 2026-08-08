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

## D-019 — Native runtime archives are maintainer handoff inputs, not releases

Career Core may build the existing locked release CLI on approved matching native runners, execute synthetic acceptance, record bounded provenance/digests, and upload a short-retention unsigned archive for reviewed import into `pi-career`. This exception does not authorize a tag, GitHub Release, public download/install channel, package publication, signing/notarization, install hook, or unsupported architecture claim. Normal Core CI remains separate and uploads no runtime artifact.

## D-020 — Managed adapters discover exact CLI contracts without changing Core authority

The additive `career.operation_catalog.v1` is separate from byte-stable `career.capabilities.v1`. It catalogs all callable machine operations: each available capability exactly once plus operation/schema bootstrap commands with null capability mappings. It declares exact transports, schemas, input bounds, and one 33,554,432-byte (32 MiB) successful machine-output ceiling. `career schema bundle` recursively rewrites only embedded sibling refs into one self-contained Draft 2020-12 root and rejects unknown/remote refs. Machine JSON is serialized and bound-checked before stdout. Generic agents remain discovery-first; a reviewed managed adapter may internally cache only non-sensitive discovery per exact `core_version`. Core still owns no handles, state, projection, persistence, provider/model, networking, UI, or managed runtime behavior.

## D-021 — npm distribution is one public launcher over internal native packages

The only npm consumer surface is exact Node 22+ `@revazi/career`, bin `career`. Apple Silicon macOS and x86-64 GNU/Linux packages remain exact-version internal optional-dependency implementation details; users and `pi-career` never address them directly. The launcher resolves only the selected package-local manifest, requires glibc 2.35 or newer on Linux, rejects older/malformed/musl/unknown libc and every unapproved target, validates strict package/provenance identity plus regular non-symlink exact-mode/size/type/SHA-256 binary consistency, then transparently spawns the native CLI with argv and inherited stdio. It has no network/download, lifecycle script, PATH fallback, telemetry/provider behavior, output embellishment, or bypass.

Package-contained SHA-256 consistency, npm registry integrity/SLSA provenance, and the absent independent native-binary signature are distinct. Checked-in templates remain `private: true`; only exact external candidate staging may remove guards.

## D-022 — npm publication is exact, ordered, environment-gated, and non-fallback

Initial release is exact `0.1.0` at annotated `v0.1.0`. Candidate source must be clean reviewed `origin/main`; release builds pin rustc/Cargo 1.97.1, Node 22.19.0/npm 11.6.2, `macos-14` ARM64, and `ubuntu-22.04` x64/glibc 2.35. Tarballs and metadata are strict external allowlists; native packages precede the launcher.

The sole publish workflow is manual and protected by `npm-production`. Explicit `bootstrap=true` uses one temporary granular token for first publication or exact interrupted recovery. Default `bootstrap=false` forbids tokens and uses package-level GitHub OIDC trusted publishing. Neither mode falls back. Exact registry integrity and SLSA provenance are mandatory, conflicts fail, and completion requires no-secret public acquisition/parity on both hosts. The bootstrap secret/token is deleted/revoked immediately, then all package trusted publishers are configured and verified interactively with account 2FA. Only afterward does the parent maintainer create the annotated `v0.1.0` GitHub Release with dated notes and no custom assets. The transitional `pi-career` artifact path remains until the external consumer migration gate passes.
