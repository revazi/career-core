# npm handoff for `pi-career`

This is the exact consumer transition contract. It is not permission to edit, publish, or release `pi-career` from Career Core.

## Consumer boundary

On macOS, Linux, or Linux under WSL, pi-career may address only:

- package: exact `@revazi/career@0.1.1`
- bin: `career`
- launcher runtime: Node.js `>=22`

The six active Darwin/Linux platform packages are internal optional dependencies owned by the Career Core launcher. pi-career must never name, install, resolve, invoke, pin, select, or expose them. The root crate rejects native Windows compilation; on a Windows host pi-career must build and run through WSL, where Rust and Node target Linux and Node reports `process.platform === "linux"`.

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

## Consumer release gate

Before a pi-career release changes its pinned Career package coordinate or managed launcher contract:

1. exact public `@revazi/career@X.Y.Z` and all six Darwin/Linux internal packages expose the reviewed npm integrity and SLSA provenance;
2. package-level GitHub trusted publishers for all seven packages point to repository `revazi/career-core`, workflow `npm-release.yml`, environment `npm-production`, and publish permission, verified with `npm trust list`;
3. no-token OIDC publication completes with exact integrity/provenance and public acceptance on all six target classes;
4. a separate pi-career change tests configured/PATH/package-local/exact-npx order, explicit network behavior, cancellation, diagnostics, and exact version pinning without naming internal packages; and
5. the pi-career release passes its full package, audit, compatibility, installation, and offline gates.

Career Core's transitional artifact mechanism remains a maintainer-only compatibility tool until separately removed; those archives are not public installation or release assets.

## Remaining external work

After Career Core publication acceptance, all consumer code changes belong in pi-career. That repository owns runner discovery, npx consent/network behavior, cancellation, compatibility pinning, migration notes, release notes, and transitional-runtime removal. Career Core must not implement those consumer behaviors or edit the sibling repository.
