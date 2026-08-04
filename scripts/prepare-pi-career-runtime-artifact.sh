#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/prepare-pi-career-runtime-artifact.sh \
  --output-dir <directory> [--expected-target <triple>] [--allow-dirty]

Build and verify one native career CLI archive for maintainer transfer to the
separate pi-career repository. The output directory must be outside this
checkout. Clean source is required unless --allow-dirty is used for local
script validation; dirty archives are labeled and are not import candidates.
EOF
}

fail() {
  printf 'runtime artifact preparation failed: %s\n' "$1" >&2
  exit 1
}

output_dir=""
expected_target=""
allow_dirty=false
while (($# > 0)); do
  case "$1" in
    --output-dir)
      (($# >= 2)) || fail "--output-dir requires a value"
      output_dir="$2"
      shift 2
      ;;
    --expected-target)
      (($# >= 2)) || fail "--expected-target requires a value"
      expected_target="$2"
      shift 2
      ;;
    --allow-dirty)
      allow_dirty=true
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

[[ -n "$output_dir" ]] || fail "--output-dir is required"

for command in cargo rustc git python3 tar uname; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
cd "$repository_root"

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || fail "repository root is not a Git worktree"

kernel="$(uname -s)"
machine="$(uname -m)"
case "$kernel:$machine" in
  Darwin:arm64|Darwin:aarch64)
    host_os="macos"
    host_arch="arm64"
    platform_key="darwin-arm64"
    target_triple="aarch64-apple-darwin"
    ;;
  Linux:x86_64|Linux:amd64)
    host_os="linux"
    host_arch="x86_64"
    platform_key="linux-x64-gnu"
    target_triple="x86_64-unknown-linux-gnu"
    ;;
  *)
    fail "unsupported native host: $kernel/$machine"
    ;;
esac

if [[ -n "$expected_target" && "$expected_target" != "$target_triple" ]]; then
  fail "native target mismatch: expected $expected_target, detected $target_triple"
fi

rust_host="$(rustc -vV | awk '/^host: / { print $2 }')"
[[ "$rust_host" == "$target_triple" ]] || fail "rustc host $rust_host does not match native target $target_triple"

git_sha="$(git rev-parse HEAD)"
[[ "$git_sha" =~ ^[0-9a-f]{40}$ ]] || fail "Git SHA is not a full lowercase commit identifier"

git_dirty=false
if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  if [[ "$allow_dirty" != true ]]; then
    fail "source worktree is dirty; use only a reviewed clean commit for transfer artifacts"
  fi
  git_dirty=true
fi

resolved_output_dir="$(python3 - "$output_dir" <<'PY'
import pathlib
import sys

print(pathlib.Path(sys.argv[1]).expanduser().resolve(strict=False))
PY
)"
case "$resolved_output_dir/" in
  "$repository_root/"*) fail "output directory must be outside the source checkout" ;;
esac
mkdir -p "$resolved_output_dir"
output_dir="$(cd "$resolved_output_dir" && pwd -P)"

suffix=""
if [[ "$git_dirty" == true ]]; then
  suffix="-dirty"
fi
archive_base="career-pi-runtime-${platform_key}-${target_triple}-${git_sha}${suffix}"
archive_path="$output_dir/${archive_base}.tar.gz"
[[ ! -e "$archive_path" ]] || fail "output archive already exists: $archive_path"

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-pi-runtime.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT
stage_dir="$temporary_root/stage"
result_dir="$temporary_root/results"
mkdir -p "$stage_dir" "$result_dir"

cargo build --release --locked -p career-cli --target "$target_triple"

target_root="${CARGO_TARGET_DIR:-$repository_root/target}"
if [[ "$target_root" != /* ]]; then
  target_root="$repository_root/$target_root"
fi
built_binary="$target_root/$target_triple/release/career"
[[ -f "$built_binary" && -x "$built_binary" ]] || fail "native release executable was not produced"

cp "$built_binary" "$stage_dir/career"
chmod 0755 "$stage_dir/career"
cp LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md "$stage_dir/"

run_json() {
  local output_name="$1"
  local maximum_bytes="$2"
  shift 2
  local output_path="$result_dir/$output_name.json"
  local error_path="$result_dir/$output_name.stderr"
  "$stage_dir/career" "$@" >"$output_path" 2>"$error_path"
  [[ ! -s "$error_path" ]] || fail "$output_name wrote unexpected stderr"
  python3 - "$output_path" "$maximum_bytes" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
maximum = int(sys.argv[2])
data = path.read_bytes()
if not data or len(data) > maximum:
    raise SystemExit(f"bounded JSON check failed for {path.name}")
if not data.endswith(b"\n") or data.count(b"\n") != 1:
    raise SystemExit(f"compact JSON framing failed for {path.name}")
value = json.loads(data)
if not isinstance(value, dict):
    raise SystemExit(f"JSON object required for {path.name}")
PY
}

version_output="$($stage_dir/career --version)"
[[ -n "$version_output" && ${#version_output} -le 128 ]] || fail "career --version output is empty or oversized"
[[ "$version_output" == career\ * ]] || fail "career --version output has an unexpected prefix"

run_json capabilities 131072 capabilities --format json-compact
run_json schema-catalog 131072 schema list --format json-compact
run_json job-match-schema 262144 schema export --id career.job_match.v1 --format json-compact
run_json resume-analysis 1048576 resume analyze --input - --format json-compact \
  < fixtures/resume/phase3/complete-analysis.input.json
run_json job-match 1048576 job match --input - --format json-compact \
  < fixtures/job/phase4b/complete-match.input.json

rustc_version="$(rustc --version)"
cargo_version="$(cargo --version)"
os_release="$(uname -sr)"

python3 - \
  "$stage_dir" \
  "$result_dir" \
  "$git_sha" \
  "$git_dirty" \
  "$host_os" \
  "$host_arch" \
  "$platform_key" \
  "$target_triple" \
  "$os_release" \
  "$rustc_version" \
  "$cargo_version" \
  "$version_output" \
  "${archive_base}.tar.gz" <<'PY'
import hashlib
import json
import pathlib
import sys

(
    stage_arg,
    result_arg,
    git_sha,
    git_dirty_arg,
    host_os,
    host_arch,
    platform_key,
    target_triple,
    os_release,
    rustc_version,
    cargo_version,
    career_version,
    archive_file,
) = sys.argv[1:]
stage = pathlib.Path(stage_arg)
results = pathlib.Path(result_arg)


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bounded_text(value: str, maximum: int, label: str) -> str:
    if not value or len(value) > maximum or any(ord(char) < 32 for char in value):
        raise SystemExit(f"invalid bounded metadata string: {label}")
    return value


def load_object(name: str, maximum: int) -> tuple[dict, pathlib.Path]:
    path = results / f"{name}.json"
    data = path.read_bytes()
    if not data or len(data) > maximum:
        raise SystemExit(f"invalid bounded result: {name}")
    value = json.loads(data)
    if not isinstance(value, dict):
        raise SystemExit(f"object result required: {name}")
    return value, path


def result_record(name: str, maximum: int, expected_schema: str) -> dict:
    value, path = load_object(name, maximum)
    if value.get("schema_version") != expected_schema:
        raise SystemExit(f"unexpected schema version: {name}")
    return {
        "schema_version": expected_schema,
        "size_bytes": path.stat().st_size,
        "sha256": digest(path),
    }


for value, maximum, label in (
    (git_sha, 40, "git_sha"),
    (host_os, 16, "host_os"),
    (host_arch, 16, "host_arch"),
    (platform_key, 32, "platform_key"),
    (target_triple, 64, "target_triple"),
    (os_release, 128, "os_release"),
    (rustc_version, 128, "rustc_version"),
    (cargo_version, 128, "cargo_version"),
    (career_version, 128, "career_version"),
    (archive_file, 192, "archive_file"),
):
    bounded_text(value, maximum, label)

capabilities = result_record("capabilities", 131072, "career.capabilities.v1")
schema_catalog_value, schema_catalog_path = load_object("schema-catalog", 131072)
if schema_catalog_value.get("schema_version") != "career.schema_catalog.v1":
    raise SystemExit("unexpected schema catalog version")
schemas = schema_catalog_value.get("schemas")
if not isinstance(schemas, list) or not 1 <= len(schemas) <= 100:
    raise SystemExit("schema catalog count is outside bounds")
if any(not isinstance(entry, dict) or not isinstance(entry.get("id"), str) for entry in schemas):
    raise SystemExit("schema catalog entries are invalid")
schema_catalog = {
    "schema_version": "career.schema_catalog.v1",
    "schema_count": len(schemas),
    "size_bytes": schema_catalog_path.stat().st_size,
    "sha256": digest(schema_catalog_path),
}

schema_export_value, schema_export_path = load_object("job-match-schema", 262144)
expected_schema_id = "career.job_match.v1"
expected_uri = "https://raw.githubusercontent.com/revazi/career-core/main/schemas/job-match-v1.schema.json"
if schema_export_value.get("$id") != expected_uri:
    raise SystemExit("job-match schema export has an unexpected identifier")
if expected_schema_id not in {entry["id"] for entry in schemas}:
    raise SystemExit("job-match schema is missing from the catalog")
schema_export = {
    "schema_id": expected_schema_id,
    "size_bytes": schema_export_path.stat().st_size,
    "sha256": digest(schema_export_path),
}

analysis_value, analysis_path = load_object("resume-analysis", 1048576)
expected_analysis = json.loads(pathlib.Path("fixtures/resume/phase3/complete-analysis.expected.json").read_text())
if analysis_value != expected_analysis:
    raise SystemExit("resume analysis output does not match its synthetic golden")

match_value, match_path = load_object("job-match", 1048576)
expected_match = json.loads(pathlib.Path("fixtures/job/phase4b/complete-match.expected.json").read_text())
if match_value != expected_match:
    raise SystemExit("job match output does not match its synthetic golden")

binary_path = stage / "career"
binary_size = binary_path.stat().st_size
if not 1 <= binary_size <= 16 * 1024 * 1024:
    raise SystemExit("career executable size is outside the artifact bound")

license_records = []
for name in ("LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"):
    path = stage / name
    size = path.stat().st_size
    if not 1 <= size <= 256 * 1024:
        raise SystemExit(f"license/notice size is outside bounds: {name}")
    license_records.append({"file_name": name, "size_bytes": size, "sha256": digest(path)})

metadata = {
    "schema_version": "career.pi_career_runtime_artifact.v1",
    "purpose": "maintainer_input_for_external_pi_career",
    "publication_status": "not_a_public_release",
    "source": {
        "repository": "https://github.com/revazi/career-core",
        "git_sha": git_sha,
        "git_dirty": git_dirty_arg == "true",
    },
    "build": {
        "command": [
            "cargo",
            "build",
            "--release",
            "--locked",
            "-p",
            "career-cli",
            "--target",
            target_triple,
        ],
        "profile": "release",
        "locked": True,
        "host_os": host_os,
        "host_arch": host_arch,
        "platform_key": platform_key,
        "target_triple": target_triple,
        "os_release": os_release,
        "rustc_version": rustc_version,
        "cargo_version": cargo_version,
    },
    "executable": {
        "file_name": "career",
        "version_output": career_version,
        "size_bytes": binary_size,
        "sha256": digest(binary_path),
    },
    "contract_digests": {
        "capabilities": capabilities,
        "schema_catalog": schema_catalog,
        "schema_export": schema_export,
    },
    "native_verification": [
        {
            "operation": "resume.analyze",
            "input_fixture": "fixtures/resume/phase3/complete-analysis.input.json",
            "golden_fixture": "fixtures/resume/phase3/complete-analysis.expected.json",
            "output_schema_version": "career.resume_analysis.v1",
            "output_size_bytes": analysis_path.stat().st_size,
            "output_sha256": digest(analysis_path),
        },
        {
            "operation": "job.match",
            "input_fixture": "fixtures/job/phase4b/complete-match.input.json",
            "golden_fixture": "fixtures/job/phase4b/complete-match.expected.json",
            "output_schema_version": "career.job_match.v1",
            "output_size_bytes": match_path.stat().st_size,
            "output_sha256": digest(match_path),
        },
    ],
    "licenses_and_notices": license_records,
    "package": {
        "archive_file": archive_file,
        "unsigned": True,
        "contents": [
            "career",
            "metadata.json",
            "LICENSE-MIT",
            "LICENSE-APACHE",
            "THIRD_PARTY_NOTICES.md",
        ],
    },
}

metadata_path = stage / "metadata.json"
metadata_path.write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n", encoding="utf-8")
if metadata_path.stat().st_size > 65536:
    raise SystemExit("metadata exceeds 64 KiB")
PY

export COPYFILE_DISABLE=1
tar -czf "$archive_path" -C "$stage_dir" \
  career metadata.json LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md

python3 - "$archive_path" "$git_sha" "$git_dirty" "$target_triple" "$platform_key" <<'PY'
import hashlib
import json
import pathlib
import sys
import tarfile

archive = pathlib.Path(sys.argv[1])
expected_sha = sys.argv[2]
expected_dirty = sys.argv[3] == "true"
expected_target = sys.argv[4]
expected_platform = sys.argv[5]
expected_names = [
    "career",
    "metadata.json",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "THIRD_PARTY_NOTICES.md",
]

if not 1 <= archive.stat().st_size <= 32 * 1024 * 1024:
    raise SystemExit("archive size is outside the 32 MiB bound")

with tarfile.open(archive, "r:gz") as package:
    members = package.getmembers()
    names = [member.name for member in members]
    if names != expected_names or len(set(names)) != len(names):
        raise SystemExit(f"unexpected archive contents: {names}")
    if any(not member.isfile() for member in members):
        raise SystemExit("archive entries must all be regular files")
    binary_member = package.getmember("career")
    if binary_member.mode & 0o111 == 0:
        raise SystemExit("archive executable mode is missing")
    binary = package.extractfile(binary_member).read()
    metadata = json.load(package.extractfile("metadata.json"))

if metadata.get("schema_version") != "career.pi_career_runtime_artifact.v1":
    raise SystemExit("unexpected metadata schema version")
if metadata.get("purpose") != "maintainer_input_for_external_pi_career":
    raise SystemExit("unexpected metadata purpose")
if metadata.get("publication_status") != "not_a_public_release":
    raise SystemExit("artifact must not claim public release status")
source = metadata.get("source", {})
if source.get("git_sha") != expected_sha or source.get("git_dirty") is not expected_dirty:
    raise SystemExit("metadata source provenance mismatch")
build = metadata.get("build", {})
if build.get("target_triple") != expected_target or build.get("platform_key") != expected_platform:
    raise SystemExit("metadata native target mismatch")
executable = metadata.get("executable", {})
if executable.get("size_bytes") != len(binary):
    raise SystemExit("metadata executable size mismatch")
if executable.get("sha256") != hashlib.sha256(binary).hexdigest():
    raise SystemExit("metadata executable digest mismatch")
if metadata.get("package", {}).get("contents") != expected_names:
    raise SystemExit("metadata package allowlist mismatch")
print(f"Verified runtime archive: {archive}")
PY

printf 'Prepared native runtime target: %s\n' "$target_triple"
printf 'Source commit: %s (dirty=%s)\n' "$git_sha" "$git_dirty"
printf 'Archive: %s\n' "$archive_path"
