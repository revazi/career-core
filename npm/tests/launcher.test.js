"use strict";

const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");
const { after, before, test } = require("node:test");

const ROOT = path.resolve(__dirname, "..");
const LAUNCHER_SOURCE = path.join(ROOT, "career", "bin", "career.js");
const LAUNCHER_MANIFEST_SOURCE = path.join(ROOT, "career", "package.json");
const HELPER_SOURCE = path.join(__dirname, "fixtures", "native-helper.rs");
const launcher = require(LAUNCHER_SOURCE);
const temporaryRoots = [];
let helperBinary;

function makeTemporaryRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  temporaryRoots.push(root);
  return root;
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

function currentTarget() {
  const key = `${process.platform}-${process.arch}`;
  const target = {
    "darwin-arm64": launcher.TARGETS["darwin-arm64"],
    "linux-x64": launcher.TARGETS["linux-x64-gnu"],
  }[key];
  assert.ok(target, "npm launcher tests require an approved native host");
  return target;
}

function hostOptions() {
  return {
    platform: process.platform,
    arch: process.arch,
    glibcVersionRuntime: process.platform === "linux" ? "2.35" : null,
  };
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

function binaryFormat(target) {
  return target.platformKey === "darwin-arm64" ? "mach-o-64-aarch64" : "elf-64-x86_64";
}

function provenanceFor(platformDirectory, target, version = "0.1.0") {
  const binaryPath = path.join(platformDirectory, "career");
  return {
    schema_version: "career.npm_native_provenance.v1",
    package: {
      name: target.packageName,
      version,
      platform_key: target.platformKey,
      node_platform: target.nodePlatform,
      node_arch: target.nodeArch,
      rust_target: target.rustTarget,
      minimum_glibc_version: target.minimumGlibcVersion,
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
        target.rustTarget,
      ],
      profile: "release",
      locked: true,
      rustc_version: "rustc 1.85.0 (synthetic test)",
      cargo_version: "cargo 1.85.0 (synthetic test)",
      runner: {
        os: target.nodePlatform === "linux" ? "Linux" : "macOS",
        arch: target.nodePlatform === "linux" ? "X64" : "ARM64",
        image: "synthetic-test",
        libc: target.nodePlatform === "linux" ? "glibc 2.35" : null,
      },
    },
    executable: {
      file_name: "career",
      binary_format: binaryFormat(target),
      mode: "0755",
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

function makeInstalledTree() {
  const root = makeTemporaryRoot("career-npm-tree-");
  const target = currentTarget();
  const launcherDirectory = path.join(root, "node_modules", "@revazi", "career");
  const platformDirectory = path.join(
    root,
    "node_modules",
    "@revazi",
    target.packageName.slice("@revazi/".length),
  );
  fs.mkdirSync(path.join(launcherDirectory, "bin"), { recursive: true });
  fs.mkdirSync(platformDirectory, { recursive: true });
  fs.copyFileSync(LAUNCHER_MANIFEST_SOURCE, path.join(launcherDirectory, "package.json"));
  fs.copyFileSync(LAUNCHER_SOURCE, path.join(launcherDirectory, "bin", "career.js"));
  fs.copyFileSync(path.join(ROOT, "career", "README.md"), path.join(launcherDirectory, "README.md"));
  fs.copyFileSync(
    path.join(ROOT, "platforms", target.platformKey, "package.json"),
    path.join(platformDirectory, "package.json"),
  );
  fs.copyFileSync(helperBinary, path.join(platformDirectory, "career"));
  fs.chmodSync(path.join(platformDirectory, "career"), 0o755);
  writeJson(
    path.join(platformDirectory, "provenance.json"),
    provenanceFor(platformDirectory, target),
  );
  return {
    root,
    target,
    launcherDirectory,
    launcherManifestPath: path.join(launcherDirectory, "package.json"),
    launcherBin: path.join(launcherDirectory, "bin", "career.js"),
    platformDirectory,
    platformManifestPath: path.join(platformDirectory, "package.json"),
    provenancePath: path.join(platformDirectory, "provenance.json"),
    binaryPath: path.join(platformDirectory, "career"),
  };
}

function resolveTree(tree, overrides = {}) {
  return launcher.resolveVerifiedBinary({
    ...hostOptions(),
    launcherManifestPath: tree.launcherManifestPath,
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

test("selects only the two approved native targets and fails closed on libc", () => {
  assert.equal(
    launcher.selectTarget("darwin", "arm64", null).packageName,
    "@revazi/career-darwin-arm64",
  );
  assert.equal(
    launcher.selectTarget("linux", "x64", "2.35").packageName,
    "@revazi/career-linux-x64-gnu",
  );
  expectCode(
    () => launcher.selectTarget("linux", "x64", "2.34"),
    "CAREER_NPM_UNSUPPORTED_LIBC",
  );
  expectCode(
    () => launcher.selectTarget("linux", "x64", "2.35-malformed"),
    "CAREER_NPM_UNSUPPORTED_LIBC",
  );
  expectCode(
    () => launcher.selectTarget("linux", "x64", "musl"),
    "CAREER_NPM_UNSUPPORTED_LIBC",
  );
  expectCode(
    () => launcher.selectTarget("linux", "x64", null),
    "CAREER_NPM_UNSUPPORTED_LIBC",
  );
  expectCode(
    () => launcher.selectTarget("linux", "arm64", "2.35"),
    "CAREER_NPM_UNSUPPORTED_PLATFORM",
  );
  expectCode(
    () => launcher.selectTarget("darwin", "x64", null),
    "CAREER_NPM_UNSUPPORTED_PLATFORM",
  );
  expectCode(
    () => launcher.selectTarget("win32", "x64", null),
    "CAREER_NPM_UNSUPPORTED_PLATFORM",
  );
});

test("glibc detection accepts only a bounded runtime report value", () => {
  assert.equal(
    launcher.detectGlibcRuntimeVersion(() => ({ header: { glibcVersionRuntime: "2.36" } })),
    "2.36",
  );
  assert.equal(
    launcher.detectGlibcRuntimeVersion(() => ({ header: { glibcVersionRuntime: "musl" } })),
    null,
  );
  assert.equal(launcher.detectGlibcRuntimeVersion(() => ({ header: {} })), null);
  assert.equal(
    launcher.detectGlibcRuntimeVersion(() => {
      throw new Error("synthetic report failure");
    }),
    null,
  );
});

test("resolves and verifies a complete package-local native install", () => {
  const tree = makeInstalledTree();
  const result = resolveTree(tree);
  assert.equal(result.binaryPath, tree.binaryPath);
  assert.equal(result.target.platformKey, tree.target.platformKey);
  assert.equal(result.launcherVersion, "0.1.0");
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

test("rejects launcher optional-dependency drift and lifecycle scripts", async (t) => {
  await t.test("optional dependency version", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.launcherManifestPath, (manifest) => {
      manifest.optionalDependencies["@revazi/career-linux-x64-gnu"] = "0.1.1";
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
      manifest.version = "0.1.1";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_VERSION_MISMATCH");
  });
  await t.test("target metadata", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.platformManifestPath, (manifest) => {
      manifest.career_native.rust_target = "x86_64-pc-windows-msvc";
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
      value.schema_version = "career.npm_native_provenance.v2";
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
      value.package.version = "0.1.1";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_VERSION_MISMATCH");
  });
  await t.test("target", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.package.rust_target = "x86_64-pc-windows-msvc";
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_TARGET_MISMATCH");
  });
  await t.test("dirty publication candidate", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.source.git_ref = "refs/tags/v0.1.0";
      value.source.git_tag = "v0.1.0";
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
      value.source.git_ref = "refs/tags/v0.1.0";
      value.source.git_tag = "v0.1.0";
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
      value.source.git_ref = "refs/tags/v0.1.1";
      value.source.git_tag = "v0.1.1";
      value.source.publication_candidate = true;
    });
    expectCode(() => resolveTree(tree), "CAREER_NPM_PROVENANCE_INVALID");
  });
  await t.test("runner target mismatch", () => {
    const tree = makeInstalledTree();
    mutateJson(tree.provenancePath, (value) => {
      value.build.runner.os = tree.target.nodePlatform === "linux" ? "macOS" : "Linux";
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
  await t.test("mode", () => {
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

test("propagates cancellation to the child and terminates with the same signal", { timeout: 10_000 }, async () => {
  const tree = makeInstalledTree();
  const pidFile = path.join(tree.root, "helper.pid");
  const child = spawn(process.execPath, [tree.launcherBin, "--test-signal"], {
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, CAREER_NPM_HELPER_PID_FILE: pidFile },
  });
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
    if (stdout.includes("READY")) child.kill("SIGTERM");
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const result = await new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code, signal) => resolve({ code, signal }));
  });
  assert.deepEqual(result, { code: null, signal: "SIGTERM" }, stderr);
  assert.match(stdout, /^READY\n$/u);
  const nativePid = Number(fs.readFileSync(pidFile, "utf8"));
  await new Promise((resolve) => setTimeout(resolve, 50));
  assert.throws(() => process.kill(nativePid, 0), /ESRCH/u);
});

test("reports launch failures with one stable bounded error code", async () => {
  await assert.rejects(
    launcher.launchVerifiedBinary(path.join(os.tmpdir(), "career-does-not-exist"), []),
    (error) => error.code === "CAREER_NPM_LAUNCH_FAILED",
  );
});

test("package templates are private, dependency-free, lifecycle-free, and exactly lockstep", () => {
  const launcherManifest = readJson(LAUNCHER_MANIFEST_SOURCE);
  const darwin = readJson(path.join(ROOT, "platforms", "darwin-arm64", "package.json"));
  const linux = readJson(path.join(ROOT, "platforms", "linux-x64-gnu", "package.json"));
  assert.equal(launcherManifest.private, true);
  assert.deepEqual(launcherManifest.optionalDependencies, {
    "@revazi/career-darwin-arm64": launcherManifest.version,
    "@revazi/career-linux-x64-gnu": launcherManifest.version,
  });
  assert.deepEqual(launcherManifest.bin, { career: "bin/career.js" });
  assert.deepEqual(launcherManifest.engines, { node: ">=22" });
  assert.equal(launcherManifest.author, "Revaz Zakalashvili");
  assert.equal(launcherManifest.homepage, "https://github.com/revazi/career-core#readme");
  assert.deepEqual(launcherManifest.bugs, { url: "https://github.com/revazi/career-core/issues" });
  assert.deepEqual(launcherManifest.keywords, ["career", "resume", "job-search", "matching", "cli"]);
  assert.ok(launcherManifest.files.includes("README.md"));
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
  for (const manifest of [launcherManifest, darwin, linux]) {
    assert.equal(manifest.private, true);
    assert.equal(manifest.scripts, undefined);
    assert.equal(manifest.dependencies, undefined);
    assert.equal(manifest.devDependencies, undefined);
    for (const lifecycle of lifecycleNames) {
      assert.equal(Object.hasOwn(manifest, lifecycle), false);
    }
  }
  assert.equal(darwin.name, "@revazi/career-darwin-arm64");
  assert.equal(linux.name, "@revazi/career-linux-x64-gnu");
  assert.match(darwin.description, /^Internal /u);
  assert.match(linux.description, /^Internal /u);
  assert.deepEqual(linux.libc, ["glibc"]);
  assert.equal(darwin.career_native.minimum_glibc_version, null);
  assert.equal(linux.career_native.minimum_glibc_version, "2.35");
  const readme = fs.readFileSync(path.join(ROOT, "career", "README.md"), "utf8");
  assert.ok(Buffer.byteLength(readme) <= 16 * 1024);
  assert.doesNotMatch(readme, /@revazi\/career-(?:darwin|linux)/u);
  assert.doesNotMatch(JSON.stringify([launcherManifest, darwin, linux]), /Provisional/u);
});

test("pi-career handoff pins the exact public package and version", () => {
  const handoff = fs.readFileSync(
    path.resolve(ROOT, "..", "docs", "pi-career-npm-handoff.md"),
    "utf8",
  );
  assert.match(
    handoff,
    /npx --yes --package=@revazi\/career@0\.1\.0 career <args>/u,
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
});
