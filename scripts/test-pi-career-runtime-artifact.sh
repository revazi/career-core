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
"$extract_dir/career" operations --format json-compact \
  >"$temporary_root/operations.json" 2>"$temporary_root/operations.stderr"
"$extract_dir/career" schema list --format json-compact \
  >"$temporary_root/schema-catalog.json" 2>"$temporary_root/schema-catalog.stderr"
"$extract_dir/career" schema export --id career.operation_catalog.v1 --format json-compact \
  >"$temporary_root/operation-catalog-schema.json" 2>"$temporary_root/operation-catalog-schema.stderr"
bundle_dir="$temporary_root/bundles"
mkdir -p "$bundle_dir"
for schema_id in \
  career.resume_analysis_replacement_review_input.v1 \
  career.job_match_input.v1 \
  career.resume_variant_review_input.v1 \
  career.resume_variant_materialization_input.v1
do
  "$extract_dir/career" schema bundle --id "$schema_id" --format json-compact \
    >"$bundle_dir/$schema_id.json" 2>"$bundle_dir/$schema_id.stderr"
  [[ ! -s "$bundle_dir/$schema_id.stderr" ]]
done
[[ ! -s "$temporary_root/capabilities.stderr" ]]
[[ ! -s "$temporary_root/operations.stderr" ]]
[[ ! -s "$temporary_root/schema-catalog.stderr" ]]
[[ ! -s "$temporary_root/operation-catalog-schema.stderr" ]]

expected_sha="$(git -C "$repository_root" rev-parse HEAD)"
expected_dirty=false
if [[ -n "$(git -C "$repository_root" status --porcelain --untracked-files=normal)" ]]; then
  expected_dirty=true
fi

python3 - \
  "$extract_dir" \
  "$temporary_root/capabilities.json" \
  "$temporary_root/operations.json" \
  "$temporary_root/schema-catalog.json" \
  "$temporary_root/operation-catalog-schema.json" \
  "$bundle_dir" \
  "$expected_sha" \
  "$expected_dirty" \
  "$expected_platform" \
  "$expected_target" <<'PY'
import copy
import hashlib
import json
import pathlib
import re
import sys

(
    extract_arg,
    capabilities_arg,
    operations_arg,
    catalog_arg,
    operation_schema_arg,
    bundle_dir_arg,
    expected_sha,
    expected_dirty_arg,
    expected_platform,
    expected_target,
) = sys.argv[1:]
extract = pathlib.Path(extract_arg)
metadata = json.loads((extract / "metadata.json").read_text())
capabilities = pathlib.Path(capabilities_arg).read_bytes()
operations = pathlib.Path(operations_arg).read_bytes()
catalog = pathlib.Path(catalog_arg).read_bytes()
operation_schema = pathlib.Path(operation_schema_arg).read_bytes()
bundle_dir = pathlib.Path(bundle_dir_arg)

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

capabilities_value = json.loads(capabilities)
operations_value = json.loads(operations)
assert capabilities_value["schema_version"] == "career.capabilities.v1"
assert operations_value["schema_version"] == "career.operation_catalog.v1"
assert json.loads(catalog)["schema_version"] == "career.schema_catalog.v1"
assert json.loads(operation_schema)["$id"].endswith("/operation-catalog-v1.schema.json")


def validate_compatibility(document):
    compatibility = document.get("managed_adapter_compatibility", {})
    assert compatibility.get("schema_version") == "career.pi_career_managed_adapter_compatibility.v1"
    assert compatibility.get("core_version") == operations_value["core_version"]

    catalog_record = compatibility.get("operation_catalog", {})
    assert catalog_record.get("schema_version") == "career.operation_catalog.v1"
    assert catalog_record.get("schema_size_bytes") == len(operation_schema)
    assert catalog_record.get("schema_sha256") == hashlib.sha256(operation_schema).hexdigest()
    assert catalog_record.get("catalog_size_bytes") == len(operations)
    assert catalog_record.get("catalog_sha256") == hashlib.sha256(operations).hexdigest()

    available_ids = [
        entry["id"]
        for entry in capabilities_value["capabilities"]
        if entry["status"] == "available"
    ]
    operation_entries = operations_value["operations"]
    expected_mapping = [
        {"capability_id": entry["capability_id"], "operation_id": entry["operation_id"]}
        for entry in operation_entries
        if entry["capability_id"] is not None
    ]
    assert [entry["capability_id"] for entry in expected_mapping] == available_ids
    assert compatibility.get("available_capability_operation_mapping") == expected_mapping

    expected_bounds = [
        {
            "operation_id": entry["operation_id"],
            "maximum_successful_machine_output_bytes": entry[
                "maximum_successful_machine_output_bytes"
            ],
        }
        for entry in operation_entries
    ]
    assert compatibility.get("declared_operation_output_bounds") == expected_bounds
    assert all(
        entry["maximum_successful_machine_output_bytes"] == 33554432
        for entry in expected_bounds
    )

    expected_bundle_ids = [
        "career.resume_analysis_replacement_review_input.v1",
        "career.job_match_input.v1",
        "career.resume_variant_review_input.v1",
        "career.resume_variant_materialization_input.v1",
    ]
    bundle_records = compatibility.get("representative_schema_bundles")
    assert isinstance(bundle_records, list)
    assert [entry["schema_id"] for entry in bundle_records] == expected_bundle_ids
    for entry in bundle_records:
        bundle = (bundle_dir / f"{entry['schema_id']}.json").read_bytes()
        assert entry["size_bytes"] == len(bundle)
        assert entry["sha256"] == hashlib.sha256(bundle).hexdigest()
        assert re.fullmatch(r"[0-9a-f]{64}", entry["sha256"])

    native_records = {entry["operation"]: entry for entry in document["native_verification"]}
    representative_outputs = compatibility.get("representative_operation_outputs")
    assert isinstance(representative_outputs, list)
    assert [entry["operation_id"] for entry in representative_outputs] == [
        "resume.analyze",
        "job.match",
    ]
    for entry in representative_outputs:
        native = native_records[entry["operation_id"]]
        assert entry["output_schema_id"] == native["output_schema_version"]
        assert entry["output_size_bytes"] == native["output_size_bytes"]
        assert entry["output_sha256"] == native["output_sha256"]


validate_compatibility(metadata)

adversarial_documents = []
for mutation in ("catalog_digest", "mapping", "bundle", "bound", "representative_output"):
    altered = copy.deepcopy(metadata)
    compatibility = altered["managed_adapter_compatibility"]
    if mutation == "catalog_digest":
        compatibility["operation_catalog"]["catalog_sha256"] = "0" * 64
    elif mutation == "mapping":
        compatibility["available_capability_operation_mapping"].pop()
    elif mutation == "bundle":
        compatibility["representative_schema_bundles"].pop()
    elif mutation == "bound":
        compatibility["declared_operation_output_bounds"][0][
            "maximum_successful_machine_output_bytes"
        ] += 1
    else:
        compatibility["representative_operation_outputs"][0]["output_sha256"] = "0" * 64
    adversarial_documents.append(altered)

for altered in adversarial_documents:
    try:
        validate_compatibility(altered)
    except AssertionError:
        pass
    else:
        raise AssertionError("tampered managed-adapter compatibility metadata was accepted")

print("Runtime artifact shell/metadata test passed.")
PY
