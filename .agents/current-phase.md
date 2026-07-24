# Current phase

## Most recently completed phase

**Phase 5 — Agent-ready CLI and distribution**

## Status

Complete. PR review and merge are pending. Phase 4B was merged through PR `#6` as `2794884`. Phase 6 Swift bindings and Phase 7 optional adapters have not started.

## Approved scope

- preserve the existing command hierarchy and operation semantics
- explicit pretty JSON, compact JSON, and human-readable output modes
- embedded offline schema discovery and export commands
- stable stdout, stderr, input, and exit-code documentation
- source-checkout and local `cargo install --path` instructions
- shell-safe invocation guidance for Pi, Claude Code, Codex, and generic agents
- Linux and macOS CLI compatibility verification
- reproducible release-build and checksum checklist for future approved binaries
- Agent Skill and public documentation updates
- additive CLI integration tests and CI checks

## Explicitly out of scope

- changing deterministic normalization, evaluation, analysis, or matching behavior
- changing existing JSON result schemas or canonical default JSON output
- publishing crates, creating a GitHub release, or uploading binaries without separate explicit approval
- package-manager distribution
- remote execution, telemetry, URL fetching, providers, prompts, or model calls
- MCP, editor extensions, or provider-specific wrappers
- persistence, databases, accounts, or application tracking
- Swift bindings, XCFrameworks, Swift packages, or SwiftUI

## Compatibility requirements

- existing `--format json` output remains the canonical pretty JSON used by reviewed goldens
- all existing commands, flags, exit statuses, schemas, and byte-equivalent goldens remain valid
- compact output is additive and contains exactly one JSON document on stdout
- schema export is embedded in the installed binary and performs no filesystem or network discovery
- machine modes write results only to stdout and bounded typed errors only to stderr
- source payloads and supplied paths remain absent from diagnostics

## Acceptance gate

- an agent can discover capabilities and available schemas without reading Rust internals
- every operation accepts explicit files and stdin as already documented
- every document/capability operation supports canonical pretty JSON, compact JSON, and text output; schema export remains JSON-only
- schema catalog and exported schemas are valid Draft 2020-12 JSON
- local source installation is documented and verified
- Linux and macOS CI exercise the CLI process contract
- release preparation is reproducible and checksum-based without publishing an artifact
- Agent Skill and examples cover all available operations and preserve uncertainty rules
- formatting, Clippy, tests, locked build, rustdoc, schemas, prior goldens, clean clone, security checks, and GitHub Actions pass

## Implemented

- Added explicit `json-pretty` and one-line `json-compact` modes without changing default canonical `json` bytes.
- Added `career schema list` with `career.schema_catalog.v1` JSON/text output.
- Added `career schema export --id <contract-id>` with exact canonical or compact embedded schema output.
- Embedded all 14 reviewed Draft 2020-12 schemas without runtime generation, path lookup, or network access.
- Added process tests covering every operation in compact mode, exact schema bytes, schema IDs, pretty compatibility, machine-clean errors, and invalid schema IDs.
- Added source installation, CLI contract, shell-safe agent, distribution, checksum, and release-preparation documentation.
- Updated the Agent Skill, handbook, changelog, security policy, contribution guidance, and PR checklist.
- Expanded CI to Linux and macOS, source installation, installed-binary schema export, schema catalog checks, and all prior goldens.
- Added no dependencies and changed no deterministic core algorithm or existing result schema.

## Verification

- formatting and Clippy pass with warnings denied
- 107 Rust tests pass with all features and the lockfile
- locked workspace build and rustdoc with warnings denied pass
- every prior canonical CLI golden remains byte-equivalent
- every machine operation emits independently parsed one-line compact JSON
- all 14 exported canonical schemas are byte-equivalent to reviewed repository files
- all schemas and `career.schema_catalog.v1` pass Draft 2020-12 validation
- local and clean-clone `cargo install --path crates/career-cli --locked` succeed; installed binaries run capability and schema discovery outside the checkout
- CLI help, JSON/text schema catalog, compact errors, input limits, and exit statuses pass
- CI YAML and relative Markdown links parse; credential and line-ending scans pass
- a local native release-preparation smoke builds, stages licenses/docs, executes the binary, creates an archive, and verifies its SHA-256 manifest without publishing it
- clean-clone formatting, Clippy, 107 tests, locked build, rustdoc, installed-binary checks, all prior goldens, and canonical schema export pass
- Fallow changed-code and security checks report no findings; Fallow does not analyze Rust source
- `git diff --check` passes
- GitHub Actions run `30076241222` passes on Linux and macOS, including Rust 1.85, 107 tests per platform, source installation, installed-binary schema export, compact output, and every canonical golden

## Deferred gate

Actual release creation, binary upload, package-manager distribution, Swift bindings, and MCP each require separate approval after this phase's review and merge. Phase 6 may begin only after Phase 5 is reviewed, merged, synchronized to local `main`, and separately approved.
