#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-core-installed.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT

install_root="$temporary_root/install"
run_root="$temporary_root/outside-checkout"
mkdir -p "$run_root"

cargo install \
  --path "$repository_root/crates/career-cli" \
  --locked \
  --root "$install_root"

career="$install_root/bin/career"
run_installed() {
  (cd "$run_root" && "$career" "$@" >/dev/null)
}

run_installed capabilities --format json-compact
run_installed operations --format json-compact
run_installed schema list --format json-compact
run_installed schema export --id career.job_match_input.v1 --format json-compact
run_installed schema bundle --id career.job_match_input.v1 --format json-compact
python3 "$repository_root/scripts/validate-managed-adapter-contracts.py" \
  --career "$career" \
  --output-dir "$temporary_root/managed-adapter-validation"
run_installed resume analyze \
  --input "$repository_root/fixtures/resume/phase3/complete-analysis.input.json" \
  --format json-compact
run_installed resume analysis-suggestions-review \
  --input "$repository_root/fixtures/resume/phase7/complete-analysis-suggestion-review.input.json" \
  --format json-compact
run_installed resume variant-materialize \
  --input "$repository_root/fixtures/resume/phase7/selected-variant-materialization.input.json" \
  --format json-compact
run_installed job match \
  --input "$repository_root/fixtures/job/phase4b/complete-match.input.json" \
  --format json-compact

printf 'Installed career CLI acceptance passed outside the checkout.\n'
