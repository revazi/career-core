# Public schemas

This directory contains versioned machine-readable JSON schemas for available `career-core` and `career` CLI contracts.

Rules:

- Never edit an existing stable schema to introduce a breaking semantic change.
- Keep schema version values aligned with Rust contract constants and CLI examples.
- Planned capabilities do not receive operation schemas until their phase defines and tests the contract.
- Examples and schemas must contain synthetic data only.

Current schemas:

- `capabilities-v1.schema.json` — capability discovery
