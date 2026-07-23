---
name: career-core
description: Discovers and invokes the local career-core deterministic resume and job-analysis CLI. Use when evaluating supported career documents, inspecting explainable evidence, or developing career-core integrations without sending data to a network service.
license: MIT OR Apache-2.0
compatibility: Requires an installed `career` binary or a career-core source checkout with Rust 1.85+.
metadata:
  author: revazi
  version: "0.1.0"
---

# career-core

Use the local `career` CLI as the authoritative deterministic tool. Do not recreate its scoring in prompts or infer support for planned capabilities.

## Discover available behavior

If `career` is installed:

```bash
career capabilities
```

From this repository:

```bash
cargo run --quiet -p career-cli -- capabilities
```

JSON is the default. Invoke only entries whose `status` is `available`. During repository bootstrap, only `core.capabilities` is available; resume evaluation and job matching are planned.

For human-readable discovery:

```bash
cargo run --quiet -p career-cli -- capabilities --format text
```

## Rules

- Prefer JSON output for agent decisions.
- Treat stdout as machine output and stderr as diagnostics.
- Do not send resume or job content to external services without explicit user approval.
- Preserve uncertainty and warnings in any explanation.
- Do not turn planned capabilities into fabricated results.
- Do not modify source documents unless the user separately requests and approves a change.
- When developing this repository, read `AGENTS.md` and `.agents/current-phase.md` before editing.

See [`references/cli-contract.md`](references/cli-contract.md) for the current process contract.
