# Phase 8A: installed CLI and native Pi integration

## Decision and scope

The demonstrated consumer is Pi. Phase 8A adds a local package that teaches Pi through the existing canonical Agent Skill and registers three collision-safe tools:

- `career_core_discover` for capabilities and embedded schema discovery
- `career_core_resume` for the eight available resume operations
- `career_core_job` for job normalization and matching

The extension is a transport adapter around an already reviewed installed `career` executable. Rust Core, CLI behavior, public JSON schemas, scoring, evidence, ordering, uncertainty, and assisted/non-authoritative labels remain authoritative and unchanged. The package has no runtime dependency beyond Pi-provided peer packages and Node built-ins. MCP, an MCP bridge, provider/model calls, UI, persistence, publication, and automatic Cargo invocation are non-scope.

The repository root is the local Pi package. Its manifest points to `extensions/career-core/index.ts` and the canonical `.agents/skills/career-core` directory; the skill is not copied.

## Process boundary

The extension resolves `career` from `PATH`, unless `CAREER_CLI_PATH` contains a bounded absolute executable path. It invokes the executable directly with an argv array and never constructs a shell command. Document operations always use:

```text
career <group> <operation> --input - --format json-compact
```

Input JSON is accepted as one bounded tool string, parsed as one object before spawn, and sent only to child stdin. Discovery operations have no document input. The extension never invokes Cargo, opens a career payload file, creates a temporary payload/result file, or makes a network/provider request.

Every child has a fixed timeout. Pi's abort signal triggers termination, and termination escalates from `SIGTERM` to `SIGKILL` after a short fixed grace period if necessary. Calls share no mutable request state, so parallel executions remain independent and read-only.

Stdout and stderr are captured separately under byte ceilings. A successful process must emit no stderr and exactly one JSON object on stdout. A complete result must also fit Pi's 50 KB / 2,000-line context guidance; an oversized authoritative JSON result fails rather than being truncated or persisted. The user may then run the installed CLI directly in a separately chosen local workflow that can consume the complete result.

Known nonzero CLI failures are exposed only after strict validation of a bounded `career.error.v1` envelope. Missing executables, timeout, cancellation, signals, malformed or multiple JSON, stream overflow, context overflow, unexpected stderr, and unknown process failures map to stable `career.pi_error.v1` messages. Raw stderr, Node errors, executable paths, environment values, inputs, and outputs are never included in errors or logs. Tool `details` contains only the operation identifier and adapter detail schema version.

## Threat model

| Threat | Mitigation |
|---|---|
| Shell injection through document text, schema IDs, or executable paths | Direct `spawn(executable, argv, { shell: false })`; document bytes use stdin only; operation enums and schema IDs are validated before spawn. |
| Oversized input, output, diagnostics, or context exhaustion | Per-operation UTF-8 input limits, strict stdout/stderr capture limits, one-object parsing, and explicit complete-result context failure. |
| Hung or abandoned subprocess | Fixed timeout, abort propagation, signal escalation, and close-before-settle cleanup. |
| Private payload disclosure through diagnostics | No payload logging; raw failures and paths are redacted; only validated bounded CLI errors cross the boundary. |
| Accidental persistence by the adapter | No payload/result filesystem writes and no extension state entries. |
| Mistaking Pi for an ephemeral host | Documentation states that Pi may store tool arguments and results in session JSONL. Private use requires an explicit user decision; `pi --no-session` is recommended for a new transient run. No secure-erasure claim is made. |
| Adapter changing deterministic authority | Raw successful CLI JSON is returned complete without reordering or semantic transformation; TypeScript contains no scoring or repair logic. |
| Assisted text entering authoritative operations | Existing operation-specific Core contracts and skill rules remain unchanged; assisted outputs retain their labels and cannot be substituted into baseline analysis or matching. |
| Hidden network/model/provider execution | Extension code uses only local process and buffer APIs; smoke testing loads resources without creating a model runtime or prompting a model. |

## Compatibility and maintenance

Tool names and operation enum values are adapter contracts and should remain stable within this package's major version. Adding a CLI operation requires capability/schema review, an explicit operation mapping and byte limit, tests, and skill/documentation updates. New `career.error.v1` codes are not exposed until reviewed and added to the extension allowlist; an unknown error remains redacted.
