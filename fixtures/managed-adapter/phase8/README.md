# Phase 8 managed-adapter fixtures

These fixtures are synthetic Career Core discovery contracts created for this repository. They contain no resume, vacancy, provider, or personal data.

`capabilities.pre-phase8.expected.json` freezes the exact canonical capability bytes from clean Phase 8 base commit `60d4a04a7be1ebd5ca7c7577458bd45fdbb41ab2`; Phase 8 must not alter them.

`operation-catalog.expected.json` is the canonical pretty JSON emitted by `career operations`. It freezes stable operation ordering, capability mapping, CLI paths, input transports, schema identifiers, and byte bounds for `career.operation_catalog.v1`.
