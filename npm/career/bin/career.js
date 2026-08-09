#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");

const LAUNCHER_PACKAGE = "@revazi/career";
const TARGET_CATALOG_SCHEMA = "career.npm_target_catalog.v1";
const PROVENANCE_SCHEMA = "career.npm_native_provenance.v2";
const NATIVE_PACKAGE_SCHEMA = "career.npm_native_package.v2";
const LAUNCHER_SCHEMA = "career.npm_launcher.v2";
const TARGET_CATALOG_SHA256 = "9e56a3ca9b68799b0ff4bd52bbd2e71c2839d05a70398c5942062cb6e68032e2";
const MAX_MANIFEST_BYTES = 32 * 1024;
const MAX_CATALOG_BYTES = 64 * 1024;
const MAX_PROVENANCE_BYTES = 64 * 1024;
const HEADER_BYTES = 4 * 1024;
const EXPECTED_MODE = 0o755;
const REPOSITORY = "https://github.com/revazi/career-core";
const LAUNCHER_FILES = [
  "bin/career.js",
  "targets.json",
  "README.md",
  "LICENSE-MIT",
  "LICENSE-APACHE",
  "THIRD_PARTY_NOTICES.md",
];
const PLATFORM_LICENSE_FILES = ["LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"];
const TARGET_KEYS = [
  "platform_key",
  "rust_target",
  "node_platform",
  "node_arch",
  "libc_family",
  "native_package",
  "executable",
  "binary_format",
  "binary_architecture",
  "runner_os",
  "runner_arch",
  "maximum_binary_size_bytes",
  "file_invariant",
  "archive_mode",
  "executable_mode",
  "minimum_glibc_version",
  "provenance_requirements",
];
const PROVENANCE_REQUIREMENTS = Object.freeze({
  native_execution_required: true,
  cross_compilation_is_release_evidence: false,
  emulation_is_release_evidence: false,
  binary_format_verification_required: true,
  dynamic_import_verification_required: true,
  sha256_verification_required: true,
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

function hasExactOrderedKeys(value, expected) {
  return isPlainObject(value) && equalStringArray(Object.keys(value), expected);
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
  if (typeof value !== "string" || value.length > 32) return null;
  return /^(?:0|[1-9]\d{0,4})(?:\.(?:0|[1-9]\d{0,4})){1,3}$/u.test(value) ? value : null;
}

function readMetadataStat(filePath, missingCode, label) {
  try {
    return fs.lstatSync(filePath);
  } catch {
    fail(missingCode, `${label} is missing from the installed package.`);
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

function readRegularJsonWithBytes(filePath, maximumBytes, missingCode, invalidCode, label) {
  const stat = readMetadataStat(filePath, missingCode, label);
  validateMetadataStat(stat, maximumBytes, invalidCode, label);
  const bytes = readMetadataBytes(filePath, invalidCode, label);
  const stableLength = bytes.length === stat.size && bytes.length <= maximumBytes;
  if (!stableLength) fail(invalidCode, `${label} changed while it was being read.`);
  return { value: parseMetadataObject(bytes, invalidCode, label), bytes };
}

function readRegularJson(filePath, maximumBytes, missingCode, invalidCode, label) {
  return readRegularJsonWithBytes(filePath, maximumBytes, missingCode, invalidCode, label).value;
}

function catalogError() {
  fail(
    "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
    "The installed reviewed target catalog is missing, malformed, or unapproved.",
  );
}

function validProvenanceRequirements(value) {
  return hasExactKeys(value, Object.keys(PROVENANCE_REQUIREMENTS)) &&
    Object.entries(PROVENANCE_REQUIREMENTS).every(([key, expected]) => value[key] === expected);
}

function validCatalogTarget(target) {
  const nullableStrings = ["libc_family", "executable_mode", "minimum_glibc_version"];
  return [
    hasExactKeys(target, TARGET_KEYS),
    TARGET_KEYS.filter((key) => !nullableStrings.includes(key) && key !== "maximum_binary_size_bytes" && key !== "provenance_requirements").every(
      (key) => isBoundedString(target?.[key], 128),
    ),
    nullableStrings.every((key) => target?.[key] === null || isBoundedString(target?.[key], 64)),
    Number.isSafeInteger(target?.maximum_binary_size_bytes),
    target?.maximum_binary_size_bytes >= 1,
    target?.maximum_binary_size_bytes <= 64 * 1024 * 1024,
    validProvenanceRequirements(target?.provenance_requirements),
  ].every(Boolean);
}

function valuesAreUnique(values) {
  return new Set(values).size === values.length;
}

function validWindowsFileMapping(target) {
  return [
    target.executable === "career.exe",
    target.file_invariant === "windows_regular_non_symlink_exe",
    target.archive_mode === "0644",
    target.executable_mode === null,
  ].every(Boolean);
}

function validUnixFileMapping(target) {
  return [
    target.executable === "career",
    target.file_invariant === "unix_regular_non_symlink_mode_0755",
    target.archive_mode === "0755",
    target.executable_mode === "0755",
  ].every(Boolean);
}

function validCatalogFileMapping(target) {
  return target.node_platform === "win32"
    ? validWindowsFileMapping(target)
    : validUnixFileMapping(target);
}

function validCatalogLibcMapping(target) {
  if (target.node_platform !== "linux") return target.libc_family === null;
  return ["glibc", "musl"].includes(target.libc_family);
}

function validCatalogGlibcFloor(target) {
  if (target.libc_family !== "glibc") return target.minimum_glibc_version === null;
  return normalizedGlibcVersion(target.minimum_glibc_version) !== null;
}

function validateCatalogSemantics(targets) {
  const valid = [
    valuesAreUnique(targets.map((target) => target.platform_key)),
    valuesAreUnique(targets.map((target) => target.native_package)),
    targets.every(validCatalogFileMapping),
    targets.every(validCatalogLibcMapping),
    targets.every(validCatalogGlibcFloor),
  ].every(Boolean);
  if (!valid) catalogError();
}

function validateTargetCatalog(document, bytes) {
  const digest = crypto.createHash("sha256").update(bytes).digest("hex");
  const valid = [
    digest === TARGET_CATALOG_SHA256,
    hasExactKeys(document, ["schema_version", "targets"]),
    document.schema_version === TARGET_CATALOG_SCHEMA,
    Array.isArray(document.targets),
    document.targets?.length === 8,
    document.targets?.every(validCatalogTarget),
  ].every(Boolean);
  if (!valid) catalogError();
  validateCatalogSemantics(document.targets);
  const targets = document.targets.map((target) =>
    Object.freeze({
      ...target,
      provenance_requirements: Object.freeze({ ...target.provenance_requirements }),
    }),
  );
  return Object.freeze({
    targets: Object.freeze(targets),
    platformPackages: Object.freeze(targets.map((target) => target.native_package)),
  });
}

function loadTargetCatalog(catalogPath = path.join(__dirname, "..", "targets.json")) {
  const { value, bytes } = readRegularJsonWithBytes(
    catalogPath,
    MAX_CATALOG_BYTES,
    "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
    "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
    "Reviewed target catalog",
  );
  return validateTargetCatalog(value, bytes);
}

const MUSL_MARKER =
  /(?:^|\/)(?:ld-musl-(x86_64|aarch64)\.so\.1|libc\.musl-(x86_64|aarch64)\.so\.1)$/u;

function reportGlibcEvidence(report) {
  if (!isPlainObject(report) || !isPlainObject(report.header)) return { valid: false, version: null };
  const value = report.header.glibcVersionRuntime;
  if (value === undefined) return { valid: true, version: null };
  const version = normalizedGlibcVersion(value);
  return { valid: version !== null, version };
}

function muslMarkerArch(value) {
  if (!isBoundedString(value, 4096)) return { valid: false, arch: null };
  const match = value.match(MUSL_MARKER);
  if (match === null) return { valid: true, arch: null };
  const marker = match[1] || match[2];
  return { valid: true, arch: marker === "x86_64" ? "x64" : "arm64" };
}

function muslEvidenceForValue(value, arch) {
  const marker = muslMarkerArch(value);
  if (!marker.valid) return "invalid";
  if (marker.arch === null) return "none";
  return marker.arch === arch ? "matching" : "conflicting";
}

function strongerMuslEvidence(left, right) {
  const priority = { none: 0, matching: 1, conflicting: 2, invalid: 3 };
  return priority[right] > priority[left] ? right : left;
}

function reportMuslEvidence(report, arch) {
  const sharedObjects = isPlainObject(report) ? report.sharedObjects : null;
  if (!Array.isArray(sharedObjects)) return "invalid";
  if (sharedObjects.length > 1024) return "invalid";
  return sharedObjects
    .map((value) => muslEvidenceForValue(value, arch))
    .reduce(strongerMuslEvidence, "none");
}

function classifyLinuxLibc(glibc, musl) {
  if (!glibc.valid) return { family: "unknown", version: null };
  const classification = `${glibc.version === null}:${musl}`;
  if (classification === "true:matching") return { family: "musl", version: null };
  if (classification === "false:none") return { family: "glibc", version: glibc.version };
  return { family: "unknown", version: null };
}

function detectLinuxLibc(reportProvider = () => process.report.getReport(), arch = process.arch) {
  try {
    const report = reportProvider();
    return classifyLinuxLibc(reportGlibcEvidence(report), reportMuslEvidence(report, arch));
  } catch {
    // Unknown libc is rejected by selectTarget; detection never falls back.
    return { family: "unknown", version: null };
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
  return value === undefined ? 0 : value;
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
  const required = normalizedGlibcVersion(minimum);
  if (actual === null || required === null) return false;
  const actualParts = actual.split(".").map(Number);
  const requiredParts = required.split(".").map(Number);
  const width = Math.max(actualParts.length, requiredParts.length);
  return compareVersionParts(actualParts, requiredParts, 0, width);
}

function validGlibcRuntime(libcRuntime) {
  return normalizedGlibcVersion(libcRuntime.version) !== null;
}

function validMuslRuntime(libcRuntime) {
  return libcRuntime.version === null;
}

function validLinuxLibcRuntime(libcRuntime) {
  if (!hasExactKeys(libcRuntime, ["family", "version"])) return false;
  const validators = { glibc: validGlibcRuntime, musl: validMuslRuntime };
  const validator = validators[libcRuntime.family];
  if (typeof validator !== "function") return false;
  return validator(libcRuntime);
}

function requireKnownLinuxLibc(libcRuntime) {
  if (validLinuxLibcRuntime(libcRuntime)) return;
  fail(
    "CAREER_NPM_UNSUPPORTED_LIBC",
    "The Linux libc runtime could not be identified as an approved glibc or musl environment.",
  );
}

function requireGlibc(version, minimum) {
  if (!glibcVersionAtLeast(version, minimum)) {
    fail(
      "CAREER_NPM_UNSUPPORTED_LIBC",
      `The GNU/Linux package requires detected GNU libc ${minimum} or newer.`,
    );
  }
}

function selectTarget(platform, arch, libcRuntime, catalog = loadTargetCatalog()) {
  const hostTargets = catalog.targets.filter(
    (target) => target.node_platform === platform && target.node_arch === arch,
  );
  if (hostTargets.length === 0) rejectUnsupportedTarget();
  if (platform !== "linux") {
    if (hostTargets.length !== 1) rejectUnsupportedTarget();
    return hostTargets[0];
  }
  requireKnownLinuxLibc(libcRuntime);
  const target = hostTargets.find((candidate) => candidate.libc_family === libcRuntime.family);
  if (!target) rejectUnsupportedTarget();
  if (target.libc_family === "glibc") {
    requireGlibc(libcRuntime.version, target.minimum_glibc_version);
  }
  return target;
}

function matchesValues(value, expected) {
  if (!isPlainObject(value)) return false;
  return Object.entries(expected).every(([key, expectedValue]) => value[key] === expectedValue);
}

function matchesExactValues(value, expected) {
  return hasExactKeys(value, Object.keys(expected)) && matchesValues(value, expected);
}

function hasNoCodeFields(manifest, includeOptional) {
  const fields = ["scripts", "dependencies", "devDependencies", "peerDependencies"];
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

function validateLauncherOptionalDependencies(manifest, platformPackages) {
  const optional = manifest.optionalDependencies;
  const versionsMatch = platformPackages.every(
    (packageName) => optional && optional[packageName] === manifest.version,
  );
  if (!hasExactOrderedKeys(optional, platformPackages) || !versionsMatch) {
    fail(
      "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
      "The launcher must declare exactly eight ordered lockstep optional native packages.",
    );
  }
}

function validateLauncherMetadata(metadata, platformPackages) {
  const valid = [
    hasExactKeys(metadata, ["schema_version", "executable", "target_catalog", "platform_packages"]),
    matchesValues(metadata, {
      schema_version: LAUNCHER_SCHEMA,
      executable: "career",
      target_catalog: "targets.json",
    }),
    metadata && equalStringArray(metadata.platform_packages, platformPackages),
  ].every(Boolean);
  if (!valid) {
    fail(
      "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
      "The installed launcher metadata does not match career.npm_launcher.v2.",
    );
  }
}

function validateLauncherManifest(manifest, catalog) {
  if (manifest.name !== LAUNCHER_PACKAGE || !isSemver(manifest.version)) {
    fail(
      "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
      "The installed launcher package manifest has an invalid name or version.",
    );
  }
  validateLauncherSurface(manifest);
  validateLauncherOptionalDependencies(manifest, catalog.platformPackages);
  validateLauncherMetadata(manifest.career_launcher, catalog.platformPackages);
  return manifest.version;
}

function platformFiles(target) {
  return [target.executable, "provenance.json", ...PLATFORM_LICENSE_FILES];
}

function validPlatformLibc(manifest, target) {
  if (target.node_platform === "linux") return equalStringArray(manifest.libc, [target.libc_family]);
  return manifest.libc === undefined;
}

function validatePlatformMapping(manifest, target) {
  const valid = [
    equalStringArray(manifest.os, [target.node_platform]),
    equalStringArray(manifest.cpu, [target.node_arch]),
    validPlatformLibc(manifest, target),
    equalStringArray(manifest.files, platformFiles(target)),
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
  };
  if (!matchesExactValues(metadata, expected)) {
    fail(
      "CAREER_NPM_TARGET_MISMATCH",
      "The versioned native package metadata does not match the selected target.",
    );
  }
}

function validatePlatformManifest(manifest, target, launcherVersion) {
  if (manifest.name !== target.native_package) {
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
    "libc_family",
    "rust_target",
    "minimum_glibc_version",
  ];
  const valid = [
    hasExactKeys(packageRecord, keys),
    packageRecord?.name === target.native_package,
  ].every(Boolean);
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
    platform_key: target.platform_key,
    node_platform: target.node_platform,
    node_arch: target.node_arch,
    libc_family: target.libc_family,
    rust_target: target.rust_target,
    minimum_glibc_version: target.minimum_glibc_version,
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
  if (source.git_dirty || !source.publication_candidate) {
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

function validLocalGlibcRunner(value) {
  if (!isBoundedString(value, 128)) return false;
  return value.startsWith("glibc ");
}

function validLocalRunnerLibc(value, target) {
  if (target.libc_family === null) return value === null;
  if (target.libc_family === "musl") return value === "musl";
  return validLocalGlibcRunner(value);
}

function expectedCandidateRunnerLibc(target) {
  if (target.libc_family === null) return null;
  if (target.libc_family === "musl") return "musl";
  return `glibc ${target.minimum_glibc_version}`;
}

function validRunnerLibc(runner, target, source) {
  if (!isPlainObject(runner)) return false;
  if (source.publication_candidate) return runner.libc === expectedCandidateRunnerLibc(target);
  return validLocalRunnerLibc(runner.libc, target);
}

function validateProvenanceRunner(runner, target, source) {
  const valid = [
    hasExactKeys(runner, ["os", "arch", "image", "libc"]),
    matchesValues(runner, { os: target.runner_os, arch: target.runner_arch }),
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
    target.rust_target,
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
  if (!isPlainObject(build) || !validProvenanceBuild(build, target)) {
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
      "The provenance integrity claims exceed the reviewed npm trust model.",
    );
  }
}

function validProvenanceExecutable(executable, target) {
  const expected = {
    file_name: target.executable,
    binary_format: target.binary_format,
    binary_architecture: target.binary_architecture,
    file_invariant: target.file_invariant,
    archive_mode: target.archive_mode,
    mode: target.executable_mode,
  };
  const keys = [
    "file_name",
    "binary_format",
    "binary_architecture",
    "file_invariant",
    "archive_mode",
    "mode",
    "size_bytes",
    "sha256",
  ];
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
  if (executable.size_bytes > target.maximum_binary_size_bytes) {
    fail(
      "CAREER_NPM_BINARY_SIZE_MISMATCH",
      "The native executable size exceeds the approved package bound.",
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

function validMachOHeader(header, target) {
  if (header.length < 8 || header.readUInt32LE(0) !== 0xfeedfacf) return false;
  const cpuType = target.binary_architecture === "aarch64" ? 0x0100000c : 0x01000007;
  return header.readUInt32LE(4) === cpuType;
}

function validElfHeader(header, target) {
  if (header.length < 20) return false;
  const prefix = Buffer.from([0x7f, 0x45, 0x4c, 0x46, 2, 1]);
  const machine = target.binary_architecture === "aarch64" ? 0xb7 : 0x3e;
  return header.subarray(0, prefix.length).equals(prefix) && header.readUInt16LE(18) === machine;
}

function validPeHeader(header, target) {
  if (header.length < 64) return false;
  if (header.readUInt16LE(0) !== 0x5a4d) return false;
  const offset = header.readUInt32LE(0x3c);
  if (offset < 64) return false;
  if (offset + 26 > header.length) return false;
  const machine = target.binary_architecture === "aarch64" ? 0xaa64 : 0x8664;
  return [
    header.readUInt32LE(offset) === 0x00004550,
    header.readUInt16LE(offset + 4) === machine,
    header.readUInt16LE(offset + 24) === 0x020b,
  ].every(Boolean);
}

function verifyBinaryFormat(header, target) {
  let valid = false;
  if (target.binary_format.startsWith("mach-o-64-")) valid = validMachOHeader(header, target);
  if (target.binary_format.startsWith("elf-64-")) valid = validElfHeader(header, target);
  if (target.binary_format.startsWith("pe32+-")) valid = validPeHeader(header, target);
  if (!valid) {
    fail(
      "CAREER_NPM_BINARY_TYPE_MISMATCH",
      "The native executable format or architecture does not match the selected target.",
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
  if (stat.isSymbolicLink() || !stat.isFile()) {
    fail(
      "CAREER_NPM_BINARY_TYPE_INVALID",
      "The native executable must be a regular non-symlink file.",
    );
  }
}

function validateBinaryFileInvariant(stat, target) {
  validateBinaryFileType(stat);
  if (target.file_invariant === "unix_regular_non_symlink_mode_0755") {
    if ((stat.mode & 0o7777) !== EXPECTED_MODE) {
      fail(
        "CAREER_NPM_BINARY_MODE_MISMATCH",
        "The Unix native executable must have exact mode 0755.",
      );
    }
    return;
  }
  if (target.file_invariant !== "windows_regular_non_symlink_exe") {
    fail(
      "CAREER_NPM_BINARY_TYPE_INVALID",
      "The native executable file invariant is not approved.",
    );
  }
}

function validateBinarySize(stat, executable, target) {
  const validSize = [
    stat.size === executable.size_bytes,
    stat.size <= target.maximum_binary_size_bytes,
  ].every(Boolean);
  if (!validSize) {
    fail(
      "CAREER_NPM_BINARY_SIZE_MISMATCH",
      "The native executable size does not match bounded provenance.",
    );
  }
}

function validateInitialBinaryStat(stat, executable, target) {
  validateBinaryFileInvariant(stat, target);
  validateBinarySize(stat, executable, target);
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
  if (!opened.isFile() || !sameFileIdentity(before, opened)) {
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
  const header = Buffer.alloc(HEADER_BYTES);
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
  if (after.isSymbolicLink() || !after.isFile() || !sameFileIdentity(opened, after)) {
    fail(
      "CAREER_NPM_BINARY_CHANGED",
      "The native executable changed after integrity verification.",
    );
  }
}

function openVerifiedBinary(binaryPath, target, executable) {
  const before = readInitialBinaryStat(binaryPath);
  validateInitialBinaryStat(before, executable, target);
  const descriptor = openBinaryNoFollow(binaryPath);
  try {
    const verified = hashOpenedBinary(descriptor, before);
    const header = verified.header.subarray(0, Math.min(verified.opened.size, verified.header.length));
    verifyBinaryFormat(header, target);
    verifyBinaryDigest(verified.digest, executable.sha256);
    verifyFinalBinaryStat(binaryPath, verified.opened);
    return descriptor;
  } catch (error) {
    fs.closeSync(descriptor);
    throw error;
  }
}

function verifyBinary(binaryPath, target, executable) {
  const descriptor = openVerifiedBinary(binaryPath, target, executable);
  fs.closeSync(descriptor);
}

function defaultPackageResolver(packageName, launcherDirectory) {
  return require.resolve(`${packageName}/package.json`, { paths: [launcherDirectory] });
}

function optionValue(options, key, fallback) {
  return options[key] === undefined ? fallback : options[key];
}

function runtimeLibc(options, platform, arch) {
  if (platform !== "linux") return null;
  if (options.libcRuntime !== undefined) return options.libcRuntime;
  return detectLinuxLibc(options.reportProvider, arch);
}

function runtimeTarget(options, catalog) {
  const platform = optionValue(options, "platform", process.platform);
  const arch = optionValue(options, "arch", process.arch);
  return selectTarget(platform, arch, runtimeLibc(options, platform, arch), catalog);
}

function launcherPackagePath(options) {
  return optionValue(options, "launcherManifestPath", path.join(__dirname, "..", "package.json"));
}

function targetCatalogPath(options, launcherManifestPath) {
  return optionValue(
    options,
    "targetCatalogPath",
    path.join(path.dirname(launcherManifestPath), "targets.json"),
  );
}

function resolvePlatformManifestPath(resolver, target, launcherDirectory, launcherVersion) {
  try {
    return resolver(target.native_package, launcherDirectory);
  } catch {
    fail(
      "CAREER_NPM_PLATFORM_PACKAGE_MISSING",
      `The selected optional native package for ${LAUNCHER_PACKAGE}@${launcherVersion} is missing; reinstall with optional dependencies enabled.`,
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
  const launcherManifestPath = launcherPackagePath(options);
  const catalog = loadTargetCatalog(targetCatalogPath(options, launcherManifestPath));
  const target = runtimeTarget(options, catalog);
  const launcherManifest = readRegularJson(
    launcherManifestPath,
    MAX_MANIFEST_BYTES,
    "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
    "CAREER_NPM_LAUNCHER_MANIFEST_INVALID",
    "Launcher package manifest",
  );
  const launcherVersion = validateLauncherManifest(launcherManifest, catalog);
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
  const binaryPath = path.join(platformDirectory, target.executable);
  verifyBinary(binaryPath, target, executable);
  return { binaryPath, target, executable, launcherVersion };
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
  return childSignal || forwardedSignal;
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
  return Number.isInteger(code) ? code : 1;
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

function launchResolvedBinary(resolved, argv, spawnImplementation = spawn) {
  const descriptor = openVerifiedBinary(resolved.binaryPath, resolved.target, resolved.executable);
  try {
    return launchVerifiedBinary(resolved.binaryPath, argv, spawnImplementation);
  } finally {
    fs.closeSync(descriptor);
  }
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
    await launchResolvedBinary(resolved, process.argv.slice(2));
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
  detectLinuxLibc,
  formatLauncherError,
  launchResolvedBinary,
  launchVerifiedBinary,
  loadTargetCatalog,
  resolveVerifiedBinary,
  selectTarget,
};
