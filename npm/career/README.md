# `@revazi/career`

Deterministic local CLI for Career Core resume evaluation, normalization, and conservative resume-to-job matching.

## Run an exact version

```bash
npx --yes --package=@revazi/career@0.1.1 career --version
```

Use an exact version rather than `latest`. npm/npx may access the npm registry to acquire the package; after installation, the launcher and native `career` CLI perform no network requests or telemetry.

The `0.1.1` source prepares exact native implementations for ARM64 and x64 macOS, GNU and musl Linux, and MSVC Windows. This release remains blocked until every final package executes on its exact native OS and architecture in CI; catalog or cross-compilation evidence alone is not support. Unknown platforms, architectures, libc families, and older GNU libc fail closed.

Platform-specific optional packages are internal implementation details. Do not install, invoke, or pin them directly.

Discover deterministic operations and embedded contracts after installation:

```bash
career capabilities
career operations --format json-compact
career schema list --format json-compact
```

Documentation, source, licenses, and security reporting are available at [github.com/revazi/career-core](https://github.com/revazi/career-core).
