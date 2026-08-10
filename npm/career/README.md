# `@revazi/career`

Deterministic local CLI for Career Core resume evaluation, normalization, and conservative resume-to-job matching.

## Run an exact version

```bash
npx --yes --package=@revazi/career@0.1.1 career --version
```

Use an exact version rather than `latest`. npm/npx may access the npm registry to acquire the package; after installation, the launcher and native `career` CLI perform no network requests or telemetry.

`0.1.1` provides exact accepted native implementations for ARM64 and x64 macOS, GNU and musl Linux, and MSVC Windows; protected publication and all eight public-registry acceptance jobs passed. Catalog metadata, cross-compilation, and skipped jobs are not support evidence. Unknown platforms, architectures, libc families, and GNU libc older than 2.35 fail closed.

Platform-specific optional packages are internal implementation details. Do not install, invoke, or pin them directly.

Discover deterministic operations and embedded contracts after installation:

```bash
career capabilities
career operations --format json-compact
career schema list --format json-compact
```

Documentation, source, licenses, and security reporting are available at [github.com/revazi/career-core](https://github.com/revazi/career-core).
