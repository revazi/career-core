# Agent handbook

This directory contains the detailed operating context for humans and coding agents working on `career-core`.

Pi automatically loads the root [`AGENTS.md`](../AGENTS.md). It does **not** automatically inject every Markdown file in this directory. `AGENTS.md` tells agents which documents to read for a task so context stays focused.

## Reading order

Every implementation task:

1. [`current-phase.md`](current-phase.md) — what is currently allowed
2. [`architecture.md`](architecture.md) — invariants and dependency direction
3. [`workflow.md`](workflow.md) — implementation and verification process
4. Relevant phase in [`phases.md`](phases.md)

Read when applicable:

- [`project.md`](project.md) — vision, users, scope, and vocabulary
- [`reference-map.md`](reference-map.md) — bounded map into `../resume-ai`
- [`agent-integration.md`](agent-integration.md) — CLI, JSON, and future external adapters
- [`decisions.md`](decisions.md) — decisions that must not be silently reversed

## Maintenance rule

Documentation is part of the product contract. When implementation changes capability status, public JSON, commands, phase acceptance, or architecture, update the corresponding handbook file in the same change.

Do not use this directory as a dumping ground for session notes or speculative designs. Keep current facts here; put bounded proposals under `docs/design/` if that directory is introduced later.
