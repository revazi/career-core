#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");

const LAUNCHER_PACKAGE = "@revazi/career";
const PROVENANCE_SCHEMA = "career.npm_native_provenance.v1";
const NATIVE_PACKAGE_SCHEMA = "career.npm_native_package.v1";
const LAUNCHER_SCHEMA = "career.npm_launcher.v1";
const MAX_MANIFEST_BYTES = 32 * 1024;
const MAX_PROVENANCE_BYTES = 64 * 1024;
const MAX_BINARY_BYTES = 16 * 1024 * 1024;
const EXPECTED_MODE = 0o755;
const EXPECTED_MODE_TEXT = "0755";
const REPOSITORY = "https://github.com/revazi/career-core";
const LAUNCHER_FILES = [
  "bin/career.js",
  "README.md",
  "LICENSE-MIT",
  "LICENSE-APACHE",
  "THIRD_PARTY_NOTICES.md",
];
const PLATFORM_FILES = [
  "career",
  "provenance.json",
  "LICENSE-MIT",
  "LICENSE-APACHE",
  "THIRD_PARTY_NOTICES.md",
];
const PLATFORM_PACKAGES = [
  "@revazi/career-darwin-arm64",
  "@revazi/career-linux-x64-gnu",
];
const TARGETS = Object.freeze({
  "darwin-arm64": Object.freeze({
    platformKey: "darwin-arm64",
    nodePlatform: "darwin",
    nodeArch: "arm64",
    rustTarget: "aarch64-apple-darwin",
    packageName: "@revazi/career-darwin-arm64",
    binaryFormat: "mach-o-64-aarch64",
    minimumGlibcVersion: null,
  }),
  "linux-x64-gnu": Object.freeze({
    platformKey: "linux-x64-gnu",
    nodePlatform: "linux",
    nodeArch: "x64",
    rustTarget: "x86_64-unknown-linux-gnu",
    packageName: "@revazi/career-linux-x64-gnu",
    binaryFormat: "elf-64-x86_64",
    minimumGlibcVersion: "2.35",
  }),
});

class LauncherError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "LauncherError";
    this.code = code;
  }
}

function fail(code, message) {
  throw new LauncherError(code, message);
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function hasExactKeys(value, expected) {
  if (!isPlainObject(value)) return false;
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  return actual.length === wanted.length && actual.every((key, index) => key === wanted[index]);
}

function equalStringArray(actual, expected) {
  return (
    Array.isArray(actual) &&
    actual.length === expected.length &&
    actual.every((value, index) => value === expected[index])
  );
}

function isBoundedString(value, maximum) {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value.length <= maximum &&
    !/[\u0000-\u001f\u007f]/u.test(value)
  );
}

function isSemver(value) {
  return (
    typeof value === "string" &&
    value.length <= 64 &&
    /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/u.test(value)
  );
}

function normalizedGlibcVersion(value) {
  if (typeof value !== "string") return null;
  if (value.length > 32) return null;
  return /^(?:0|[1-9]\d{0,4})(?:\.(?:0|[1-9]\d{0,4})){1,3}$/u.test(value) ? value : null;
}

function detectGlibcRuntimeVersion(reportProvider = () => process.report.getReport()) {
  try {
    return normalizedGlibcVersion(reportProvider()?.header?.glibcVersionRuntime);
  } catch {
    // Unknown libc is rejected by selectTarget; detection never falls back.
    return null;
  }
}

function rejectUnsupportedTarget() {
  fail(
    "CAREER_NPM_UNSUPPORTED_PLATFORM",
    "No Career Core npm native package is approved for this platform and architecture.",
  );
}

function versionPart(parts, index) {
  const value = parts.at(index);
  if (value === undefined) return 0;
  return value;
}

function compareVersionParts(actual, required, index, width) {
  if (index >= width) return true;
  const left = versionPart(actual, index);
  const right = versionPart(required, index);
  if (left > right) return true;
  if (left < right) return false;
  return compareVersionParts(actual, required, index + 1, width);
}

function glibcVersionAtLeast(version, minimum) {
  const actual = normalizedGlibcVersion(version);
  if (actual === null) return false;
  const required = normalizedGlibcVersion(minimum);
  if (required === null) return false;
  const actualParts = actual.split(".").map(Number);
  const requiredParts = required.split(".").map(Number);
  const width = Math.max(actualParts.length, requiredParts.length);
  return compareVersionParts(actualParts, requiredParts, 0, width);
}

function requireGlibc(version, minimum) {
  if (!glibcVersionAtLeast(version, minimum)) {
    fail(
      "CAREER_NPM_UNSUPPORTED_LIBC",
      `The linux-x64 package requires detected GNU libc ${minimum} or newer; musl, older, malformed, and unknown libc runtimes are unsupported.`,
    );
  }
}

function selectTarget(platform, arch, glibcVersionRuntime) {
  const host = `${platform}-${arch}`;
  if (host === "darwin-arm64") return TARGETS["darwin-arm64"];
  if (host !== "linux-x64") rejectUnsupportedTarget();
  const target = TARGETS["linux-x64-gnu"];
  requireGlibc(glibcVersionRuntime, target.minimumGlibcVersion);
  return target;
}

function readMetadataStat(filePath, missingCode, label) {
  try {
    return fs.lstatSync(filePath);
  } catch {
    fail(missingCode, `${label} is missing from the installed platform package.`);
  }
}

function validateMetadataStat(stat, maximumBytes, invalidCode, label) {
  const valid = [!stat.isSymbolicLink(), stat.isFile(), stat.size >= 2, stat.size <= maximumBytes].every(
    Boolean,
  );
  if (!valid) fail(invalidCode, `${label} must be a bounded regular non-symlink file.`);
}

function readMetadataBytes(filePath, invalidCode, label) {
  try {
    return fs.readFileSync(filePath);
  } catch {
    fail(invalidCode, `${label} could not be read safely.`);
  }
}

function parseMetadataObject(bytes, invalidCode, label) {
  let value;
  try {
    value = JSON.parse(bytes.toString("utf8"));
  } catch {
    fail(invalidCode, `${label} must contain one valid JSON object.`);
  }
  if (!isPlainObject(value)) fail(invalidCode, `${label} must contain one valid JSON object.`);
  return value;
}

function readRegularJson(filePath, maximumBytes, missingCode, invalidCode, label) {
  const stat = readMetadataStat(filePath, missingCode, label);
  validateMetadataStat(stat, maximumBytes, invalidCode, label);
  const bytes = readMetadataBytes(filePath, invalidCode, label);
  const stableLength = bytes.length === stat.size && bytes.length <= maximumBytes;
  if (!stableLength) fail(invalidCode, `${label} changed while it was being read.`);
  return parseMetadataObject(bytes, invalidCode, label);
}

function matchesValues(value, expected) {
  if (!isPlainObject(value)) return false;
  return Object.entries(expected).every(([key, expectedValue]) => value[key] === expectedValue);
}

function matchesExactValues(value, expected) {
  return hasExactKeys(value, Object.keys(expected)) && matchesValues(value, expected);
}

function hasNoCodeFields(manifest, includeOptional) {
  const fields = ["scripts", "dependencies", "devDependencies"];
  if (includeOptional) fields.push("optionalDependencies");
  return fields.every((field) => manifest[field] === undefined);
}

function validateLauncherSurface(manifest) {
  const valid = [
    matchesExactValues(manifest.bin, { career: "bin/career.js" }),
    matchesExactValues(manifest.engines, { node: ">=22" }),
    equalStringArray(manifest.files, LAUNCHER_FILES),
    hasNoCodeFields(manifest, false),
  ].every(Boolean);
  if (!valid) {
    fail(
      "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
      "The launcher bin, Node engine, files, or dependency-free policy is invalid.",
    );
  }
}

function validateLauncherOptionalDependencies(manifest) {
  const optional = manifest.optionalDependencies;
  const versionsMatch = PLATFORM_PACKAGES.every(
    (packageName) => optional && optional[packageName] === manifest.version,
  );
  if (!hasExactKeys(optional, PLATFORM_PACKAGES) || !versionsMatch) {
    fail(
      "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
      "The launcher must declare exactly two lockstep optional native packages.",
    );
  }
}

function validateLauncherMetadata(metadata) {
  const valid = [
    hasExactKeys(metadata, ["schema_version", "executable", "platform_packages"]),
    matchesValues(metadata, { schema_version: LAUNCHER_SCHEMA, executable: "career" }),
    metadata && equalStringArray(metadata.platform_packages, PLATFORM_PACKAGES),
  ].every(Boolean);
  if (!valid) {
    fail(
      "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
      "The installed launcher metadata does not match career.npm_launcher.v1.",
    );
  }
}

function validateLauncherManifest(manifest) {
  if (manifest.name !== LAUNCHER_PACKAGE || !isSemver(manifest.version)) {
    fail(
      "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
      "The installed launcher package manifest has an invalid name or version.",
    );
  }
  validateLauncherSurface(manifest);
  validateLauncherOptionalDependencies(manifest);
  validateLauncherMetadata(manifest.career_launcher);
  return manifest.version;
}

function validPlatformLibc(manifest, target) {
  if (target.nodePlatform === "linux") return equalStringArray(manifest.libc, ["glibc"]);
  return manifest.libc === undefined;
}

function validatePlatformMapping(manifest, target) {
  const valid = [
    equalStringArray(manifest.os, [target.nodePlatform]),
    equalStringArray(manifest.cpu, [target.nodeArch]),
    validPlatformLibc(manifest, target),
    equalStringArray(manifest.files, PLATFORM_FILES),
  ].every(Boolean);
  if (!valid) {
    fail(
      "CAREER_NPM_TARGET_MISMATCH",
      "The native package manifest platform mapping does not match the selected target.",
    );
  }
}

function validateNativeMetadata(metadata, target) {
  const expected = {
    schema_version: NATIVE_PACKAGE_SCHEMA,
    platform_key: target.platformKey,
    node_platform: target.nodePlatform,
    node_arch: target.nodeArch,
    rust_target: target.rustTarget,
    binary_file: "career",
    provenance_file: "provenance.json",
    executable_mode: EXPECTED_MODE_TEXT,
    maximum_binary_size_bytes: MAX_BINARY_BYTES,
    minimum_glibc_version: target.minimumGlibcVersion,
  };
  if (!matchesExactValues(metadata, expected)) {
    fail(
      "CAREER_NPM_TARGET_MISMATCH",
      "The versioned native package metadata does not match the selected target.",
    );
  }
}

function validatePlatformManifest(manifest, target, launcherVersion) {
  if (manifest.name !== target.packageName) {
    fail(
      "CAREER_NPM_PLATFORM_PACKAGE_MISMATCH",
      "The resolved optional package name does not match the selected native target.",
    );
  }
  if (manifest.version !== launcherVersion) {
    fail(
      "CAREER_NPM_VERSION_MISMATCH",
      "The launcher and native platform package versions must match exactly.",
    );
  }
  if (!hasNoCodeFields(manifest, true)) {
    fail(
      "CAREER_NPM_PLATFORM_MANIFEST_INVALID",
      "The native package manifest must contain no scripts or dependencies.",
    );
  }
  validatePlatformMapping(manifest, target);
  validateNativeMetadata(manifest.career_native, target);
}

function validateProvenanceShape(provenance) {
  const keys = ["schema_version", "package", "source", "build", "executable", "integrity"];
  if (!hasExactKeys(provenance, keys) || provenance.schema_version !== PROVENANCE_SCHEMA) {
    fail(
      "CAREER_NPM_PROVENANCE_INVALID",
      "The native package provenance schema or property set is invalid.",
    );
  }
}

function validateProvenancePackageIdentity(packageRecord, target) {
  const keys = [
    "name",
    "version",
    "platform_key",
    "node_platform",
    "node_arch",
    "rust_target",
    "minimum_glibc_version",
  ];
  const valid = [hasExactKeys(packageRecord, keys), packageRecord?.name === target.packageName].every(
    Boolean,
  );
  if (!valid) {
    fail(
      "CAREER_NPM_PROVENANCE_PACKAGE_MISMATCH",
      "The provenance package identity does not match the selected optional package.",
    );
  }
}

function validateProvenancePackageVersion(packageRecord, launcherVersion) {
  if (packageRecord.version !== launcherVersion) {
    fail(
      "CAREER_NPM_VERSION_MISMATCH",
      "The launcher, platform manifest, and provenance versions must match exactly.",
    );
  }
}

function validateProvenancePackageTarget(packageRecord, target) {
  const targetValues = {
    platform_key: target.platformKey,
    node_platform: target.nodePlatform,
    node_arch: target.nodeArch,
    rust_target: target.rustTarget,
    minimum_glibc_version: target.minimumGlibcVersion,
  };
  if (!matchesValues(packageRecord, targetValues)) {
    fail(
      "CAREER_NPM_PROVENANCE_TARGET_MISMATCH",
      "The provenance target mapping does not match the selected native package.",
    );
  }
}

function validateProvenancePackage(packageRecord, target, launcherVersion) {
  validateProvenancePackageIdentity(packageRecord, target);
  validateProvenancePackageVersion(packageRecord, launcherVersion);
  validateProvenancePackageTarget(packageRecord, target);
}

function invalidProvenance(message) {
  fail("CAREER_NPM_PROVENANCE_INVALID", message);
}

function rejectDirtyCandidate(source) {
  if (source.git_dirty && source.publication_candidate) {
    fail(
      "CAREER_NPM_DIRTY_PROVENANCE",
      "A dirty source build cannot be marked as a publication candidate.",
    );
  }
}

function requireCleanCandidateWithoutPrivateGuard(source, privateGuard) {
  if (privateGuard === true) return;
  if (source.git_dirty) {
    fail(
      "CAREER_NPM_DIRTY_PROVENANCE",
      "A native package without the private guard must carry clean publication-candidate provenance.",
    );
  }
  if (!source.publication_candidate) {
    fail(
      "CAREER_NPM_DIRTY_PROVENANCE",
      "A native package without the private guard must carry clean publication-candidate provenance.",
    );
  }
}

function validatePublicationState(source, privateGuard) {
  rejectDirtyCandidate(source);
  requireCleanCandidateWithoutPrivateGuard(source, privateGuard);
}

function validSourceReleaseBinding(source, version) {
  if (source.publication_candidate) {
    return source.git_ref === `refs/tags/v${version}` && source.git_tag === `v${version}`;
  }
  return source.git_ref === null && source.git_tag === null;
}

function validateProvenanceSource(source, manifest) {
  if (!isPlainObject(source)) invalidProvenance("The native package source provenance is invalid.");
  const valid = [
    hasExactKeys(source, [
      "repository",
      "git_sha",
      "git_ref",
      "git_tag",
      "git_dirty",
      "publication_candidate",
    ]),
    source.repository === REPOSITORY,
    typeof source.git_dirty === "boolean",
    typeof source.publication_candidate === "boolean",
    /^[0-9a-f]{40}$/u.test(source.git_sha),
    validSourceReleaseBinding(source, manifest.version),
  ].every(Boolean);
  if (!valid) invalidProvenance("The native package source provenance is invalid.");
  validatePublicationState(source, manifest.private);
}

function expectedRunner(target) {
  if (target.nodePlatform === "linux") return { os: "Linux", arch: "X64" };
  return { os: "macOS", arch: "ARM64" };
}

function validLinuxRunnerLibc(runner) {
  return [isBoundedString(runner?.libc, 128), runner?.libc?.startsWith("glibc ")].every(Boolean);
}

function validCandidateRunnerLibc(runner, source) {
  if (!source.publication_candidate) return true;
  return runner.libc === "glibc 2.35";
}

function validRunnerLibc(runner, target, source) {
  if (target.nodePlatform !== "linux") return runner?.libc === null;
  return [validLinuxRunnerLibc(runner), validCandidateRunnerLibc(runner, source)].every(Boolean);
}

function validateProvenanceRunner(runner, target, source) {
  const valid = [
    hasExactKeys(runner, ["os", "arch", "image", "libc"]),
    matchesValues(runner, expectedRunner(target)),
    isBoundedString(runner?.image, 128),
    validRunnerLibc(runner, target, source),
  ].every(Boolean);
  if (!valid) invalidProvenance("The native package runner provenance is invalid.");
}

function expectedBuildCommand(target) {
  return [
    "cargo",
    "build",
    "--release",
    "--locked",
    "-p",
    "career-cli",
    "--target",
    target.rustTarget,
  ];
}

function validProvenanceBuild(build, target) {
  return [
    hasExactKeys(build, [
      "command",
      "profile",
      "locked",
      "rustc_version",
      "cargo_version",
      "runner",
    ]),
    equalStringArray(build.command, expectedBuildCommand(target)),
    matchesValues(build, { profile: "release", locked: true }),
    isBoundedString(build.rustc_version, 128),
    isBoundedString(build.cargo_version, 128),
  ].every(Boolean);
}

function validCandidateToolchain(build, source) {
  if (!source.publication_candidate) return true;
  return [
    build.rustc_version.startsWith("rustc 1.97.1 "),
    build.cargo_version.startsWith("cargo 1.97.1 "),
  ].every(Boolean);
}

function validateProvenanceBuild(build, target, source) {
  if (!isPlainObject(build)) invalidProvenance("The native package build provenance is invalid.");
  if (!validProvenanceBuild(build, target)) {
    invalidProvenance("The native package build provenance is invalid.");
  }
  if (!validCandidateToolchain(build, source)) {
    invalidProvenance("The publication candidate toolchain is invalid.");
  }
  validateProvenanceRunner(build.runner, target, source);
}

function validateProvenanceIntegrity(integrity) {
  const expected = {
    npm_registry_integrity: "external_to_launcher_runtime",
    package_contained_sha256: "consistency_only",
    independent_signature: "absent",
  };
  if (!matchesExactValues(integrity, expected)) {
    fail(
      "CAREER_NPM_PROVENANCE_INVALID",
      "The provenance integrity claims exceed the Phase 9 trust model.",
    );
  }
}

function validProvenanceExecutable(executable, target) {
  const expected = {
    file_name: "career",
    binary_format: target.binaryFormat,
    mode: EXPECTED_MODE_TEXT,
  };
  const keys = ["file_name", "binary_format", "mode", "size_bytes", "sha256"];
  return [
    hasExactKeys(executable, keys),
    matchesValues(executable, expected),
    Number.isSafeInteger(executable?.size_bytes),
    executable?.size_bytes >= 1,
    /^[0-9a-f]{64}$/u.test(executable?.sha256),
  ].every(Boolean);
}

function validateProvenanceExecutable(executable, target) {
  if (!validProvenanceExecutable(executable, target)) {
    invalidProvenance("The native executable provenance is invalid.");
  }
  if (executable.size_bytes > MAX_BINARY_BYTES) {
    fail(
      "CAREER_NPM_BINARY_SIZE_MISMATCH",
      "The native executable size exceeds the 16 MiB package bound.",
    );
  }
  return executable;
}

function validateProvenance(provenance, manifest, target, launcherVersion) {
  validateProvenanceShape(provenance);
  validateProvenancePackage(provenance.package, target, launcherVersion);
  validateProvenanceSource(provenance.source, manifest);
  validateProvenanceBuild(provenance.build, target, provenance.source);
  validateProvenanceIntegrity(provenance.integrity);
  return validateProvenanceExecutable(provenance.executable, target);
}

function validMachOHeader(header) {
  if (header.length < 8) return false;
  return [header.readUInt32LE(0) === 0xfeedfacf, header.readUInt32LE(4) === 0x0100000c].every(
    Boolean,
  );
}

function validElfHeader(header) {
  if (header.length < 20) return false;
  const prefix = Buffer.from([0x7f, 0x45, 0x4c, 0x46, 2, 1]);
  return [header.subarray(0, prefix.length).equals(prefix), header.readUInt16LE(18) === 0x3e].every(
    Boolean,
  );
}

function verifyBinaryFormat(header, target) {
  if (target.binaryFormat === "mach-o-64-aarch64") {
    if (!validMachOHeader(header)) {
      fail(
        "CAREER_NPM_BINARY_TYPE_MISMATCH",
        "The native executable is not a 64-bit Apple ARM Mach-O binary.",
      );
    }
    return;
  }
  if (!validElfHeader(header)) {
    fail(
      "CAREER_NPM_BINARY_TYPE_MISMATCH",
      "The native executable is not a 64-bit little-endian x86-64 ELF binary.",
    );
  }
}

function sameFileIdentity(left, right) {
  return ["dev", "ino", "size", "mode", "mtimeMs"].every((field) => left[field] === right[field]);
}

function readInitialBinaryStat(binaryPath) {
  try {
    return fs.lstatSync(binaryPath);
  } catch {
    fail(
      "CAREER_NPM_BINARY_MISSING",
      "The native executable is missing from the installed platform package.",
    );
  }
}

function validateBinaryFileType(stat) {
  const regular = [!stat.isSymbolicLink(), stat.isFile()].every(Boolean);
  if (!regular) {
    fail(
      "CAREER_NPM_BINARY_TYPE_INVALID",
      "The native executable must be a regular non-symlink file.",
    );
  }
}

function validateBinaryMode(stat) {
  if ((stat.mode & 0o7777) !== EXPECTED_MODE) {
    fail(
      "CAREER_NPM_BINARY_MODE_MISMATCH",
      "The native executable must have exact Unix mode 0755.",
    );
  }
}

function validateBinarySize(stat, executable) {
  const validSize = [stat.size === executable.size_bytes, stat.size <= MAX_BINARY_BYTES].every(Boolean);
  if (!validSize) {
    fail(
      "CAREER_NPM_BINARY_SIZE_MISMATCH",
      "The native executable size does not match bounded provenance.",
    );
  }
}

function validateInitialBinaryStat(stat, executable) {
  validateBinaryFileType(stat);
  validateBinaryMode(stat);
  validateBinarySize(stat, executable);
}

function openBinaryNoFollow(binaryPath) {
  try {
    return fs.openSync(binaryPath, fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW || 0));
  } catch {
    fail(
      "CAREER_NPM_BINARY_TYPE_INVALID",
      "The native executable could not be opened as a regular non-symlink file.",
    );
  }
}

function validateOpenedBinary(before, opened) {
  const unchanged = [opened.isFile(), sameFileIdentity(before, opened)].every(Boolean);
  if (!unchanged) {
    fail(
      "CAREER_NPM_BINARY_CHANGED",
      "The native executable changed before integrity verification completed.",
    );
  }
}

function requireReadProgress(count) {
  if (count <= 0) {
    fail(
      "CAREER_NPM_BINARY_SIZE_MISMATCH",
      "The native executable ended before its declared size.",
    );
  }
}

function copyHeaderChunk(header, buffer, count, offset) {
  if (offset >= header.length) return;
  buffer.copy(header, offset, 0, Math.min(count, header.length - offset));
}

function hashOpenedBinary(descriptor, before) {
  const opened = fs.fstatSync(descriptor);
  validateOpenedBinary(before, opened);
  const hash = crypto.createHash("sha256");
  const buffer = Buffer.alloc(64 * 1024);
  const header = Buffer.alloc(20);
  let offset = 0;
  while (offset < opened.size) {
    const wanted = Math.min(buffer.length, opened.size - offset);
    const count = fs.readSync(descriptor, buffer, 0, wanted, offset);
    requireReadProgress(count);
    hash.update(buffer.subarray(0, count));
    copyHeaderChunk(header, buffer, count, offset);
    offset += count;
  }
  return { opened, digest: hash.digest(), header };
}

function verifyBinaryDigest(digest, expectedHex) {
  const expected = Buffer.from(expectedHex, "hex");
  const matches = digest.length === expected.length && crypto.timingSafeEqual(digest, expected);
  if (!matches) {
    fail(
      "CAREER_NPM_BINARY_HASH_MISMATCH",
      "The native executable SHA-256 does not match package-contained provenance.",
    );
  }
}

function verifyFinalBinaryStat(binaryPath, opened) {
  let after;
  try {
    after = fs.lstatSync(binaryPath);
  } catch {
    fail(
      "CAREER_NPM_BINARY_CHANGED",
      "The native executable changed after integrity verification.",
    );
  }
  const stable = [!after.isSymbolicLink(), after.isFile(), sameFileIdentity(opened, after)].every(
    Boolean,
  );
  if (!stable) {
    fail(
      "CAREER_NPM_BINARY_CHANGED",
      "The native executable changed after integrity verification.",
    );
  }
}

function verifyBinary(binaryPath, target, executable) {
  const before = readInitialBinaryStat(binaryPath);
  validateInitialBinaryStat(before, executable);
  const descriptor = openBinaryNoFollow(binaryPath);
  let verified;
  try {
    verified = hashOpenedBinary(descriptor, before);
  } finally {
    fs.closeSync(descriptor);
  }
  const header = verified.header.subarray(0, Math.min(verified.opened.size, verified.header.length));
  verifyBinaryFormat(header, target);
  verifyBinaryDigest(verified.digest, executable.sha256);
  verifyFinalBinaryStat(binaryPath, verified.opened);
}

function defaultPackageResolver(packageName, launcherDirectory) {
  return require.resolve(`${packageName}/package.json`, { paths: [launcherDirectory] });
}

function optionValue(options, key, fallback) {
  if (options[key] === undefined) return fallback;
  return options[key];
}

function runtimeGlibcVersion(options, platform) {
  if (platform !== "linux") return null;
  if (options.glibcVersionRuntime !== undefined) return options.glibcVersionRuntime;
  return detectGlibcRuntimeVersion(options.reportProvider);
}

function runtimeTarget(options) {
  const platform = optionValue(options, "platform", process.platform);
  const arch = optionValue(options, "arch", process.arch);
  return selectTarget(platform, arch, runtimeGlibcVersion(options, platform));
}

function launcherPackagePath(options) {
  return optionValue(options, "launcherManifestPath", path.join(__dirname, "..", "package.json"));
}

function resolvePlatformManifestPath(resolver, target, launcherDirectory, launcherVersion) {
  try {
    return resolver(target.packageName, launcherDirectory);
  } catch {
    fail(
      "CAREER_NPM_PLATFORM_PACKAGE_MISSING",
      `Optional package ${target.packageName}@${launcherVersion} is missing; reinstall ${LAUNCHER_PACKAGE}@${launcherVersion} with optional dependencies enabled.`,
    );
  }
}

function readPlatformManifest(platformManifestPath, target, launcherVersion) {
  const manifest = readRegularJson(
    platformManifestPath,
    MAX_MANIFEST_BYTES,
    "CAREER_NPM_PLATFORM_PACKAGE_MISSING",
    "CAREER_NPM_PLATFORM_MANIFEST_INVALID",
    "Native package manifest",
  );
  validatePlatformManifest(manifest, target, launcherVersion);
  return manifest;
}

function readNativeProvenance(platformDirectory) {
  return readRegularJson(
    path.join(platformDirectory, "provenance.json"),
    MAX_PROVENANCE_BYTES,
    "CAREER_NPM_PROVENANCE_MISSING",
    "CAREER_NPM_PROVENANCE_INVALID",
    "Native package provenance",
  );
}

function resolveVerifiedBinary(options = {}) {
  const target = runtimeTarget(options);
  const launcherManifestPath = launcherPackagePath(options);
  const launcherManifest = readRegularJson(
    launcherManifestPath,
    MAX_MANIFEST_BYTES,
    "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
    "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
    "Launcher package manifest",
  );
  const launcherVersion = validateLauncherManifest(launcherManifest);
  const launcherDirectory = path.dirname(launcherManifestPath);
  const resolver = optionValue(options, "packageResolver", defaultPackageResolver);
  const platformManifestPath = resolvePlatformManifestPath(
    resolver,
    target,
    launcherDirectory,
    launcherVersion,
  );
  const platformManifest = readPlatformManifest(platformManifestPath, target, launcherVersion);
  const platformDirectory = path.dirname(platformManifestPath);
  const provenance = readNativeProvenance(platformDirectory);
  const executable = validateProvenance(provenance, platformManifest, target, launcherVersion);
  const binaryPath = path.join(platformDirectory, "career");
  verifyBinary(binaryPath, target, executable);
  return { binaryPath, target, launcherVersion };
}

function launchError() {
  return new LauncherError(
    "CAREER_NPM_LAUNCH_FAILED",
    "The verified native executable could not be launched.",
  );
}

function spawnVerifiedChild(state, binaryPath, argv, spawnImplementation) {
  try {
    state.child = spawnImplementation(binaryPath, argv, { shell: false, stdio: "inherit" });
    return true;
  } catch {
    state.settled = true;
    removeSignalHandlers(state);
    state.reject(launchError());
    return false;
  }
}

function removeSignalHandlers(state) {
  for (const [signal, handler] of state.handlers) process.removeListener(signal, handler);
}

function forwardSignal(state, signal) {
  if (state.forwardedSignal === null) state.forwardedSignal = signal;
  if (state.child === null) return;
  try {
    state.child.kill(signal);
  } catch {
    // The close/error event remains authoritative.
  }
}

function installSignalHandlers(state) {
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
    const handler = () => forwardSignal(state, signal);
    state.handlers.set(signal, handler);
    process.on(signal, handler);
  }
}

function handleChildError(state) {
  if (state.settled) return;
  state.settled = true;
  removeSignalHandlers(state);
  state.reject(launchError());
}

function selectedSignal(childSignal, forwardedSignal) {
  if (childSignal) return childSignal;
  return forwardedSignal;
}

function terminateLauncher(signal, resolve) {
  try {
    process.kill(process.pid, signal);
  } catch {
    process.exitCode = 1;
    resolve();
  }
}

function normalExitCode(code) {
  if (Number.isInteger(code)) return code;
  return 1;
}

function handleChildClose(state, code, childSignal) {
  if (state.settled) return;
  state.settled = true;
  removeSignalHandlers(state);
  const signal = selectedSignal(childSignal, state.forwardedSignal);
  if (signal) {
    terminateLauncher(signal, state.resolve);
    return;
  }
  process.exitCode = normalExitCode(code);
  state.resolve();
}

function runVerifiedChild(binaryPath, argv, spawnImplementation, resolve, reject) {
  const state = {
    child: null,
    forwardedSignal: null,
    handlers: new Map(),
    reject,
    resolve,
    settled: false,
  };
  installSignalHandlers(state);
  if (!spawnVerifiedChild(state, binaryPath, argv, spawnImplementation)) return;
  state.child.once("error", () => handleChildError(state));
  state.child.once("close", (code, signal) => handleChildClose(state, code, signal));
  if (state.forwardedSignal !== null) forwardSignal(state, state.forwardedSignal);
}

function launchVerifiedBinary(binaryPath, argv, spawnImplementation = spawn) {
  return new Promise((resolve, reject) =>
    runVerifiedChild(binaryPath, argv, spawnImplementation, resolve, reject),
  );
}

function formatLauncherError(error) {
  const safe =
    error instanceof LauncherError
      ? error
      : new LauncherError(
          "CAREER_NPM_LAUNCH_FAILED",
          "The npm launcher failed before the native CLI could start.",
        );
  return `career npm launcher [${safe.code}]: ${safe.message}\n`;
}

async function main() {
  try {
    const resolved = resolveVerifiedBinary();
    await launchVerifiedBinary(resolved.binaryPath, process.argv.slice(2));
  } catch (error) {
    process.stderr.write(formatLauncherError(error));
    process.exitCode = 1;
  }
}

if (require.main === module) {
  void main();
}

module.exports = {
  LauncherError,
  TARGETS,
  detectGlibcRuntimeVersion,
  formatLauncherError,
  launchVerifiedBinary,
  resolveVerifiedBinary,
  selectTarget,
};
