#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
prepare_script="$script_dir/prepare-pi-career-runtime-artifact.sh"

case "$(uname -s):$(uname -m)" in
  Darwin:arm64|Darwin:aarch64)
    expected_platform="darwin-arm64"
    expected_target="aarch64-apple-darwin"
    wrong_target="x86_64-unknown-linux-gnu"
    ;;
  Linux:x86_64|Linux:amd64)
    expected_platform="linux-x64-gnu"
    expected_target="x86_64-unknown-linux-gnu"
    wrong_target="aarch64-apple-darwin"
    ;;
  *)
    printf 'runtime artifact test requires an approved native host\n' >&2
    exit 1
    ;;
esac

for command in git python3 tar; do
  command -v "$command" >/dev/null 2>&1 || {
    printf 'runtime artifact test requires %s\n' "$command" >&2
    exit 1
  }
done

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-pi-runtime-test.XXXXXX")"
inside_output="$repository_root/.runtime-artifact-test-inside-$$"
inside_link="$temporary_root/repository-link"
inside_argument="$inside_link/$(basename "$inside_output")"
trap 'rm -rf "$temporary_root" "$inside_output"' EXIT
output_dir="$temporary_root/output"
mkdir -p "$output_dir"
ln -s "$repository_root" "$inside_link"

[[ ! -e "$inside_output" ]]
if "$prepare_script" \
  --output-dir "$inside_argument" \
  --expected-target "$expected_target" \
  --allow-dirty >"$temporary_root/inside.stdout" 2>"$temporary_root/inside.stderr"; then
  printf 'runtime artifact test expected in-checkout output rejection\n' >&2
  exit 1
fi
grep -Fq 'output directory must be outside the source checkout' "$temporary_root/inside.stderr"
[[ ! -e "$inside_output" ]] || {
  printf 'runtime artifact test found an in-checkout output side effect\n' >&2
  exit 1
}

if [[ -n "$(git -C "$repository_root" status --porcelain --untracked-files=normal)" ]]; then
  if "$prepare_script" \
    --output-dir "$output_dir" \
    --expected-target "$expected_target" \
    >"$temporary_root/dirty.stdout" 2>"$temporary_root/dirty.stderr"; then
    printf 'runtime artifact test expected dirty transfer rejection\n' >&2
    exit 1
  fi
  grep -Fq 'source worktree is dirty' "$temporary_root/dirty.stderr"
fi

if "$prepare_script" \
  --output-dir "$output_dir" \
  --expected-target "$wrong_target" \
  --allow-dirty >"$temporary_root/wrong.stdout" 2>"$temporary_root/wrong.stderr"; then
  printf 'runtime artifact test expected target mismatch rejection\n' >&2
  exit 1
fi
grep -Fq 'native target mismatch' "$temporary_root/wrong.stderr"

"$prepare_script" \
  --output-dir "$output_dir" \
  --expected-target "$expected_target" \
  --allow-dirty

archive_count="$(find "$output_dir" -maxdepth 1 -type f -name '*.tar.gz' | wc -l | tr -d ' ')"
[[ "$archive_count" == "1" ]] || {
  printf 'runtime artifact test expected exactly one archive\n' >&2
  exit 1
}
archive="$(find "$output_dir" -maxdepth 1 -type f -name '*.tar.gz' -print)"

extract_dir="$temporary_root/extracted"
mkdir -p "$extract_dir"
tar -xzf "$archive" -C "$extract_dir"
"$extract_dir/career" capabilities --format json-compact \
  >"$temporary_root/capabilities.json" 2>"$temporary_root/capabilities.stderr"
"$extract_dir/career" schema list --format json-compact \
  >"$temporary_root/schema-catalog.json" 2>"$temporary_root/schema-catalog.stderr"
[[ ! -s "$temporary_root/capabilities.stderr" && ! -s "$temporary_root/schema-catalog.stderr" ]]

expected_sha="$(git -C "$repository_root" rev-parse HEAD)"
expected_dirty=false
if [[ -n "$(git -C "$repository_root" status --porcelain --untracked-files=normal)" ]]; then
  expected_dirty=true
fi

python3 - \
  "$extract_dir" \
  "$temporary_root/capabilities.json" \
  "$temporary_root/schema-catalog.json" \
  "$expected_sha" \
  "$expected_dirty" \
  "$expected_platform" \
  "$expected_target" <<'PY'
import hashlib
import json
import pathlib
import re
import sys

(
    extract_arg,
    capabilities_arg,
    catalog_arg,
    expected_sha,
    expected_dirty_arg,
    expected_platform,
    expected_target,
) = sys.argv[1:]
extract = pathlib.Path(extract_arg)
metadata = json.loads((extract / "metadata.json").read_text())
capabilities = pathlib.Path(capabilities_arg).read_bytes()
catalog = pathlib.Path(catalog_arg).read_bytes()

assert metadata["schema_version"] == "career.pi_career_runtime_artifact.v1"
assert metadata["source"] == {
    "repository": "https://github.com/revazi/career-core",
    "git_sha": expected_sha,
    "git_dirty": expected_dirty_arg == "true",
}
assert metadata["build"]["platform_key"] == expected_platform
assert metadata["build"]["target_triple"] == expected_target
assert metadata["package"]["unsigned"] is True
assert metadata["package"]["contents"] == [
    "career",
    "metadata.json",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "THIRD_PARTY_NOTICES.md",
]
assert [entry["operation"] for entry in metadata["native_verification"]] == [
    "resume.analyze",
    "job.match",
]

binary = (extract / "career").read_bytes()
assert metadata["executable"]["size_bytes"] == len(binary)
assert metadata["executable"]["sha256"] == hashlib.sha256(binary).hexdigest()

for key, data in (("capabilities", capabilities), ("schema_catalog", catalog)):
    record = metadata["contract_digests"][key]
    assert record["size_bytes"] == len(data)
    assert record["sha256"] == hashlib.sha256(data).hexdigest()
    assert re.fullmatch(r"[0-9a-f]{64}", record["sha256"])

assert json.loads(capabilities)["schema_version"] == "career.capabilities.v1"
assert json.loads(catalog)["schema_version"] == "career.schema_catalog.v1"
print("Runtime artifact shell/metadata test passed.")
PY
