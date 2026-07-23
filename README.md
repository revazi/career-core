# career-core

`career-core` is an open-source Rust library for deterministic, explainable resume evaluation and resume-to-job matching.

The project is intentionally **not an AI service**. The core performs no network requests and has no LLM dependency. Native applications, command-line tools, and coding-agent integrations can use its versioned evidence and scoring contracts. Optional AI interpretation belongs in a host application or agent, outside the authoritative core.

> **Status:** repository bootstrap. Capability discovery is available; resume evaluation and job matching are explicitly reported as planned until their contracts and golden fixtures are implemented.

## Goals

- deterministic output for identical versioned input
- explainable scores with bounded evidence and warnings
- conservative matching that prefers false negatives over unsafe equivalence
- stable JSON contracts suitable for CLIs and coding agents
- a portable Rust library suitable for future Swift bindings
- local-first operation with no telemetry or implicit network access

## Repository layout

```text
.
├── src/                         career-core library package
├── crates/career-cli/           `career` command-line adapter
├── schemas/                     versioned public JSON schemas
├── .agents/                     project and agent-integration handbook
├── AGENTS.md                    instructions loaded by coding agents
├── LICENSE-MIT
└── LICENSE-APACHE
```

The repository root is the `career-core` library package. The CLI is a separate workspace package so UI and argument-parsing dependencies do not leak into the core library.

## Requirements

- Rust 1.85 or newer

## Build and verify

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features --locked
```

## Coding-agent discovery

Run the CLI from the repository:

```bash
cargo run --quiet -p career-cli -- capabilities
```

JSON is the default output:

```json
{
  "schema_version": "career.capabilities.v1",
  "core_version": "0.1.0",
  "deterministic": true,
  "performs_network_requests": false,
  "capabilities": [
    {
      "id": "core.capabilities",
      "status": "available",
      "summary": "Discover versioned functionality exposed by this build."
    },
    {
      "id": "resume.evaluate",
      "status": "planned",
      "summary": "Evaluate resume content with explainable deterministic checks."
    },
    {
      "id": "job.match",
      "status": "planned",
      "summary": "Compare normalized resume and job evidence conservatively."
    }
  ]
}
```

Callers must only invoke entries whose `status` is `available`.

For concise human output:

```bash
cargo run --quiet -p career-cli -- capabilities --format text
```

The project includes an Agent Skills-compatible guide at `.agents/skills/career-core/SKILL.md`. Pi discovers that skill after the repository is trusted. Other coding agents can read the same file or invoke the CLI directly.

## Reference implementation

The sibling Django repository at `../resume-ai` is a read-only behavioral reference during the port. It contains mature deterministic normalization, scoring, confidence, matching, fixtures, and regression tests. `career-core` must not import it, execute it at runtime, or claim parity until Rust golden tests prove the behavior.

See [`.agents/reference-map.md`](.agents/reference-map.md) for the bounded reference map.

## Roadmap

The detailed, gated roadmap lives in [`.agents/phases.md`](.agents/phases.md). The broad order is:

1. versioned contracts and a minimal resume-evaluation vertical slice
2. deterministic resume normalization
3. explainable resume scoring parity
4. job-description normalization and conservative matching
5. hardened CLI and coding-agent distribution
6. Swift bindings for a later SwiftUI application
7. optional adapters only when concrete consumers require them

## Security and privacy

The core is designed to process sensitive career documents locally. It must not add telemetry, remote fetching, hidden model calls, or payload logging. See [`SECURITY.md`](SECURITY.md) and [`.agents/architecture.md`](.agents/architecture.md).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Coding agents must also follow [`AGENTS.md`](AGENTS.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
