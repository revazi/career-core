# npm handoff for `pi-career`

This is the exact future consumer contract; it is not permission to edit or publish `pi-career` from Career Core.

## Only consumer package and command

`pi-career` may address only:

- package: exact `@revazi/career@0.1.0`
- bin: `career`
- launcher runtime: Node `>=22`

Native platform packages are internal optional-dependency implementation details owned entirely by the Career Core launcher. `pi-career` must never name, install, resolve, invoke, pin, select, or expose them to users.

The exact npx process contract is:

```text
executable: npx
argv: [
  "--yes",
  "--package=@revazi/career@0.1.0",
  "career",
  ...careerArguments
]
shell: false
stdio: caller-owned
```

Equivalent command documentation is:

```bash
npx --yes --package=@revazi/career@0.1.0 career <args>
```

Never substitute `latest`, a range, or an internal platform package. npx acquisition may use the network and therefore remains an explicit external runner owned by `pi-career`; the installed Career Core launcher performs no network request.

## Future runner order

A separately reviewed `pi-career` change may implement only this order:

1. user-configured absolute `career` executable path;
2. an existing `career` on the caller's `PATH`;
3. package-local exact `@revazi/career@0.1.0` resolution;
4. a separately enabled exact-version npx runner.

Package-local resolution locates `@revazi/career/package.json` relative to `pi-career`, reads exact `bin.career`, and executes it with an argument vector. It must not inspect or resolve optional native packages; target/libc selection and all package/provenance/binary checks belong to the launcher.

Musl, glibc below 2.35, malformed/unknown libc, selection failure, and unsupported hosts fail closed. `pi-career` must preserve bounded `CAREER_NPM_*` diagnostics rather than trying an internal package or broadening platform policy.

## Preserved process and Core contract

The launcher adds no framing. Native stdout/stderr, normal exit status, and signal termination remain intact. All Phase 8 operation catalog/schema bundle contracts and the 33,554,432-byte successful machine-output ceiling still apply. `pi-career` must continue enforcing exact discovered Core version/contracts and all authority, uncertainty, evidence, and assisted-document boundaries.

npm registry integrity/provenance, package-contained SHA-256 consistency, and independent native-binary signatures are distinct. Career Core requires npm SLSA registry provenance and exact package integrity for publication, retains package-contained consistency hashing at runtime, and explicitly records `independent_signature: "absent"`.

## Transitional bundled-runtime removal gate

The existing `pi-career-runtime-artifacts.yml` and related scripts remain transitional. Remove bundled/tracked runtimes only after all of these are true:

1. the protected Career Core `v0.1.0` npm workflow completed successfully;
2. exact `@revazi/career@0.1.0` exists publicly with required npm SLSA provenance;
3. no-secret public-registry acceptance passed on `macos-14` ARM64 and `ubuntu-22.04` x64, including exact optional-native selection and deterministic parity;
4. temporary bootstrap token/secret were deleted and revoked;
5. package-level GitHub trusted publishers for all implementation packages were configured to repository `revazi/career-core`, workflow `npm-publish.yml`, environment `npm-production`, and verified with `npm trust list`;
6. a separate `pi-career` change implements and tests configured/PATH/package-local/exact-npx order, explicit npx network consent, cancellation, and diagnostics without naming native implementation packages; and
7. a reviewed `pi-career` release no longer requires tracked transitional runtime archives.

Until every gate passes, Career Core keeps the transitional artifact mechanism. Its archives are not public installation or release assets.

## External work remaining

After Career Core publication acceptance, the only code change belongs in `pi-career`. That repository owns runner discovery, npx consent/network behavior, process cancellation, compatibility pinning, migration/release notes, and removal of bundled runtimes. Career Core must not implement those consumer behaviors.
