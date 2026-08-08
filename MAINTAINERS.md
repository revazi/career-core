# Maintainers

## Primary maintainer

| Name | Role | GitHub | Contact |
| --- | --- | --- | --- |
| Revaz Zakalashvili | Primary maintainer | [@revazi](https://github.com/revazi) | [revaz.zakalashvili@gmail.com](mailto:revaz.zakalashvili@gmail.com) |

Revaz maintains the deterministic Rust library, `career` CLI, public schemas and contracts, Swift binding boundary, distribution policy, and release review for Career Core.

## Contact and support

- Use [GitHub Issues](https://github.com/revazi/career-core/issues) for sanitized bug reports and scoped feature requests.
- Use pull requests for reviewed contributions and follow [`CONTRIBUTING.md`](CONTRIBUTING.md).
- Report vulnerabilities privately as described in [`SECURITY.md`](SECURITY.md); never put exploit details or private career documents in a public issue.

The project does not promise a response-time or release-frequency service level. Support and release decisions are made against the latest reviewed `main` branch and the repository's deterministic, privacy, compatibility, and security constraints.

## Governance

The primary maintainer reviews contract/compatibility changes, dependencies, supported targets, security-sensitive behavior, and releases. No crate, binary, tag, or package-manager channel is published without explicit approval and [`docs/releasing.md`](docs/releasing.md) verification. The `@revazi/career@0.1.0` release additionally requires independent scope-ownership proof, exact clean annotated tag/main candidate provenance, protected `npm-production` approval, explicit bootstrap-versus-OIDC selection, native-before-launcher publication, npm SLSA provenance, immediate bootstrap-token deletion/revocation, verified package-level trusted publishers, and both public-registry acceptance jobs. Internal native package names are not consumer surfaces.
