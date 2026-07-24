# Coding-agent integration

## Universal interface

The `career` executable is the primary agent interface. A CLI is portable across coding-agent harnesses, inspectable, scriptable, and does not require each harness to implement a protocol client.

Pi specifically favors well-documented CLI tools and Agent Skills. MCP is not required for the initial integration.

The installed CLI embeds reviewed Draft 2020-12 schemas. Agents can discover exact contracts without a source checkout or network request:

```bash
career schema list --format json-compact
career schema export --id career.job_match_input.v1
```

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

- One JSON result goes to stdout; `json-compact` keeps it on one line.
- `json` is compatibility-preserving canonical pretty output; `json-pretty` is explicit pretty output; `text` is human-only.
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
2. local install via `cargo install --path crates/career-cli --locked`
3. checksummed release binaries only after explicit release approval
4. package-manager distribution only with a maintenance plan

Source installation, future checksum verification, and the release-preparation gate are documented under `docs/`. Do not tell users to pipe remote scripts into a shell.

## Future MCP decision gate

Consider MCP-over-stdio only if a concrete consumer requires structured tool discovery that cannot use capability JSON and subprocess execution. An MCP adapter must remain thin, local by default, and depend on `career-core`. It must not add LLM behavior.

## Future coding-agent packages

Provider-specific extensions may wrap the same CLI, but the repository should not duplicate scoring in TypeScript, Python, prompts, or skills. Skills teach invocation; Rust remains authoritative.
