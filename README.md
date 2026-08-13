# Career Core

Career Core is a deterministic, local-first Rust engine for resume evaluation, resume and job-description normalization, and conservative resume-to-job matching.

It is **not** an AI service. The core library performs no network requests, model calls, telemetry, filesystem access, database access, or UI work. Given the same versioned input, it produces the same bounded, explainable output.

## Release status

- Source and CLI version: `0.2.0`
- npm consumer package candidate: exact `@revazi/career@0.2.0`
- Public CLI name: `career`
- Minimum Rust version for source builds: Rust `1.85`
- npm launcher runtime: Node.js `22` or newer

Career Core, its CLI, adapters, npm source, and releases support macOS and Linux only and do not build or run natively on Windows. Windows users must build and run Career Core and pi-career through WSL, where the target and runtime are Linux and the Linux package policy applies. Existing Apple iOS/Swift build targets remain supported.

Use exact versions, never `latest` or a range.

## What Career Core does

| Operation | Purpose |
|---|---|
| `resume evaluate` | Checks recognized core resume-section coverage. This is not a complete quality or ATS score. |
| `resume analyze` | Runs 18 deterministic readiness checks with confidence-aware evidence and limitations. |
| `resume normalize` | Extracts bounded, source-grounded resume facts and parser confidence. |
| `resume enrich` | Validates an explicit external proposal without making a provider call. |
| `resume analysis-suggestions-review` | Reviews bounded external advisory suggestions beside an unchanged baseline analysis. |
| `resume analysis-replacements-review` | Reviews bounded exact replacements beside an unchanged baseline analysis. |
| `resume variant-review` | Reviews evidence-linked external resume changes without certifying generated prose. |
| `resume variant-materialize` | Revalidates and applies only explicitly selected assisted changes. |
| `job normalize` | Extracts source-grounded job requirements and confidence from caller-supplied text. |
| `job match` | Conservatively matches independently normalized resume and job baselines. |

Every score is bounded and explainable. Low-confidence parsing remains provisional. A field reported as `not_detected` is not necessarily absent. Assisted content is labeled non-authoritative and cannot alter deterministic analysis or matching.

## What Career Core does not do

Career Core does not:

- fetch resumes, job URLs, or other remote content
- call an LLM or provider
- guarantee ATS performance, interviews, hiring, or factual correctness
- inspect visual PDF/DOCX layout
- treat related technologies as interchangeable without a reviewed equivalence
- write to the original resume
- use assisted fields as authoritative scoring or matching input
- publish telemetry or log complete resume/job payloads

Document text is always data, never instructions.

## Install the CLI

### npm

After the six-target publication and public-acceptance gate passes, install the exact release:

```bash
npm install --save-exact @revazi/career@0.2.0
npx --yes --package=@revazi/career@0.2.0 career --version
```

Users install only `@revazi/career`. Its six active platform-specific optional packages are internal launcher implementation details and must not be installed, invoked, or pinned directly. Native Windows installation is unsupported; use WSL on a Windows host.

npm/npx may contact the npm registry to acquire packages. After installation, the launcher and native CLI make no network requests and provide no PATH or runtime-download fallback.

### From a reviewed source checkout

```bash
git clone https://github.com/revazi/career-core.git
cd career-core
cargo install --path crates/career-cli --locked
career --version
```

For reproducible use, check out an exact reviewed commit or release tag. On a Windows host, run these source-build commands inside WSL; native Windows Cargo builds fail at compile time.

## Supported npm targets

Active source and future releases contain one launcher and six native implementations:

| Operating system | Architecture | Runtime |
|---|---|---|
| macOS | ARM64 | native Mach-O |
| macOS | x86-64 | native Mach-O |
| GNU/Linux | x86-64 | glibc `2.35` or newer |
| GNU/Linux | ARM64 | glibc `2.35` or newer |
| musl Linux | x86-64 | native musl |
| musl Linux | ARM64 | native musl |

A target is supported only after its final public package executes on the exact native OS and architecture. Unknown OS, architecture, libc, package metadata, provenance, executable format, or binary bytes fail closed. Emulation and cross-compilation are not support evidence. WSL is supported as Linux; native Windows is rejected by the root Rust crate at compile time and has no launcher route in source.

See [`docs/distribution.md`](docs/distribution.md) and [`docs/contracts/npm-cli-distribution-v2.md`](docs/contracts/npm-cli-distribution-v2.md) for the exact policy.

## Discover the machine interface

The installed binary embeds its operation catalog and Draft 2020-12 JSON Schemas:

```bash
career capabilities --format json-compact
career operations --format json-compact
career schema list --format json-compact
career schema export --id career.job_match_input.v1
career schema bundle --id career.job_match_input.v1 --format json-compact
```

Call only capabilities whose status is `available`. `career.operation_catalog.v1` declares exact command paths, transports, schemas, input ceilings, and output limits.

Machine-output rules:

- JSON is the default.
- `json-pretty` explicitly selects pretty JSON.
- `json-compact` emits one JSON document on one line.
- `text` is human-only.
- Successful machine output is fully serialized and checked before writing.
- Complete successful machine output is limited to 33,554,432 bytes, including its trailing newline.
- Diagnostics go to stderr and do not repeat source documents.

See [`docs/cli.md`](docs/cli.md), [`docs/agent-usage.md`](docs/agent-usage.md), and [`docs/contracts/managed-adapter-v1.md`](docs/contracts/managed-adapter-v1.md).

## Minimal examples

Evaluate recognized resume sections:

```bash
career resume evaluate \
  --input fixtures/resume/phase1/complete-sections.input.json
```

Run full deterministic resume analysis:

```bash
career resume analyze \
  --input fixtures/resume/phase3/complete-analysis.input.json
```

Normalize a job description and match a resume to a job:

```bash
career job normalize \
  --input fixtures/job/phase4a/complete-normalization.input.json

career job match \
  --input fixtures/job/phase4b/complete-match.input.json
```

Use `--input -` for stdin where supported. Invalid user input returns a versioned typed error and a nonzero exit status; it must not panic.

## Integration with pi-career

The Pi integration is maintained separately in [`revazi/pi-career`](https://github.com/revazi/pi-career). Career Core does not ship a Pi package or modify that repository.

On macOS, Linux, or Linux under WSL, pi-career should address only:

```text
package: @revazi/career@0.2.0
bin: career
```

It must not name or resolve internal platform packages. The exact consumer transition and removal gates are documented in [`docs/pi-career-npm-handoff.md`](docs/pi-career-npm-handoff.md).

## Library and adapter boundaries

Repository layout:

```text
src/                         pure career-core Rust library
crates/career-cli/           filesystem/stdin/CLI adapter
crates/career-swift/         narrow UniFFI JSON facade
swift/CareerCoreSwift/       local, unpublished Swift wrapper
npm/                         private npm source templates
schemas/                     versioned public JSON Schemas
fixtures/                    synthetic reviewed contract fixtures
docs/contracts/              scoring, matching, and distribution policies
```

The root library has no network, filesystem, database, UI, telemetry, provider, prompt, or model behavior. Adapters depend inward on the core; the core never depends on an adapter.

The Swift facade is local and unpublished. It is not the SwiftUI product, persistence layer, or an App Store package. See [`docs/swift-bindings.md`](docs/swift-bindings.md).

## Build and verify

```bash
scripts/verify-installed-cli.sh
node --test npm/tests/launcher.test.js
scripts/test-npm-cli-packages.sh
scripts/test-npm-publication.sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features --locked
```

The complete required command ladder is in [`AGENTS.md`](AGENTS.md). Release publication has additional clean-source, annotated-tag, native-runner, candidate-integrity, npm provenance, and public-acceptance gates in [`docs/releasing.md`](docs/releasing.md).

Source templates remain `private: true`. Generated public manifests, native binaries, provenance records, and tarballs are staged outside the checkout. No Rust crate, Homebrew formula, independent native signature, notarized binary, or custom GitHub Release asset is part of the `v0.2.0` candidate.

## Security and privacy

Career documents may contain sensitive information. Keep inputs local unless the user explicitly approves external processing. External proposals are untrusted data and remain separate from deterministic baselines.

Report vulnerabilities privately as described in [`SECURITY.md`](SECURITY.md). Do not include real resumes, job descriptions, credentials, or other sensitive payloads in public issues.

## Maintenance, contributing, and license

The primary maintainer is [Revaz Zakalashvili](https://github.com/revazi). Ownership and support policy are in [`MAINTAINERS.md`](MAINTAINERS.md); contribution guidance is in [`CONTRIBUTING.md`](CONTRIBUTING.md).

Project-authored source is available under either license, at your option:

- [`MIT`](LICENSE-MIT)
- [`Apache-2.0`](LICENSE-APACHE)

Third-party notices, including the adapter-only MPL-2.0 UniFFI components, are recorded in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
