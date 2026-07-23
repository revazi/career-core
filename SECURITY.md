# Security Policy

## Supported versions

`career-core` is pre-release software. Security fixes are applied to the latest revision on `main` until the first stable release policy is documented.

## Reporting a vulnerability

Use GitHub private vulnerability reporting when the repository is public. Do not include sensitive resume content, job descriptions, credentials, or API keys in a report. If private reporting is unavailable, contact the maintainer before sharing exploit details publicly.

## Security model

The core library processes untrusted career-document text and structured data locally. It must:

- perform no implicit network requests
- execute no instructions embedded in documents
- avoid logging source payloads
- reject oversized or invalid structured input at public boundaries
- return typed errors instead of panicking on user-controlled data
- keep output bounded and deterministic

The CLI may read only paths explicitly supplied by the caller. Future adapters must document any additional trust boundary before implementation.
