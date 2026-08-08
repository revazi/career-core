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
import hashlib
import json
import pathlib
import sys
import tarfile
root = pathlib.Path(sys.argv[1])
sha = sys.argv[2]
plan = pathlib.Path(sys.argv[3])
expected = [
    (10, "internal_native", "@revazi/career-darwin-arm64", "10-revazi-career-darwin-arm64-0.1.0.tgz"),
    (20, "internal_native", "@revazi/career-linux-x64-gnu", "20-revazi-career-linux-x64-gnu-0.1.0.tgz"),
    (30, "user_facing_launcher", "@revazi/career", "30-revazi-career-0.1.0.tgz"),
]
expected_entries = {item[3] for item in expected} | {"publication-manifest.json"}
entries = list(root.iterdir())
if {path.name for path in entries} != expected_entries:
    raise SystemExit("publication candidate directory allowlist mismatch")
if any(path.is_symlink() or not path.is_file() for path in entries):
    raise SystemExit("publication candidate contains a non-regular or nested entry")
manifest = json.loads((root / "publication-manifest.json").read_text(encoding="utf-8"))
if set(manifest) != {"schema_version", "source", "release", "packages"}:
    raise SystemExit("publication manifest property set mismatch")
if manifest["schema_version"] != "career.npm_publication_candidate.v1":
    raise SystemExit("publication manifest schema mismatch")
if manifest["source"] != {
    "repository": "https://github.com/revazi/career-core",
    "git_sha": sha,
    "git_ref": "refs/tags/v0.1.0",
    "git_tag": "v0.1.0",
    "git_dirty": False,
    "publication_candidate": True,
}:
    raise SystemExit("publication manifest source binding mismatch")
if manifest["release"] != {
    "version": "0.1.0",
    "node_version": "v22.19.0",
    "npm_version": "11.6.2",
    "access": "public",
    "npm_provenance": "required",
    "package_contained_sha256": "consistency_only",
    "independent_signature": "absent",
}:
    raise SystemExit("publication manifest release policy mismatch")
rows = manifest["packages"]
if not isinstance(rows, list) or len(rows) != 3:
    raise SystemExit("publication manifest package count mismatch")
lines = []
for row, (order, role, name, filename) in zip(rows, expected):
    path = root / filename
    data = path.read_bytes()
    integrity = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode("ascii")
    if row != {
        "order": order,
        "role": role,
        "name": name,
        "version": "0.1.0",
        "file": filename,
        "sha256": hashlib.sha256(data).hexdigest(),
        "integrity": integrity,
    }:
        raise SystemExit("publication manifest order/integrity mismatch")
    with tarfile.open(path, "r:gz") as archive:
        members = archive.getmembers()
        if any(not member.isfile() for member in members):
            raise SystemExit("packed package contains a non-regular entry")
        if len({member.name for member in members}) != len(members):
            raise SystemExit("packed package contains duplicate entries")
        names = {member.name for member in members}
        modes = {member.name: member.mode & 0o777 for member in members}
        package = json.load(archive.extractfile("package/package.json"))
        common = {"name", "version", "description", "license", "repository", "files", "publishConfig"}
        if role == "internal_native":
            keys = common | {"os", "cpu", "exports", "career_native"}
            if name == "@revazi/career-linux-x64-gnu":
                keys.add("libc")
            expected_names = {
                "package/package.json", "package/career", "package/provenance.json",
                "package/LICENSE-MIT", "package/LICENSE-APACHE", "package/THIRD_PARTY_NOTICES.md",
            }
            expected_modes = {entry: 0o644 for entry in expected_names}
            expected_modes["package/career"] = 0o755
            provenance = json.load(archive.extractfile("package/provenance.json"))
            if provenance.get("source") != manifest["source"]:
                raise SystemExit("packed native source provenance mismatch")
            if provenance.get("integrity") != {
                "npm_registry_integrity": "external_to_launcher_runtime",
                "package_contained_sha256": "consistency_only",
                "independent_signature": "absent",
            }:
                raise SystemExit("packed native integrity/signature policy mismatch")
        else:
            keys = common | {
                "author", "homepage", "bugs", "keywords", "engines", "bin",
                "optionalDependencies", "career_launcher",
            }
            expected_names = {
                "package/package.json", "package/bin/career.js", "package/README.md",
                "package/LICENSE-MIT", "package/LICENSE-APACHE", "package/THIRD_PARTY_NOTICES.md",
            }
            expected_modes = {entry: 0o644 for entry in expected_names}
            expected_modes["package/bin/career.js"] = 0o755
            if package.get("optionalDependencies") != {
                "@revazi/career-darwin-arm64": "0.1.0",
                "@revazi/career-linux-x64-gnu": "0.1.0",
            }:
                raise SystemExit("packed launcher optional dependency mismatch")
        if set(package) != keys:
            raise SystemExit("packed package property set mismatch")
        if names != expected_names or modes != expected_modes:
            raise SystemExit("packed package file/mode allowlist mismatch")
    if package.get("name") != name or package.get("version") != "0.1.0":
        raise SystemExit("packed package identity/version mismatch")
    if package.get("publishConfig") != {"access": "public", "provenance": True}:
        raise SystemExit("packed package publishability mismatch")
    lines.append(f"{order}\t{name}\t0.1.0\t{path}\t{integrity}")
plan.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY

registry="https://registry.npmjs.org"
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

require_registry_integrity() {
  local name="$1" version="$2" expected="$3" attempt=1 status
  while ((attempt <= 6)); do
    if matching_or_absent_version "$name" "$version" "$expected"; then
      return 0
    else
      status=$?
      [[ "$status" -eq 4 ]] || return "$status"
      if ((attempt < 6)); then
        sleep 10
        attempt=$((attempt + 1))
        continue
      fi
      fail "$name@$version did not expose exact reviewed registry integrity"
    fi
  done
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

require_registry_provenance() {
  local name="$1" version="$2" attempt=1 status
  while ((attempt <= 6)); do
    if registry_provenance_ready "$name" "$version"; then
      return 0
    else
      status=$?
      if [[ "$status" -ne 4 && "$status" -ne 5 ]]; then return "$status"; fi
      if ((attempt < 6)); then
        sleep 10
        attempt=$((attempt + 1))
        continue
      fi
      fail "$name@$version is missing valid npm registry SLSA provenance attestations"
    fi
  done
}

preflight_bootstrap_package() {
  local name="$1" version="$2" expected="$3" status
  if name_exists "$name"; then
    if matching_or_absent_version "$name" "$version" "$expected"; then
      require_registry_provenance "$name" "$version"
      return 0
    else
      status=$?
      [[ "$status" -eq 4 ]] && fail "$name exists without exact reviewed $version integrity"
      return "$status"
    fi
  else
    status=$?
    [[ "$status" -eq 4 ]] && return 0
    return "$status"
  fi
}

preflight_oidc_package() {
  local name="$1" version="$2" expected="$3" status
  if name_exists "$name"; then
    if matching_or_absent_version "$name" "$version" "$expected"; then
      require_registry_provenance "$name" "$version"
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
    require_registry_provenance "$name" "$version"
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
      require_registry_integrity "$name" "$version" "$expected"
      require_registry_provenance "$name" "$version"
      return 0
    else
      status=$?
      cat "$output"
      if matching_or_absent_version "$name" "$version" "$expected"; then
        require_registry_provenance "$name" "$version"
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
  expected_names=("@revazi/career-darwin-arm64" "@revazi/career-linux-x64-gnu" "@revazi/career")
  expected_orders=(10 20 30)
  [[ "$name" == "${expected_names[$index]}" && "$order" == "${expected_orders[$index]}" ]] || \
    fail "publish plan order mismatch"
  if [[ "$name" == "@revazi/career" ]]; then
    while IFS=$'\t' read -r native_order native_name native_version _native_file native_integrity; do
      [[ "$native_order" == "30" ]] && continue
      matching_or_absent_version "$native_name" "$native_version" "$native_integrity" || \
        fail "launcher publication is blocked until both native packages exactly match"
      require_registry_provenance "$native_name" "$native_version"
    done <"$plan"
  fi
  publish_one "$name" "$version" "$file" "$integrity"
  index=$((index + 1))
done <"$plan"
[[ "$index" -eq 3 ]] || fail "publish plan must contain exactly three rows"

printf 'npm publication mode %s completed in native-before-launcher order.\n' "$mode"
