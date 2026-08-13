"use strict";

const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");
const { EventEmitter } = require("node:events");
const { after, before, test } = require("node:test");

const ROOT = path.resolve(__dirname, "..");
const LAUNCHER_SOURCE = path.join(ROOT, "career", "bin", "career.js");
const LAUNCHER_MANIFEST_SOURCE = path.join(ROOT, "career", "package.json");
const TARGET_CATALOG_SOURCE = path.join(ROOT, "career", "targets.json");
const PACKAGE_VERSION = JSON.parse(fs.readFileSync(LAUNCHER_MANIFEST_SOURCE, "utf8")).version;
const HELPER_SOURCE = path.join(__dirname, "fixtures", "native-helper.rs");
const launcher = require(LAUNCHER_SOURCE);
const CATALOG = launcher.loadTargetCatalog(TARGET_CATALOG_SOURCE);
const PLATFORM_LICENSE_FILES = ["LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"];
const temporaryRoots = [];
let helperBinary;

function makeTemporaryRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  temporaryRoots.push(root);
  return root;
}

async function waitForCondition(description, predicate, timeoutMs = 3_000) {
  const deadline = Date.now() + timeoutMs;
  while (!predicate()) {
    if (Date.now() >= deadline) throw new Error(`Timed out waiting for ${description}.`);
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

function linuxProcessCatchesSignal(pid, signal) {
  if (process.platform !== "linux") return true;
  const status = fs.readFileSync(`/proc/${pid}/status`, "utf8");
  const caughtField = status.match(/^SigCgt:\s+([0-9a-f]+)$/imu);
  assert.ok(caughtField, "Linux process status must expose SigCgt");
  const signalNumber = os.constants.signals[signal];
  const signalBit = 1n << BigInt(signalNumber - 1);
  return (BigInt(`0x${caughtField[1]}`) & signalBit) !== 0n;
}

function processIsGone(pid) {
  try {
    process.kill(pid, 0);
    return false;
  } catch (error) {
    if (error?.code === "ESRCH") return true;
    throw error;
  }
}

function throwUnlessProcessIsGone(error) {
  if (error?.code === "ESRCH") return;
  throw error;
}

function killProcessForCleanup(pid) {
  if (!Number.isInteger(pid)) return;
  try {
    process.kill(pid, "SIGKILL");
  } catch (error) {
    throwUnlessProcessIsGone(error);
  }
}

function cleanupCancellationProcesses(child, pidFile) {
  if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
  if (!fs.existsSync(pidFile)) return;
  killProcessForCleanup(Number(fs.readFileSync(pidFile, "utf8")));
}

before(() => {
  const buildRoot = makeTemporaryRoot("career-npm-helper-");
  helperBinary = path.join(buildRoot, "career");
  const result = spawnSync(
    "rustc",
    ["--edition=2024", "-C", "opt-level=0", HELPER_SOURCE, "-o", helperBinary],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  fs.chmodSync(helperBinary, 0o755);
});

after(() => {
  for (const root of temporaryRoots.reverse()) {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function libcRuntimeFor(target) {
  if (target.libc_family === "glibc") return { family: "glibc", version: "2.35" };
  if (target.libc_family === "musl") return { family: "musl", version: null };
  return null;
}

function currentTarget() {
  const libcRuntime = process.platform === "linux" ? launcher.detectLinuxLibc() : null;
  return launcher.selectTarget(process.platform, process.arch, libcRuntime, CATALOG);
}

function differentRustTarget(target) {
  const different = CATALOG.targets.find((value) => value.rust_target !== target.rust_target);
  assert.ok(different);
  return different.rust_target;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, JSON.stringify(value, null, 2) + "\n");
}

function sha256(filePath) {
  return crypto.createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function syntheticBinary(target) {
  const binary = Buffer.alloc(512);
  if (target.binary_format.startsWith("mach-o-64-")) {
    binary.writeUInt32LE(0xfeedfacf, 0);
    binary.writeUInt32LE(target.binary_architecture === "aarch64" ? 0x0100000c : 0x01000007, 4);
  } else if (target.binary_format.startsWith("elf-64-")) {
    Buffer.from([0x7f, 0x45, 0x4c, 0x46, 2, 1]).copy(binary);
    binary.writeUInt16LE(target.binary_architecture === "aarch64" ? 0xb7 : 0x3e, 18);
  } else {
    throw new Error(`Unsupported synthetic binary format: ${target.binary_format}`);
  }
  binary[binary.length - 1] = 1;
  return binary;
}

function provenanceFor(platformDirectory, target, version = PACKAGE_VERSION) {
  const binaryPath = path.join(platformDirectory, target.executable);
  const runnerLibc =
    target.libc_family === "glibc" ? "glibc 2.35" : target.libc_family === "musl" ? "musl" : null;
  return {
    schema_version: "career.npm_native_provenance.v2",
    package: {
      name: target.native_package,
      version,
      platform_key: target.platform_key,
      node_platform: target.node_platform,
      node_arch: target.node_arch,
      libc_family: target.libc_family,
      rust_target: target.rust_target,
      minimum_glibc_version: target.minimum_glibc_version,
    },
    source: {
      repository: "https://github.com/revazi/career-core",
      git_sha: "0".repeat(40),
      git_ref: null,
      git_tag: null,
      git_dirty: false,
      publication_candidate: false,
    },
    build: {
      command: [
        "cargo",
        "build",
        "--release",
        "--locked",
        "-p",
        "career-cli",
        "--target",
        target.rust_target,
      ],
      profile: "release",
      locked: true,
      rustc_version: "rustc 1.85.0 (synthetic test)",
      cargo_version: "cargo 1.85.0 (synthetic test)",
      runner: {
        os: target.runner_os,
        arch: target.runner_arch,
        image: "synthetic-test",
        libc: runnerLibc,
      },
    },
    executable: {
      file_name: target.executable,
      binary_format: target.binary_format,
      binary_architecture: target.binary_architecture,
      file_invariant: target.file_invariant,
      archive_mode: target.archive_mode,
      mode: target.executable_mode,
      size_bytes: fs.statSync(binaryPath).size,
      sha256: sha256(binaryPath),
    },
    integrity: {
      npm_registry_integrity: "external_to_launcher_runtime",
      package_contained_sha256: "consistency_only",
      independent_signature: "absent",
    },
  };
}

function makeInstalledTree(target = currentTarget(), useSyntheticBinary = false) {
  const root = makeTemporaryRoot("career npm tree Unicode 東京 ");
  const launcherDirectory = path.join(root, "node_modules", "@revazi", "career");
  const platformDirectory = path.join(
    root,
    "node_modules",
    "@revazi",
    target.native_package.slice("@revazi/".length),
  );
  fs.mkdirSync(path.join(launcherDirectory, "bin"), { recursive: true });
  fs.mkdirSync(platformDirectory, { recursive: true });
  fs.copyFileSync(LAUNCHER_MANIFEST_SOURCE, path.join(launcherDirectory, "package.json"));
  fs.copyFileSync(TARGET_CATALOG_SOURCE, path.join(launcherDirectory, "targets.json"));
  fs.copyFileSync(LAUNCHER_SOURCE, path.join(launcherDirectory, "bin", "career.js"));
  fs.copyFileSync(path.join(ROOT, "career", "README.md"), path.join(launcherDirectory, "README.md"));
  fs.copyFileSync(
    path.join(ROOT, "platforms", target.platform_key, "package.json"),
    path.join(platformDirectory, "package.json"),
  );
  const binaryPath = path.join(platformDirectory, target.executable);
  if (useSyntheticBinary) {
    fs.writeFileSync(binaryPath, syntheticBinary(target));
  } else {
    fs.copyFileSync(helperBinary, binaryPath);
  }
  fs.chmodSync(binaryPath, target.executable_mode === "0755" ? 0o755 : 0o644);
  writeJson(
    path.join(platformDirectory, "provenance.json"),
    provenanceFor(platformDirectory, target),
  );
  return {
    root,
    target,
    launcherDirectory,
    launcherManifestPath: path.join(launcherDirectory, "package.json"),
    targetCatalogPath: path.join(launcherDirectory, "targets.json"),
    launcherBin: path.join(launcherDirectory, "bin", "career.js"),
    platformDirectory,
    platformManifestPath: path.join(platformDirectory, "package.json"),
    provenancePath: path.join(platformDirectory, "provenance.json"),
    binaryPath,
  };
}

function resolveTree(tree, overrides = {}) {
  return launcher.resolveVerifiedBinary({
    platform: tree.target.node_platform,
    arch: tree.target.node_arch,
    libcRuntime: libcRuntimeFor(tree.target),
    launcherManifestPath: tree.launcherManifestPath,
    targetCatalogPath: tree.targetCatalogPath,
    packageResolver: () => tree.platformManifestPath,
    ...overrides,
  });
}

function expectCode(callback, code) {
  assert.throws(callback, (error) => {
    assert.equal(error.code, code);
    assert.ok(error.message.length > 0 && error.message.length <= 256);
    return true;
  });
}

function mutateJson(filePath, mutation) {
  const value = readJson(filePath);
  mutation(value);
  writeJson(filePath, value);
}

function refreshProvenanceExecutable(tree) {
  const provenance = readJson(tree.provenancePath);
  provenance.executable.size_bytes = fs.statSync(tree.binaryPath).size;
  provenance.executable.sha256 = sha256(tree.binaryPath);
  writeJson(tree.provenancePath, provenance);
}

function runInstalledLauncher(tree, args, options = {}) {
  return spawnSync(process.execPath, [tree.launcherBin, ...args], {
    encoding: options.encoding || "utf8",
    input: options.input,
    env: { ...process.env, ...(options.env || {}) },
  });
}

test("selects all six exact catalog targets and rejects unknown hosts and libc", () => {
  const cases = [
    ["darwin", "arm64", null, "darwin-arm64"],
    ["darwin", "x64", null, "darwin-x64"],
    ["linux", "x64", { family: "glibc", version: "2.35" }, "linux-x64-gnu"],
    ["linux", "arm64", { family: "glibc", version: "2.35" }, "linux-arm64-gnu"],
    ["linux", "x64", { family: "musl", version: null }, "linux-x64-musl"],
    ["linux", "arm64", { family: "musl", version: null }, "linux-arm64-musl"],
  ];
  for (const [platform, arch, libcRuntime, key] of cases) {
    assert.equal(launcher.selectTarget(platform, arch, libcRuntime, CATALOG).platform_key, key);
  }
  for (const libcRuntime of [
    { family: "glibc", version: "2.34" },
    { family: "glibc", version: "2.35-malformed" },
    { family: "unknown", version: null },
    { family: "musl", version: "1.2" },
    null,
  ]) {
    expectCode(
      () => launcher.selectTarget("linux", "x64", libcRuntime, CATALOG),
      "CAREER_NPM_UNSUPPORTED_LIBC",
    );
  }
  expectCode(
    () => launcher.selectTarget("linux", "ppc64", { family: "glibc", version: "2.35" }, CATALOG),
    "CAREER_NPM_UNSUPPORTED_PLATFORM",
  );
  for (const [platform, arch] of [
    ["win32", "x64"],
    ["win32", "arm64"],
    ["freebsd", "x64"],
  ]) {
    expectCode(
      () => launcher.selectTarget(platform, arch, null, CATALOG),
      "CAREER_NPM_UNSUPPORTED_PLATFORM",
    );
  }
});

test("Linux libc detection requires positive bounded architecture-matched evidence", () => {
  assert.deepEqual(
    launcher.detectLinuxLibc(
      () => ({ header: { glibcVersionRuntime: "2.36" }, sharedObjects: [] }),
      "x64",
    ),
    { family: "glibc", version: "2.36" },
  );
  assert.deepEqual(
    launcher.detectLinuxLibc(
      () => ({ header: {}, sharedObjects: ["/lib/ld-musl-x86_64.so.1"] }),
      "x64",
    ),
    { family: "musl", version: null },
  );
  for (const reportProvider of [
    () => ({ header: {}, sharedObjects: [] }),
    () => ({ header: { glibcVersionRuntime: "musl" }, sharedObjects: [] }),
    () => ({ header: { glibcVersionRuntime: "2.36" } }),
    () => ({ header: { glibcVersionRuntime: "2.36" }, sharedObjects: "invalid" }),
    () => ({ header: { glibcVersionRuntime: "2.36" }, sharedObjects: [42] }),
    () => ({ header: {}, sharedObjects: ["/lib/ld-musl-aarch64.so.1"] }),
    () => ({
      header: { glibcVersionRuntime: "2.36" },
      sharedObjects: ["/lib/ld-musl-x86_64.so.1"],
    }),
    () => ({ header: {}, sharedObjects: Array(1025).fill("/lib/libc.so") }),
    () => {
      throw new Error("synthetic report failure");
    },
  ]) {
    assert.deepEqual(launcher.detectLinuxLibc(reportProvider, "x64"), {
      family: "unknown",
      version: null,
    });
  }
});

test("resolves and verifies every exact synthetic target package", () => {
  for (const target of CATALOG.targets) {
    const tree = makeInstalledTree(target, true);
    const result = resolveTree(tree);
    assert.equal(result.binaryPath, tree.binaryPath);
    assert.equal(result.target.platform_key, target.platform_key);
    assert.equal(result.launcherVersion, PACKAGE_VERSION);
  }
});

test("resolves and verifies a complete package-local native install", () => {
  const tree = makeInstalledTree();
  const result = resolveTree(tree);
  assert.equal(result.binaryPath, tree.binaryPath);
  assert.equal(result.target.platform_key, tree.target.platform_key);
  assert.equal(result.launcherVersion, PACKAGE_VERSION);
});

test("reverifies immediately before spawn and rejects a replaced pathname", async () => {
  const tree = makeInstalledTree();
  const resolved = resolveTree(tree);
  const replacement = fs.readFileSync(tree.binaryPath);
  replacement[replacement.length - 1] ^= 0xff;
  fs.renameSync(tree.binaryPath, `${tree.binaryPath}.verified`);
  fs.writeFileSync(tree.binaryPath, replacement, { mode: 0o755 });
  let spawned = false;
  await assert.rejects(
    async () =>
      launcher.launchResolvedBinary(resolved, [], () => {
        spawned = true;
        throw new Error("unverified replacement reached spawn");
      }),
    (error) => error.code === "CAREER_NPM_BINARY_HASH_MISMATCH",
  );
  assert.equal(spawned, false);
});

test("rejects a missing optional package with a bounded path-free error", () => {
  const tree = makeInstalledTree();
  let captured;
  try {
    resolveTree(tree, {
      packageResolver: () => {
        throw new Error(`/private/path/${"x".repeat(5000)}`);
      },
    });
  } catch (error) {
    captured = error;
  }
  assert.equal(captured.code, "CAREER_NPM_PLATFORM_PACKAGE_MISSING");
  const rendered = launcher.formatLauncherError(captured);
  assert.ok(rendered.length <= 512);
  assert.doesNotMatch(rendered, /private\/path/u);
  assert.match(rendered, /optional dependencies enabled/u);
});

test("the production package-local resolver fails when the optional package is absent", () => {
  const tree = makeInstalledTree();
  fs.rmSync(tree.platformDirectory, { recursive: true, force: true });
  const result = runInstalledLauncher(tree, ["--version"]);
  assert.equal(result.status, 1);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /CAREER_NPM_PLATFORM_PACKAGE_MISSING/u);
  assert.ok(result.stderr.length <= 512);
  assert.doesNotMatch(result.stderr, new RegExp(tree.root.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "u"));
});

test("rejects missing, malformed, and byte-drifted target catalogs", async (t) => {
  await t.test("missing", () => {
    const tree = makeInstalledTree();
    fs.rmSync(tree.targetCatalogPath);
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("malformed", () => {
    const tree = makeInstalledTree();
    fs.writeFileSync(tree.targetCatalogPath, "[]\n");
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("reviewed byte drift", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.targetCatalogPath, (catalog) => {
      catalog.targets[0].native_package = "@revazi/career-unreviewed";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
});

test("rejects launcher package-set, order, version, and lifecycle drift", async (t) => {
  await t.test("optional dependency version", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      manifest.optionalDependencies["@revazi/career-linux-x64-gnu"] = "0.1.2";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("missing optional dependency", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      delete manifest.optionalDependencies["@revazi/career-linux-arm64-musl"];
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("extra optional dependency", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      manifest.optionalDependencies["@revazi/career-unreviewed"] = manifest.version;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("reordered optional dependency", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      const value = manifest.optionalDependencies["@revazi/career-darwin-arm64"];
      delete manifest.optionalDependencies["@revazi/career-darwin-arm64"];
      manifest.optionalDependencies["@revazi/career-darwin-arm64"] = value;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("reordered platform package list", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      manifest.career_launcher.platform_packages.reverse();
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("duplicated platform package list", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      manifest.career_launcher.platform_packages[5] = manifest.career_launcher.platform_packages[0];
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
  await t.test("lifecycle script", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      manifest.scripts = { postinstall: "node download.js" };
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_LAUNCHER_MANIFEST_INVALID");
  });
});

test("rejects malformed and mismatched platform manifests", async (t) => {
  await t.test("malformed JSON", () => {
    const tree = makeInstalledTree();
    fs.writeFileSync(tree.platformManifestPath, "not-json\n");
    expectCode(() => resolveTree(tree), "CAREER_NPM_PLATFORM_MANIFEST_INVALID");
  });
  await t.test("package name", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      manifest.name = "@revazi/not-career";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PLATFORM_PACKAGE_MISMATCH");
  });
  await t.test("version", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      manifest.version = "0.1.2";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_VERSION_MISMATCH");
  });
  await t.test("target metadata", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      manifest.career_native.rust_target = differentRustTarget(tree.target);
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_TARGET_MISMATCH");
  });
  await t.test("lifecycle script", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      manifest.scripts = { install: "node install.js" };
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PLATFORM_MANIFEST_INVALID");
  });
});

test("rejects missing, malformed, and mismatched provenance", async (t) => {
  await t.test("missing", () => {
    const tree = makeInstalledTree();
    fs.rmSync(tree.provenancePath);
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_MISSING");
  });
  await t.test("malformed", () => {
    const tree = makeInstalledTree();
    fs.writeFileSync(tree.provenancePath, "[]\n");
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_INVALID");
  });
  await t.test("schema", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.schema_version = "career.npm_native_provenance.v3";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_INVALID");
  });
  await t.test("package identity", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.package.name = "@revazi/not-career";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_PACKAGE_MISMATCH");
  });
  await t.test("version", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.package.version = "0.1.2";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_VERSION_MISMATCH");
  });
  await t.test("target", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.package.rust_target = differentRustTarget(tree.target);
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_TARGET_MISMATCH");
  });
  await t.test("dirty publication candidate", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.source.git_ref = `refs/tags/v${PACKAGE_VERSION}`;
      value.source.git_tag = `v${PACKAGE_VERSION}`;
      value.source.git_dirty = true;
      value.source.publication_candidate = true;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_DIRTY_PROVENANCE");
  });
  await t.test("removed private guard without clean candidate provenance", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      delete manifest.private;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_DIRTY_PROVENANCE");
  });
  await t.test("explicit publishable manifest without candidate provenance", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      manifest.private = false;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_DIRTY_PROVENANCE");
  });
  await t.test("clean candidate provenance permits a future manifest without the guard", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      delete manifest.private;
    });
    mutateJson(tree.provenancePath, (value) => {
      value.source.git_ref = `refs/tags/v${PACKAGE_VERSION}`;
      value.source.git_tag = `v${PACKAGE_VERSION}`;
      value.source.publication_candidate = true;
      value.build.rustc_version = "rustc 1.97.1 (synthetic test)";
      value.build.cargo_version = "cargo 1.97.1 (synthetic test)";
    });
    assert.equal(resolveTree(tree).binaryPath, tree.binaryPath);
  });
  await t.test("publication candidate tag/version mismatch", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      delete manifest.private;
    });
    mutateJson(tree.provenancePath, (value) => {
      value.source.git_ref = "refs/tags/v0.1.2";
      value.source.git_tag = "v0.1.2";
      value.source.publication_candidate = true;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_INVALID");
  });
  await t.test("runner target mismatch", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.build.runner.os = tree.target.runner_os === "Linux" ? "macOS" : "Linux";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_INVALID");
  });
  await t.test("unsupported signature claim", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.integrity.independent_signature = "present";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_INVALID");
  });
});

test("rejects unsafe binary file types and exact mode drift", async (t) => {
  await t.test("missing", () => {
    const tree = makeInstalledTree();
    fs.rmSync(tree.binaryPath);
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_MISSING");
  });
  await t.test("symlink", () => {
    const tree = makeInstalledTree();
    fs.rmSync(tree.binaryPath);
    fs.symlinkSync(helperBinary, tree.binaryPath);
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_TYPE_INVALID");
  });
  await t.test("non-regular directory", () => {
    const tree = makeInstalledTree();
    fs.rmSync(tree.binaryPath);
    fs.mkdirSync(tree.binaryPath);
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_TYPE_INVALID");
  });
  await t.test("Unix mode", () => {
    const tree = makeInstalledTree();
    fs.chmodSync(tree.binaryPath, 0o700);
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_MODE_MISMATCH");
  });
});

test("rejects binary size, SHA-256, and native format mismatches", async (t) => {
  await t.test("size", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.executable.size_bytes += 1;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_SIZE_MISMATCH");
  });
  await t.test("hard 16 MiB maximum", () => {
    const tree = makeInstalledTree();
    fs.truncateSync(tree.binaryPath, 16 * 1024 * 1024 + 1);
    fs.chmodSync(tree.binaryPath, 0o755);
    refreshProvenanceExecutable(tree);
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_SIZE_MISMATCH");
  });
  await t.test("hash", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.executable.sha256 = "a".repeat(64);
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_HASH_MISMATCH");
  });
  await t.test("native type", () => {
    const tree = makeInstalledTree();
    fs.writeFileSync(tree.binaryPath, Buffer.alloc(64));
    fs.chmodSync(tree.binaryPath, 0o755);
    refreshProvenanceExecutable(tree);
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_TYPE_MISMATCH");
  });
  await t.test("native architecture", () => {
    const target = CATALOG.targets.find((value) => value.platform_key === "darwin-arm64");
    const tree = makeInstalledTree(target, true);
    const binary = fs.readFileSync(tree.binaryPath);
    binary.writeUInt32LE(0x01000007, 4);
    fs.writeFileSync(tree.binaryPath, binary);
    refreshProvenanceExecutable(tree);
    expectCode(() => resolveTree(tree), "CAREER_NPM_BINARY_TYPE_MISMATCH");
  });
});

test("preserves literal argv and makes shell metacharacters inert", () => {
  const tree = makeInstalledTree();
  const marker = path.join(tree.root, "must-not-exist");
  const values = [
    "plain",
    "spaces stay together",
    `; touch ${marker}`,
    `$(touch ${marker})`,
    "*?[brackets]$HOME|&<>`quote`'\"",
    "unicode-λ-東京",
  ];
  const result = runInstalledLauncher(tree, ["--test-argv", ...values]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.signal, null);
  const expected = values.map((value) => Buffer.from(value).toString("hex")).join("\n") + "\n";
  assert.equal(result.stdout, expected);
  assert.equal(fs.existsSync(marker), false);
});

test("inherits stdin/stdout/stderr and preserves the native exit code", () => {
  const tree = makeInstalledTree();
  const input = "synthetic stdin\nsecond line\n";
  const result = runInstalledLauncher(tree, ["--test-stdio-exit", "37"], { input });
  assert.equal(result.status, 37);
  assert.equal(result.signal, null);
  assert.equal(result.stdout, `stdout:${input}`);
  assert.equal(result.stderr, `stderr:${input}`);
});

test("installs cancellation handlers before spawning the native process", async () => {
  const signals = ["SIGINT", "SIGTERM", "SIGHUP"];
  const listenerCounts = new Map(signals.map((signal) => [signal, process.listenerCount(signal)]));
  const fakeChild = new EventEmitter();
  fakeChild.kill = () => true;
  let observedInstalledHandlers = false;
  const completion = launcher.launchVerifiedBinary("/synthetic/career", [], () => {
    for (const signal of signals) {
      assert.equal(process.listenerCount(signal), listenerCounts.get(signal) + 1);
    }
    observedInstalledHandlers = true;
    return fakeChild;
  });
  assert.equal(observedInstalledHandlers, true);
  fakeChild.emit("close", 0, null);
  await completion;
  for (const signal of signals) {
    assert.equal(process.listenerCount(signal), listenerCounts.get(signal));
  }
});

test(
  "propagates cancellation to the child and terminates with the same signal",
  { timeout: 10_000 },
  async (t) => {
  const tree = makeInstalledTree();
  const pidFile = path.join(tree.root, "helper.pid");
  const child = spawn(process.execPath, [tree.launcherBin, "--test-signal"], {
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, CAREER_NPM_HELPER_PID_FILE: pidFile },
  });
  let cleanupComplete = false;
  t.after(() => {
    if (!cleanupComplete) cleanupCancellationProcesses(child, pidFile);
  });
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const closed = new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code, signal) => resolve({ code, signal }));
  });
  await waitForCondition("native readiness", () => stdout.includes("READY\n"));
  await waitForCondition(
    "the launcher SIGTERM handler",
    () => linuxProcessCatchesSignal(child.pid, "SIGTERM"),
  );
  assert.equal(child.kill("SIGTERM"), true);
  const result = await closed;
  assert.deepEqual(result, { code: null, signal: "SIGTERM" }, stderr);
  assert.match(stdout, /^READY\n$/u);
  const nativePid = Number(fs.readFileSync(pidFile, "utf8"));
  await waitForCondition("the native process to exit", () => processIsGone(nativePid));
    cleanupComplete = true;
  },
);

test("reports launch failures with one stable bounded error code", async () => {
  await assert.rejects(
    launcher.launchVerifiedBinary(path.join(os.tmpdir(), "career-does-not-exist"), []),
    (error) => error.code === "CAREER_NPM_LAUNCH_FAILED",
  );
});

test("all six package templates are private, exact, lifecycle-free, and lockstep", () => {
  const launcherManifest = readJson(LAUNCHER_MANIFEST_SOURCE);
  const platformManifests = CATALOG.targets.map((target) =>
    readJson(path.join(ROOT, "platforms", target.platform_key, "package.json")),
  );
  assert.equal(launcherManifest.version, PACKAGE_VERSION);
  assert.equal(launcherManifest.private, true);
  assert.deepEqual(
    Object.keys(launcherManifest.optionalDependencies),
    CATALOG.platformPackages,
  );
  assert.deepEqual(
    launcherManifest.optionalDependencies,
    Object.fromEntries(CATALOG.platformPackages.map((name) => [name, launcherManifest.version])),
  );
  assert.deepEqual(launcherManifest.career_launcher.platform_packages, CATALOG.platformPackages);
  assert.deepEqual(launcherManifest.bin, { career: "bin/career.js" });
  assert.deepEqual(launcherManifest.engines, { node: ">=22" });
  assert.deepEqual(launcherManifest.os, ["darwin", "linux"]);
  assert.deepEqual(CATALOG.targets.map((target) => target.node_platform), [
    "darwin",
    "darwin",
    "linux",
    "linux",
    "linux",
    "linux",
  ]);
  assert.equal(launcherManifest.author, "Revaz Zakalashvili");
  assert.equal(launcherManifest.homepage, "https://github.com/revazi/career-core#readme");
  assert.deepEqual(launcherManifest.bugs, { url: "https://github.com/revazi/career-core/issues" });
  assert.deepEqual(launcherManifest.keywords, ["career", "resume", "job-search", "matching", "cli"]);
  assert.deepEqual(launcherManifest.files.slice(0, 2), ["bin/career.js", "targets.json"]);
  const lifecycleNames = [
    "preinstall",
    "install",
    "postinstall",
    "prepack",
    "prepare",
    "postpack",
    "publish",
    "postpublish",
  ];
  for (const manifest of [launcherManifest, ...platformManifests]) {
    assert.equal(manifest.private, true);
    assert.equal(manifest.scripts, undefined);
    assert.equal(manifest.dependencies, undefined);
    assert.equal(manifest.devDependencies, undefined);
    assert.equal(manifest.peerDependencies, undefined);
    for (const lifecycle of lifecycleNames) {
      assert.equal(Object.hasOwn(manifest, lifecycle), false);
    }
  }
  for (const [index, target] of CATALOG.targets.entries()) {
    const manifest = platformManifests[index];
    assert.equal(manifest.name, target.native_package);
    assert.equal(manifest.version, launcherManifest.version);
    assert.match(manifest.description, /^Internal /u);
    assert.deepEqual(manifest.os, [target.node_platform]);
    assert.deepEqual(manifest.cpu, [target.node_arch]);
    assert.deepEqual(manifest.libc, target.libc_family === null ? undefined : [target.libc_family]);
    assert.deepEqual(manifest.files, [target.executable, "provenance.json", ...PLATFORM_LICENSE_FILES]);
    assert.deepEqual(manifest.career_native, {
      schema_version: "career.npm_native_package.v2",
      platform_key: target.platform_key,
      node_platform: target.node_platform,
      node_arch: target.node_arch,
      libc_family: target.libc_family,
      rust_target: target.rust_target,
      binary_file: target.executable,
      provenance_file: "provenance.json",
      binary_format: target.binary_format,
      binary_architecture: target.binary_architecture,
      file_invariant: target.file_invariant,
      archive_mode: target.archive_mode,
      executable_mode: target.executable_mode,
      maximum_binary_size_bytes: target.maximum_binary_size_bytes,
      minimum_glibc_version: target.minimum_glibc_version,
    });
  }
  const readme = fs.readFileSync(path.join(ROOT, "career", "README.md"), "utf8");
  assert.ok(Buffer.byteLength(readme) <= 16 * 1024);
  assert.doesNotMatch(readme, /@revazi\/career-(?:darwin|linux|win32)/u);
  assert.doesNotMatch(JSON.stringify([launcherManifest, ...platformManifests]), /Provisional/u);
});

test("pi-career handoff pins the exact public package and version", () => {
  const handoff = fs.readFileSync(
    path.resolve(ROOT, "..", "docs", "pi-career-npm-handoff.md"),
    "utf8",
  );
  assert.match(
    handoff,
    new RegExp(`npx --yes --package=@revazi/career@${PACKAGE_VERSION.replaceAll(".", "\\.")} career <args>`, "u"),
  );
  assert.match(handoff, /[Nn]ever substitute `latest`/u);
});

test("launcher source has no network, dependency fallback, PATH lookup, or verification bypass", () => {
  const source = fs.readFileSync(LAUNCHER_SOURCE, "utf8");
  assert.doesNotMatch(source, /require\(["']node:(?:http|https|net|tls|dns)/u);
  assert.doesNotMatch(source, /\bfetch\s*\(/u);
  assert.doesNotMatch(source, /\bexec(?:File|Sync)?\s*\(/u);
  assert.doesNotMatch(source, /process\.env/u);
  assert.doesNotMatch(source, /\bPATH\b/u);
  assert.match(source, /require\.resolve\(`/u);
  assert.match(source, /shell:\s*false/u);
  assert.match(source, /stdio:\s*["']inherit["']/u);
  assert.doesNotMatch(source, /SKIP|BYPASS|telemetry|provider|download/u);
  assert.doesNotMatch(source, /career\.exe|pe32\+|windows_regular_non_symlink_exe/u);
});
