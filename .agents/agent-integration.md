# Coding-agent integration

## Universal interface

The `career` executable is the primary agent interface. A CLI is portable across coding-agent harnesses, inspectable, scriptable, and does not require each harness to implement a protocol client.

Pi specifically favors well-documented CLI tools and Agent Skills. MCP is not required for the initial integration.

## Discovery first

Agents must begin with:

```bash
career capabilities
```

Inside a source checkout:

```bash
cargo run --quiet -p career-cli -- capabilities
```

JSON is the default. Only capabilities with `status: "available"` may be invoked. Planned entries are roadmap visibility, not callable behavior.

## Process contract

All machine commands follow these rules as they are introduced:

- JSON or JSONL results go to stdout.
- Diagnostics go to stderr.
- Success exits `0`.
- CLI usage exits `2`, input I/O/byte-limit failures exit `3`, invalid JSON exits `4`, core validation exits `5`, and output failures exit `6`.
- No progress indicators or ANSI escapes appear in machine modes.
- Input from `-` means stdin where documented.
- Commands never make implicit network requests.
- Commands never modify input files unless a future explicit mutation command is reviewed.
- Source payloads are not repeated in diagnostics.

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

## Agent Skills

The project skill is:

```text
.agents/skills/career-core/SKILL.md
```

It follows the Agent Skills standard so Pi and compatible harnesses can discover it. Keep the skill concise and move detailed command contracts into its `references/` directory. Update the skill whenever an available CLI capability changes.

Pi project skills load only after the repository is trusted. Use `/trust`, restart Pi, and run `/reload` after edits.

## Available Phase 1 operation

```bash
career resume evaluate --input <path|-> [--format json|text]
```

The input and result contracts are `career.resume_input.v1` and `career.resume_evaluation.v1`. This operation evaluates core-section header coverage only. Agent explanations must retain the limited-scope warning and must not label this score as complete resume quality or ATS compatibility.

## Available Phase 2 operations

```bash
career resume normalize --input <path|-> [--format json|text]
career resume enrich --input <path|-> [--format json|text]
```

`resume normalize` returns `career.resume_normalization.v1`, including deterministic facts, source spans, confidence, field statuses, warnings, and an `enrichment_request`.

An agent may attempt external enrichment only when all of these are true:

1. `enrichment_request.status` is `eligible`
2. the user has approved sending the bounded resume context to the current model/provider
3. the agent can produce exact `career.resume_enrichment_proposal.v1` JSON

The agent then submits one `career.resume_enrichment_input.v1` envelope to `resume enrich`. It must copy values from source text, populate only `target_sections`, and leave ambiguous targets empty. Invalid proposals are not repaired by guesswork.

`resume enrich` performs no model call. It returns the unchanged deterministic normalization under `baseline` and accepted values under `assisted_document`. Authoritative scores must consume the baseline. If the external model or validation fails, the agent must continue with the deterministic normalization rather than treating the operation as failed.

## Distribution stages

1. source checkout via `cargo run`
2. local install via `cargo install --path crates/career-cli --locked`
3. checksummed release binaries after release approval
4. package-manager distribution only with a maintenance plan

Do not tell users to pipe remote scripts into a shell.

## Future MCP decision gate

Consider MCP-over-stdio only if a concrete consumer requires structured tool discovery that cannot use capability JSON and subprocess execution. An MCP adapter must remain thin, local by default, and depend on `career-core`. It must not add LLM behavior.

## Future coding-agent packages

Provider-specific extensions may wrap the same CLI, but the repository should not duplicate scoring in TypeScript, Python, prompts, or skills. Skills teach invocation; Rust remains authoritative.
