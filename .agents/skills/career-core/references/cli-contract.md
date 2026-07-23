# CLI contract reference

## Bootstrap command

```bash
career capabilities [--format json|text]
```

JSON is the default. The current schema is tracked in:

```text
schemas/capabilities-v1.schema.json
```

Current JSON fields:

- `schema_version`
- `core_version`
- `deterministic`
- `performs_network_requests`
- `capabilities[]`
  - `id`
  - `status`
  - `summary`

`available` means callable in the current build. `planned` means no command contract exists yet.

## Execution from source

```bash
cargo run --quiet -p career-cli -- capabilities
```

The `--quiet` Cargo flag suppresses Cargo status output; it does not alter `career` output.

## Safety

The capabilities command reads no documents, performs no network requests, and writes no files. Future operation contracts must be added here only after their capability becomes available.
