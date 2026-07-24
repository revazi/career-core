# Coding-agent usage

All coding-agent harnesses should invoke the same local `career` executable. Agents must not reproduce scoring in prompts or assume unavailable behavior.

## Safe invocation sequence

Start with discovery:

```bash
career capabilities --format json-compact
career schema list --format json-compact
```

Invoke only capabilities whose status is `available`. Use an explicit input file and capture stdout separately from stderr:

```bash
input_path="/absolute/path/to/job-match-input.json"
output_path="/absolute/path/to/job-match-result.json"
error_path="/absolute/path/to/job-match-error.json"

if career job match \
  --input "$input_path" \
  --format json-compact \
  >"$output_path" 2>"$error_path"
then
  # Parse exactly one JSON document from "$output_path".
  :
else
  # Parse career.error.v1 from "$error_path" and inspect the exit status.
  :
fi
```

Quote every path. Do not concatenate document contents into a shell command, use `eval`, or place resume text in command-line arguments. For stdin, pass bytes directly:

```bash
career resume analyze --input - --format json-compact <"$input_path"
```

When a temporary file is unavoidable, restrict its permissions and remove it:

```bash
umask 077
temporary_input="$(mktemp)"
trap 'rm -f "$temporary_input"' EXIT
# Write one versioned JSON input document to "$temporary_input" without logging it.
career resume normalize --input "$temporary_input" --format json-compact
```

## Pi

The repository includes `.agents/skills/career-core/SKILL.md`. After trusting the checkout and reloading skills, Pi should follow that guide and invoke either an installed `career` binary or:

```bash
cargo run --quiet --locked -p career-cli -- capabilities --format json-compact
```

The Cargo `--quiet` flag keeps build status off stdout; `--locked` preserves the reviewed dependency graph.

## Claude Code and Codex

Grant the agent access only to the local executable and the specific input paths required for the task. In project instructions, require it to:

1. run `career capabilities`
2. export schemas when it needs an exact contract
3. use `json-compact` for machine parsing
4. preserve evidence, uncertainty, and warnings
5. request approval before sending any source content to an external service

No provider-specific extension or duplicate TypeScript/Python scoring implementation is needed.

## Generic subprocess clients

Pass an argument vector rather than constructing a shell string. For example:

```text
["career", "job", "match", "--input", input_path, "--format", "json-compact"]
```

Read stdout and stderr independently, enforce a caller-owned timeout, and branch on the documented exit status. Do not retry typed validation failures without changing the input.

## Interpretation rules

- JSON, not text output, is authoritative for agent decisions.
- `resume evaluate` is section coverage only; use `resume analyze` for full deterministic readiness checks.
- Assisted resume values never replace the deterministic baseline or enter authoritative scores.
- `not_detected`, `inconclusive`, `provisional`, and `unverified` do not prove absence.
- Match only skills returned as `normalized_exact` or `conservative_alias`.
- Recommendation labels are workflow guidance, not proprietary ATS rankings or hiring predictions.
- Preserve source spans, basis IDs, evidence, confidence, and warnings in explanations.

See [`cli.md`](cli.md) for the complete process contract and [`.agents/skills/career-core/SKILL.md`](../.agents/skills/career-core/SKILL.md) for operation-specific guidance.
