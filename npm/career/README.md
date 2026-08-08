# `@revazi/career`

Deterministic local CLI for Career Core resume evaluation, normalization, and conservative resume-to-job matching.

## Run an exact version

```bash
npx --yes --package=@revazi/career@0.1.0 career --version
```

Use an exact version rather than `latest`. npm/npx may access the npm registry to acquire the package; after installation, the launcher and native `career` CLI perform no network requests or telemetry.

The package exposes one command: `career`. It selects and verifies its package-local native implementation for Apple Silicon macOS or x86-64 GNU/Linux with glibc 2.35 or newer. Other platforms, musl, unknown libc, and older glibc fail closed.

Platform-specific optional packages are internal implementation details. Do not install, invoke, or pin them directly.

Discover deterministic operations and embedded contracts after installation:

```bash
career capabilities
career operations --format json-compact
career schema list --format json-compact
```

Documentation, source, licenses, and security reporting are available at [github.com/revazi/career-core](https://github.com/revazi/career-core).
