# Coding-agent usage

All coding-agent harnesses should invoke the same local `career` executable. Agents must not reproduce scoring in prompts or assume unavailable behavior.

## Safe invocation sequence

Start with discovery:

```bash
career capabilities --format json-compact
career operations --format json-compact
career schema list --format json-compact
```

Invoke only capabilities whose status is `available`. Use `career operations` to map those capabilities to exact CLI paths, transports, schemas, and byte bounds. Bootstrap entries with `capability_id: null` describe operation/schema discovery rather than new Core capabilities. Use an explicit input file and capture stdout separately from stderr:

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

## External harness integrations

Career Core ships no harness-specific package or bundled Agent Skill. The optional native Pi integration is maintained in the separate [`pi-career`](https://github.com/revazi/pi-career) repository. Review that package's own installation, privacy, and security guidance there; do not register this Career Core checkout as a Pi package.

Phase 9 adds one generic npm consumer surface for the same CLI:

```bash
npx --yes --package=@revazi/career@0.1.1 career <args>
```

Use exact `0.1.1`; never use `latest` or a range. Active distribution supports macOS and Linux only. The root crate rejects native Windows compilation. On a Windows host, build and run Career Core or pi-career through WSL, where Rust and Node target Linux and Node reports `process.platform === "linux"`. npx acquisition may use the network and remains caller-owned; launcher/native runtime is network-free, package-local, and fail-closed. Native optional packages are internal implementation details and must never appear in consumer installation or invocation logic. Historical run `31346152236` included retired Windows artifacts and is not a future support matrix. See [`contracts/npm-cli-distribution-v2.md`](contracts/npm-cli-distribution-v2.md).

All harnesses may invoke the installed `career` CLI directly using the safe process rules in this document.

Generic raw agents remain discovery-first and should make capability/schema decisions from current installed output. A separately reviewed managed adapter may discover `career operations` and required `schema bundle` documents internally once per verified `core_version`, then cache only that non-sensitive metadata. It must not hide warnings or authority boundaries, cache private documents/results by default, or expose schema bootstrap calls to the model merely to reconstruct exact envelopes.

## Claude Code and Codex

Grant the agent access only to the local executable and the specific input paths required for the task. In project instructions, require it to:

1. run `career capabilities`
2. run `career operations` and export or bundle schemas when it needs an exact contract
3. use `json-compact` for machine parsing and enforce the declared 33,554,432-byte (32 MiB) complete-result ceiling
4. preserve evidence, uncertainty, and warnings
5. request approval before sending any source content to an external service

No provider-specific extension or duplicate TypeScript/Python scoring implementation is needed.

## Generic subprocess clients

Pass an argument vector rather than constructing a shell string. This also applies to any future npm launcher or explicit npx runner. For example:

```text
["career", "job", "match", "--input", input_path, "--format", "json-compact"]
```

Read stdout and stderr independently, enforce a caller-owned timeout, and branch on the documented exit status. Capture stdout through the operation's declared successful-output ceiling and reject any incomplete/oversized stream; the CLI itself serializes before writing and never truncates successful JSON. Do not retry typed validation failures without changing the input.

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

See [`cli.md`](cli.md) for the complete process contract and operation-specific guidance.
