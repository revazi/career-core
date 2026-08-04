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

The repository is a reviewed local Pi package with one canonical skill at `.agents/skills/career-core/SKILL.md` and three native tools:

- `career_core_discover`
- `career_core_resume`
- `career_core_job`

Install the CLI first, then register the checkout without piping remote code into a shell:

```bash
cargo install --path crates/career-cli --locked
repository_root="$(pwd -P)"
pi install "$repository_root"
```

The native extension invokes installed `career` directly with argv, passes document JSON only through stdin, and never invokes Cargo, a shell, a model/provider, the network, or a temporary career payload/result file. Use `career_core_discover` before document operations. Document tools accept one exact versioned object serialized as `input_json` and return one complete compact CLI JSON result.

Pi session history is separate from Career Core's no-persistence property. Pi may save tool arguments and results in its session JSONL. Before private document use, require an explicit user decision and recommend a new transient run:

```bash
pi --no-session
```

Do not claim secure erasure. Local Pi-session approval also does not authorize sending content to an external provider. If a complete tool result exceeds the 50,000-byte / 2,000-line context bound, do not accept truncation; use the installed CLI directly in a separately user-approved local workflow capable of consuming the full result.

When native tools are not installed, Pi may invoke the CLI directly or, only from a source checkout at the user's request:

```bash
cargo run --quiet --locked -p career-cli -- capabilities --format json-compact
```

The extension itself never selects this Cargo fallback. MCP and an MCP bridge are non-scope.

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
- `resume analysis-suggestions-review` reruns that baseline; retain only its canonical assisted suggestions, action status, discard codes, and mandatory factuality warning. Its v1 `suggestion` remains advisory text and never becomes a replacement.
- `resume analysis-replacements-review` returns exact canonical before/proposed-after values for non-authoritative diff display. It never creates, selects, applies, or materializes a rewrite; retain the baseline, action/status, evidence, and warnings.
- Assisted resume values never replace the deterministic baseline or enter authoritative scores.
- `not_detected`, `inconclusive`, `provisional`, and `unverified` do not prove absence.
- Match only skills returned as `normalized_exact` or `conservative_alias`.
- Recommendation labels are workflow guidance, not proprietary ATS rankings or hiring predictions.
- Preserve source spans, basis IDs, evidence, confidence, and warnings in explanations.

See [`cli.md`](cli.md) for the complete process contract and [`.agents/skills/career-core/SKILL.md`](../.agents/skills/career-core/SKILL.md) for operation-specific guidance.
