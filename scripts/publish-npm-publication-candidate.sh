#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'npm publication failed: %s\n' "$1" >&2
  exit 1
}

mode=""
candidate_dir=""
reviewed_sha=""
while (($# > 0)); do
  case "$1" in
    --mode) (($# >= 2)) || fail "--mode requires a value"; mode="$2"; shift 2 ;;
    --candidate-dir) (($# >= 2)) || fail "--candidate-dir requires a value"; candidate_dir="$2"; shift 2 ;;
    --reviewed-sha) (($# >= 2)) || fail "--reviewed-sha requires a value"; reviewed_sha="$2"; shift 2 ;;
    --help|-h)
      printf 'Usage: %s --mode <bootstrap|oidc> --candidate-dir <directory> --reviewed-sha <sha>\n' "$0"
      exit 0
      ;;
    *) fail "unknown argument: $1" ;;
  esac
done

[[ "$mode" == "bootstrap" || "$mode" == "oidc" ]] || fail "mode must be bootstrap or oidc"
[[ -d "$candidate_dir" ]] || fail "candidate directory is missing"
[[ "$reviewed_sha" =~ ^[0-9a-f]{40}$ ]] || fail "reviewed SHA is invalid"
for command in node npm python3; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done
[[ "$(node --version)" == "v22.19.0" ]] || fail "publication requires exact Node v22.19.0"
[[ "$(npm --version)" == "11.6.2" ]] || fail "publication requires exact npm 11.6.2"
if [[ "$mode" == "bootstrap" ]]; then
  [[ -n "${NODE_AUTH_TOKEN:-}" ]] || fail "bootstrap requires the temporary granular NPM_TOKEN"
else
  [[ -z "${NODE_AUTH_TOKEN:-}" ]] || fail "OIDC mode forbids token authentication"
  unset NODE_AUTH_TOKEN
fi

candidate_dir="$(cd "$candidate_dir" && pwd -P)"
temporary_root="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/career-npm-publish.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT
plan="$temporary_root/publish-plan.tsv"
userconfig="$temporary_root/npmrc"
umask 077
if [[ "$mode" == "bootstrap" ]]; then
  printf '%s\n' \
    'registry=https://registry.npmjs.org/' \
    '//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}' >"$userconfig"
else
  printf '%s\n' 'registry=https://registry.npmjs.org/' >"$userconfig"
fi
chmod 0600 "$userconfig"
export NPM_CONFIG_USERCONFIG="$userconfig"

python3 - "$candidate_dir" "$reviewed_sha" "$plan" <<'PY'
import base64
import gzip
import hashlib
import io
import json
import os
import pathlib
import stat
import sys
import tarfile

MAX_TARBALL_BYTES = 32 * 1024 * 1024
MAX_UNCOMPRESSED_TARBALL_BYTES = 20 * 1024 * 1024
MAX_BINARY_BYTES = 16 * 1024 * 1024
MAX_MANIFEST_BYTES = 32 * 1024
MAX_METADATA_BYTES = 64 * 1024
MAX_SOURCE_BYTES = 256 * 1024
MAX_README_BYTES = 16 * 1024


def same_file(left, right):
    return (left.st_dev, left.st_ino, left.st_size, left.st_mode, left.st_mtime_ns) == (
        right.st_dev, right.st_ino, right.st_size, right.st_mode, right.st_mtime_ns
    )


def bounded_file_bytes(path, maximum, label):
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or not 1 <= before.st_size <= maximum:
        raise SystemExit(f"{label} is not a bounded regular file")
    with path.open("rb") as handle:
        opened = os.fstat(handle.fileno())
        if not stat.S_ISREG(opened.st_mode) or not same_file(before, opened):
            raise SystemExit(f"{label} changed before reading")
        data = handle.read(maximum + 1)
    after = path.lstat()
    if len(data) != opened.st_size or len(data) > maximum or not same_file(opened, after):
        raise SystemExit(f"{label} changed or exceeded its bound while reading")
    return data


class BoundedDecompressedReader:
    def __init__(self, source, maximum):
        self.source = source
        self.maximum = maximum
        self.total = 0

    def read(self, size=-1):
        remaining = self.maximum - self.total
        wanted = remaining + 1 if size < 0 else min(size, remaining + 1)
        data = self.source.read(wanted)
        self.total += len(data)
        if self.total > self.maximum:
            raise SystemExit("packed package exceeds its uncompressed bound")
        return data


def read_tarball(data, limits):
    files = {}
    modes = {}
    try:
        with gzip.GzipFile(fileobj=io.BytesIO(data), mode="rb") as decompressed:
            bounded = BoundedDecompressedReader(decompressed, MAX_UNCOMPRESSED_TARBALL_BYTES)
            with tarfile.open(fileobj=bounded, mode="r|") as archive:
                for member in archive:
                    if not member.isfile() or member.name not in limits:
                        raise SystemExit("packed package member allowlist mismatch")
                    if member.name in files:
                        raise SystemExit("packed package contains duplicate entries")
                    maximum = limits[member.name]
                    if not 1 <= member.size <= maximum:
                        raise SystemExit("packed package member exceeds its reviewed bound")
                    extracted = archive.extractfile(member)
                    if extracted is None:
                        raise SystemExit("packed package member is unreadable")
                    value = extracted.read(maximum + 1)
                    if len(value) != member.size or len(value) > maximum:
                        raise SystemExit("packed package member is truncated or oversized")
                    files[member.name] = value
                    modes[member.name] = member.mode & 0o777
    except (OSError, tarfile.TarError) as error:
        raise SystemExit("packed package is malformed") from error
    if set(files) != set(limits):
        raise SystemExit("packed package member count or allowlist mismatch")
    return files, modes


root = pathlib.Path(sys.argv[1])
sha = sys.argv[2]
plan = pathlib.Path(sys.argv[3])
expected = [
    (10, "internal_native", "@revazi/career-darwin-arm64", "10-revazi-career-darwin-arm64-0.1.1.tgz"),
    (20, "internal_native", "@revazi/career-darwin-x64", "20-revazi-career-darwin-x64-0.1.1.tgz"),
    (30, "internal_native", "@revazi/career-linux-x64-gnu", "30-revazi-career-linux-x64-gnu-0.1.1.tgz"),
    (40, "internal_native", "@revazi/career-linux-arm64-gnu", "40-revazi-career-linux-arm64-gnu-0.1.1.tgz"),
    (50, "internal_native", "@revazi/career-linux-x64-musl", "50-revazi-career-linux-x64-musl-0.1.1.tgz"),
    (60, "internal_native", "@revazi/career-linux-arm64-musl", "60-revazi-career-linux-arm64-musl-0.1.1.tgz"),
    (70, "internal_native", "@revazi/career-win32-x64-msvc", "70-revazi-career-win32-x64-msvc-0.1.1.tgz"),
    (80, "internal_native", "@revazi/career-win32-arm64-msvc", "80-revazi-career-win32-arm64-msvc-0.1.1.tgz"),
    (90, "user_facing_launcher", "@revazi/career", "90-revazi-career-0.1.1.tgz"),
]
expected_entries = {item[3] for item in expected} | {"publication-manifest.json"}
entries = []
for path in root.iterdir():
    if len(entries) >= len(expected_entries):
        raise SystemExit("publication candidate directory exceeds its entry bound")
    entries.append(path)
if {path.name for path in entries} != expected_entries:
    raise SystemExit("publication candidate directory allowlist mismatch")
if any(path.is_symlink() or not path.is_file() for path in entries):
    raise SystemExit("publication candidate contains a non-regular or nested entry")
manifest = json.loads(
    bounded_file_bytes(
        root / "publication-manifest.json", MAX_METADATA_BYTES, "publication manifest"
    ).decode("utf-8")
)
if not isinstance(manifest, dict):
    raise SystemExit("publication manifest must be one JSON object")
if set(manifest) != {"schema_version", "source", "release", "packages"}:
    raise SystemExit("publication manifest property set mismatch")
if manifest["schema_version"] != "career.npm_publication_candidate.v1":
    raise SystemExit("publication manifest schema mismatch")
if manifest["source"] != {
    "repository": "https://github.com/revazi/career-core",
    "git_sha": sha,
    "git_ref": "refs/tags/v0.1.1",
    "git_tag": "v0.1.1",
    "git_dirty": False,
    "publication_candidate": True,
}:
    raise SystemExit("publication manifest source binding mismatch")
if manifest["release"] != {
    "version": "0.1.1",
    "node_version": "v22.19.0",
    "npm_version": "11.6.2",
    "access": "public",
    "npm_provenance": "required",
    "package_contained_sha256": "consistency_only",
    "independent_signature": "absent",
}:
    raise SystemExit("publication manifest release policy mismatch")
rows = manifest["packages"]
if not isinstance(rows, list) or len(rows) != len(expected):
    raise SystemExit("publication manifest package count mismatch")
platform_names = [name for _, role, name, _ in expected if role == "internal_native"]
lines = []
for row, (order, role, name, filename) in zip(rows, expected):
    path = root / filename
    data = bounded_file_bytes(path, MAX_TARBALL_BYTES, "candidate tarball")
    integrity = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode("ascii")
    if row != {
        "order": order,
        "role": role,
        "name": name,
        "version": "0.1.1",
        "file": filename,
        "sha256": hashlib.sha256(data).hexdigest(),
        "integrity": integrity,
    }:
        raise SystemExit("publication manifest order/integrity mismatch")
    common = {"name", "version", "description", "license", "repository", "files", "publishConfig"}
    if role == "internal_native":
        keys = common | {"os", "cpu", "exports", "career_native"}
        if "-linux-" in name:
            keys.add("libc")
        executable = "career.exe" if "-win32-" in name else "career"
        executable_mode = 0o644 if executable.endswith(".exe") else 0o755
        expected_names = {
            "package/package.json", f"package/{executable}", "package/provenance.json",
            "package/LICENSE-MIT", "package/LICENSE-APACHE", "package/THIRD_PARTY_NOTICES.md",
        }
        limits = {entry: MAX_SOURCE_BYTES for entry in expected_names}
        limits["package/package.json"] = MAX_MANIFEST_BYTES
        limits["package/provenance.json"] = MAX_METADATA_BYTES
        limits[f"package/{executable}"] = MAX_BINARY_BYTES
    else:
        keys = common | {
            "author", "homepage", "bugs", "keywords", "engines", "bin",
            "optionalDependencies", "career_launcher",
        }
        expected_names = {
            "package/package.json", "package/bin/career.js", "package/targets.json", "package/README.md",
            "package/LICENSE-MIT", "package/LICENSE-APACHE", "package/THIRD_PARTY_NOTICES.md",
        }
        limits = {entry: MAX_SOURCE_BYTES for entry in expected_names}
        limits["package/package.json"] = MAX_MANIFEST_BYTES
        limits["package/targets.json"] = MAX_METADATA_BYTES
        limits["package/README.md"] = MAX_README_BYTES
    files, modes = read_tarball(data, limits)
    try:
        package = json.loads(files["package/package.json"].decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise SystemExit("packed package manifest is invalid") from error
    if not isinstance(package, dict):
        raise SystemExit("packed package manifest must be one JSON object")
    if role == "internal_native":
        expected_modes = {entry: 0o644 for entry in expected_names}
        expected_modes[f"package/{executable}"] = executable_mode
        try:
            provenance = json.loads(files["package/provenance.json"].decode("utf-8"))
        except (UnicodeError, json.JSONDecodeError) as error:
            raise SystemExit("packed native provenance is invalid") from error
        if not isinstance(provenance, dict):
            raise SystemExit("packed native provenance must be one JSON object")
        if provenance.get("schema_version") != "career.npm_native_provenance.v2":
            raise SystemExit("packed native provenance schema mismatch")
        if provenance.get("source") != manifest["source"]:
            raise SystemExit("packed native source provenance mismatch")
        if provenance.get("integrity") != {
            "npm_registry_integrity": "external_to_launcher_runtime",
            "package_contained_sha256": "consistency_only",
            "independent_signature": "absent",
        }:
            raise SystemExit("packed native integrity/signature policy mismatch")
    else:
        expected_modes = {entry: 0o644 for entry in expected_names}
        expected_modes["package/bin/career.js"] = 0o755
        expected_optional = {package_name: "0.1.1" for package_name in platform_names}
        optional = package.get("optionalDependencies")
        if optional != expected_optional or list(optional) != list(expected_optional):
            raise SystemExit("packed launcher optional dependency order mismatch")
        if package.get("career_launcher", {}).get("platform_packages") != platform_names:
            raise SystemExit("packed launcher platform package order mismatch")
    if set(package) != keys:
        raise SystemExit("packed package property set mismatch")
    if modes != expected_modes:
        raise SystemExit("packed package file/mode allowlist mismatch")
    if package.get("name") != name or package.get("version") != "0.1.1":
        raise SystemExit("packed package identity/version mismatch")
    if package.get("publishConfig") != {"access": "public", "provenance": True}:
        raise SystemExit("packed package publishability mismatch")
    lines.append(f"{order}\t{name}\t0.1.1\t{path}\t{integrity}")
plan.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY

registry="https://registry.npmjs.org"
registry_visibility_attempts=61
registry_visibility_sleep_seconds=10
lookup() {
  local spec="$1" field="$2"
  local output="$temporary_root/npm-view.json" errors="$temporary_root/npm-view.stderr"
  if npm view "$spec" "$field" --json --registry="$registry" >"$output" 2>"$errors"; then
    cat "$output"
    return 0
  else
    local status=$?
    if grep -q 'E404' "$errors"; then return 4; fi
    printf 'npm registry preflight failed for an expected public package\n' >&2
    head -c 512 "$errors" >&2 || true
    return "$status"
  fi
}

name_exists() {
  local name="$1" ignored status
  if ignored="$(lookup "$name" name)"; then
    return 0
  else
    status=$?
    [[ "$status" -eq 4 ]] && return 4
    return "$status"
  fi
}

version_integrity() {
  local name="$1" version="$2" raw status
  if raw="$(lookup "$name@$version" dist.integrity)"; then
    node -e '
      const fs = require("node:fs");
      const value = JSON.parse(fs.readFileSync(0, "utf8"));
      if (typeof value !== "string") process.exit(1);
      process.stdout.write(value);
    ' <<<"$raw"
    return 0
  else
    status=$?
    [[ "$status" -eq 4 ]] && return 4
    return "$status"
  fi
}

matching_or_absent_version() {
  local name="$1" version="$2" expected="$3" actual status
  if actual="$(version_integrity "$name" "$version")"; then
    [[ "$actual" == "$expected" ]] || fail "$name@$version conflicts with reviewed tarball integrity"
    return 0
  else
    status=$?
    [[ "$status" -eq 4 ]] && return 4
    return "$status"
  fi
}

historical_v010_integrity() {
  case "$1" in
    @revazi/career-darwin-arm64)
      printf '%s\n' 'sha512-2h+TLqrZx+UfSb7pYxhZjZLxImAaUjERgHvlGZ/OJDe2rxFrOvBbvwFHA4iiyeU6Qkd+XeOhqKcBUYPvLC9lWQ=='
      ;;
    @revazi/career-linux-x64-gnu)
      printf '%s\n' 'sha512-e/EwBLqAWJyOy9/q1+BK/5dCuC6c554sWBfDKMvevWhQM+ymD9qniTWKhExEpFXrCHlpAUpdHw9z5uWx2FvMuA=='
      ;;
    @revazi/career)
      printf '%s\n' 'sha512-pyH821D9QsWTxbMXYit35+Yl8EdIiaaqpjUh8+CyJc2urE48de6Gh4POLUL4EnP0zJZe4efxtHVfDMyD5kJivg=='
      ;;
    *) return 4 ;;
  esac
}

registry_provenance_ready() {
  local name="$1" version="$2" raw status
  if raw="$(lookup "$name@$version" dist.attestations)"; then
    node -e '
      const fs = require("node:fs");
      const value = JSON.parse(fs.readFileSync(0, "utf8"));
      const url = value?.url;
      const predicate = value?.provenance?.predicateType;
      if (predicate !== "https://slsa.dev/provenance/v1") process.exit(1);
      if (typeof url !== "string" || url.length < 1 || url.length > 512) process.exit(1);
      const parsed = new URL(url);
      if (parsed.protocol !== "https:" || parsed.hostname !== "registry.npmjs.org") process.exit(1);
    ' <<<"$raw" || return 5
    return 0
  else
    status=$?
    [[ "$status" -eq 4 ]] && return 4
    return "$status"
  fi
}

require_registry_package_ready() {
  local name="$1" version="$2" expected="$3" attempt=1 status missing="integrity"
  while ((attempt <= registry_visibility_attempts)); do
    if matching_or_absent_version "$name" "$version" "$expected"; then
      missing="provenance"
      if registry_provenance_ready "$name" "$version"; then
        return 0
      else
        status=$?
        if [[ "$status" -ne 4 && "$status" -ne 5 ]]; then return "$status"; fi
      fi
    else
      status=$?
      [[ "$status" -eq 4 ]] || return "$status"
      missing="integrity"
    fi
    if ((attempt < registry_visibility_attempts)); then
      sleep "$registry_visibility_sleep_seconds"
      attempt=$((attempt + 1))
      continue
    fi
    if [[ "$missing" == "integrity" ]]; then
      fail "$name@$version did not expose exact reviewed registry integrity"
    fi
    fail "$name@$version is missing valid npm registry SLSA provenance attestations"
  done
}

require_historical_v010_package() {
  local name="$1" expected status
  if ! expected="$(historical_v010_integrity "$name")"; then
    fail "existing package name has no reviewed historical release: $name"
  fi
  if matching_or_absent_version "$name" "0.1.0" "$expected"; then
    if ! registry_provenance_ready "$name" "0.1.0"; then
      fail "$name@0.1.0 is missing valid reviewed npm registry SLSA provenance"
    fi
    return 0
  else
    status=$?
    [[ "$status" -eq 4 ]] && fail "$name is missing its exact reviewed historical 0.1.0 release"
    return "$status"
  fi
}

preflight_bootstrap_package() {
  local name="$1" version="$2" expected="$3" status
  if name_exists "$name"; then
    if matching_or_absent_version "$name" "$version" "$expected"; then
      require_registry_package_ready "$name" "$version" "$expected"
      return 0
    else
      status=$?
      [[ "$status" -eq 4 ]] || return "$status"
    fi
    require_historical_v010_package "$name"
    return 0
  else
    status=$?
    if [[ "$status" -eq 4 ]]; then
      if historical_v010_integrity "$name" >/dev/null; then
        fail "reviewed historical package name is unexpectedly absent: $name"
      fi
      return 0
    fi
    return "$status"
  fi
}

preflight_oidc_package() {
  local name="$1" version="$2" expected="$3" status
  if name_exists "$name"; then
    if matching_or_absent_version "$name" "$version" "$expected"; then
      require_registry_package_ready "$name" "$version" "$expected"
      return 0
    else
      status=$?
      [[ "$status" -eq 4 ]] && return 0
      return "$status"
    fi
  else
    status=$?
    [[ "$status" -eq 4 ]] && fail "OIDC mode requires an existing package with an exact trusted publisher: $name"
    return "$status"
  fi
}

while IFS=$'\t' read -r _order name version _file integrity; do
  if [[ "$mode" == "bootstrap" ]]; then
    preflight_bootstrap_package "$name" "$version" "$integrity"
  else
    preflight_oidc_package "$name" "$version" "$integrity"
  fi
done <"$plan"

publish_one() {
  local name="$1" version="$2" file="$3" expected="$4"
  local status attempt=1 output
  if matching_or_absent_version "$name" "$version" "$expected"; then
    require_registry_package_ready "$name" "$version" "$expected"
    printf '%s@%s already exists with exact reviewed integrity and registry provenance; skipping\n' "$name" "$version"
    return 0
  else
    status=$?
    [[ "$status" -eq 4 ]] || return "$status"
  fi
  while ((attempt <= 3)); do
    output="$temporary_root/npm-publish-$attempt.log"
    if npm publish "$file" \
      --access public \
      --provenance \
      --ignore-scripts \
      --registry="$registry" >"$output" 2>&1; then
      cat "$output"
      require_registry_package_ready "$name" "$version" "$expected"
      return 0
    else
      status=$?
      cat "$output"
      if matching_or_absent_version "$name" "$version" "$expected"; then
        require_registry_package_ready "$name" "$version" "$expected"
        printf '%s@%s appeared with exact integrity and registry provenance after an interrupted response\n' "$name" "$version"
        return 0
      else
        local lookup_status=$?
        [[ "$lookup_status" -eq 4 ]] || return "$lookup_status"
      fi
      if grep -qiE 'E401|E403|ENEEDAUTH|trusted publish|permission|provenance.*required' "$output"; then
        fail "publication authentication/provenance failed; no fallback is permitted"
      fi
      if ((attempt < 3)) && grep -qiE 'E429|429 Too Many|50[234]|timed out|network|socket hang up' "$output"; then
        sleep 20
        attempt=$((attempt + 1))
        continue
      fi
      return "$status"
    fi
  done
}

index=0
while IFS=$'\t' read -r order name version file integrity; do
  expected_names=(
    "@revazi/career-darwin-arm64"
    "@revazi/career-darwin-x64"
    "@revazi/career-linux-x64-gnu"
    "@revazi/career-linux-arm64-gnu"
    "@revazi/career-linux-x64-musl"
    "@revazi/career-linux-arm64-musl"
    "@revazi/career-win32-x64-msvc"
    "@revazi/career-win32-arm64-msvc"
    "@revazi/career"
  )
  expected_orders=(10 20 30 40 50 60 70 80 90)
  [[ "$name" == "${expected_names[$index]}" && "$order" == "${expected_orders[$index]}" ]] || \
    fail "publish plan order mismatch"
  if [[ "$name" == "@revazi/career" ]]; then
    while IFS=$'\t' read -r native_order native_name native_version _native_file native_integrity; do
      [[ "$native_order" == "90" ]] && continue
      require_registry_package_ready "$native_name" "$native_version" "$native_integrity" || \
        fail "launcher publication is blocked until all eight native packages exactly match"
    done <"$plan"
  fi
  publish_one "$name" "$version" "$file" "$integrity"
  index=$((index + 1))
done <"$plan"
[[ "$index" -eq 9 ]] || fail "publish plan must contain exactly nine rows"

printf 'npm publication mode %s completed in native-before-launcher order.\n' "$mode"
