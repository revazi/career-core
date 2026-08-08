#!/usr/bin/env bash
set -euo pipefail

if (($# != 1)); then
  printf 'Usage: scripts/verify-managed-adapter-contracts.sh <career-executable>\n' >&2
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
career="$(python3 - "$1" <<'PY'
import pathlib
import sys
print(pathlib.Path(sys.argv[1]).expanduser().resolve(strict=True))
PY
)"
check_jsonschema="${CHECK_JSONSCHEMA_BIN:-check-jsonschema}"
command -v python3 >/dev/null 2>&1 || {
  printf 'managed-adapter verification requires python3\n' >&2
  exit 1
}
command -v "$check_jsonschema" >/dev/null 2>&1 || {
  printf 'managed-adapter verification requires check-jsonschema; set CHECK_JSONSCHEMA_BIN to a pinned executable\n' >&2
  exit 1
}
[[ -f "$career" && -x "$career" ]] || {
  printf 'managed-adapter verification requires an executable career binary\n' >&2
  exit 1
}

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-managed-adapter.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT

python3 "$script_dir/validate-managed-adapter-contracts.py" \
  --career "$career" \
  --output-dir "$temporary_root/validation"

bundle_dir="$temporary_root/validation/bundles"
discovery_dir="$temporary_root/validation/discovery"
"$check_jsonschema" --check-metaschema "$bundle_dir"/*.bundle.json

validate() {
  local schema_file="$1"
  shift
  "$check_jsonschema" --schemafile "$bundle_dir/$schema_file.bundle.json" "$@"
}

validate capabilities-v1.schema.json "$discovery_dir/capabilities.json"
validate operation-catalog-v1.schema.json \
  "$discovery_dir/operations.json" \
  "$repository_root/fixtures/managed-adapter/phase8/operation-catalog.expected.json"
validate schema-catalog-v1.schema.json "$discovery_dir/schema-catalog.json"

validate resume-input-v1.schema.json \
  "$repository_root/fixtures/resume/phase1/complete-sections.input.json" \
  "$repository_root/fixtures/resume/phase3/complete-analysis.input.json" \
  "$repository_root/fixtures/resume/phase2/complete-normalization.input.json"
validate resume-evaluation-v1.schema.json \
  "$repository_root/fixtures/resume/phase1/complete-sections.expected.json"
validate resume-analysis-v1.schema.json \
  "$repository_root/fixtures/resume/phase3/complete-analysis.expected.json"
validate resume-normalization-v1.schema.json \
  "$repository_root/fixtures/resume/phase2/complete-normalization.expected.json"
validate resume-enrichment-input-v1.schema.json \
  "$repository_root/fixtures/resume/phase2/messy-unlabeled.enrichment-input.json"
validate resume-enrichment-result-v1.schema.json \
  "$repository_root/fixtures/resume/phase2/messy-unlabeled.enrichment-expected.json"
validate resume-analysis-suggestion-review-input-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/complete-analysis-suggestion-review.input.json"
validate resume-analysis-suggestion-review-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/complete-analysis-suggestion-review.expected.json"
validate resume-analysis-replacement-review-input-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/complete-analysis-replacement-review.input.json"
validate resume-analysis-replacement-review-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/complete-analysis-replacement-review.expected.json"
validate resume-variant-review-input-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/complete-variant-review.input.json"
validate resume-variant-review-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/complete-variant-review.expected.json"
validate resume-variant-materialization-input-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/selected-variant-materialization.input.json"
validate resume-variant-v1.schema.json \
  "$repository_root/fixtures/resume/phase7/selected-variant-materialization.expected.json"
validate job-input-v1.schema.json \
  "$repository_root/fixtures/job/phase4a/complete-normalization.input.json"
validate job-normalization-v1.schema.json \
  "$repository_root/fixtures/job/phase4a/complete-normalization.expected.json"
validate job-match-input-v1.schema.json \
  "$repository_root/fixtures/job/phase4b/complete-match.input.json"
validate job-match-v1.schema.json \
  "$repository_root/fixtures/job/phase4b/complete-match.expected.json"

printf 'Independent Draft 2020-12 bundle and representative-instance validation passed.\n'
