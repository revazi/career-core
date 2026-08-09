# npm handoff for `pi-career`

This is the exact consumer transition contract. It is not permission to edit, publish, or release `pi-career` from Career Core.

## Consumer boundary

After all `v0.1.1` publication and public-acceptance gates pass, pi-career may address only:

- package: exact `@revazi/career@0.1.1`
- bin: `career`
- launcher runtime: Node.js `>=22`

The eight native platform packages are internal optional dependencies owned by the Career Core launcher. pi-career must never name, install, resolve, invoke, pin, select, or expose them.

Exact npx process contract:

```text
executable: npx
argv: [
  "--yes",
  "--package=@revazi/career@0.1.1",
  "career",
  ...careerArguments
]
shell: false
stdio: caller-owned
```

Equivalent command:

```bash
npx --yes --package=@revazi/career@0.1.1 career <args>
```

Never substitute `latest`, a range, or an internal native package. npx acquisition may use the network and remains an explicit pi-career-owned action. The installed Career Core launcher and native CLI make no network requests.

## Runner order owned by pi-career

A separately reviewed pi-career change may use only this order:

1. user-configured absolute `career` executable path;
2. an existing `career` on the caller's `PATH`;
3. package-local exact `@revazi/career@0.1.1` resolution;
4. a separately enabled exact-version npx runner.

Package-local resolution locates `@revazi/career/package.json` relative to pi-career, reads exact `bin.career`, and executes it with an argument vector. It must not inspect internal optional packages. Target/libc selection and package/provenance/binary checks belong exclusively to the launcher.

Unknown OS, architecture, libc, package metadata, provenance, or binary bytes fail closed. pi-career must preserve bounded `CAREER_NPM_*` diagnostics rather than selecting an internal package or broadening platform policy.

## Preserved process and Core contracts

The launcher adds no result framing. Native stdin/stdout/stderr, normal exit status, and signal termination remain intact. The operation catalog, embedded schema bundles, versioned JSON contracts, and 33,554,432-byte successful machine-output ceiling continue to apply.

pi-career must preserve:

- deterministic baseline authority
- evidence, warnings, and uncertainty
- confirmed versus provisional findings
- assisted/non-authoritative labels
- exact discovered Core version and schemas

npm registry integrity/provenance, package-contained SHA-256 consistency, and independent binary signatures are distinct. Career Core requires exact npm integrity and SLSA registry provenance, uses package-contained hashes only for consistency, and records independent native signatures as absent.

## Transitional bundled-runtime removal gate

Do not remove pi-career's transitional runtime until all conditions are true:

1. the protected Career Core `v0.1.1` workflow completed successfully from the exact annotated tag and reviewed `origin/main` SHA;
2. exact public `@revazi/career@0.1.1` and all eight internal packages expose the reviewed npm integrity and SLSA provenance;
3. no-secret public-registry acceptance passed on native macOS ARM64/x64, GNU Linux ARM64/x64, musl Linux ARM64/x64, and Windows MSVC ARM64/x64;
4. the temporary bootstrap token and GitHub environment secret were deleted and the npm token was revoked;
5. package-level GitHub trusted publishers for all nine packages point to repository `revazi/career-core`, workflow `npm-publish-v0.1.1.yml`, environment `npm-production`, and publish permission, verified with `npm trust list`;
6. a no-token OIDC rerun completed idempotently with exact integrity/provenance;
7. a separate pi-career change tests configured/PATH/package-local/exact-npx order, explicit npx network consent, cancellation, diagnostics, and exact version pinning without naming internal packages; and
8. a reviewed pi-career release no longer needs tracked transitional runtime archives.

Until every condition passes, Career Core retains the transitional artifact mechanism. Those archives are maintainer handoff inputs, not public installation or release assets.

## Remaining external work

After Career Core publication acceptance, all consumer code changes belong in pi-career. That repository owns runner discovery, npx consent/network behavior, cancellation, compatibility pinning, migration notes, release notes, and transitional-runtime removal. Career Core must not implement those consumer behaviors or edit the sibling repository.
