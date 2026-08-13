#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-core-installed.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT

install_root="$temporary_root/install"
run_root="$temporary_root/outside-checkout"
mkdir -p "$run_root"

python3 - "$repository_root/src/lib.rs" <<'PY'
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
policy = '''#[cfg(target_os = "windows")]
compile_error!("native Windows is unsupported; build and run Career Core in WSL");'''
if source.count(policy) != 1:
    raise SystemExit("career-core native Windows compile rejection policy is missing or changed")
if source.index(policy) > source.index("use serde"):
    raise SystemExit("career-core native Windows compile rejection must precede library implementation")
PY

windows_policy_output="$temporary_root/native-windows-policy.rlib"
windows_policy_stdout="$temporary_root/native-windows-policy.stdout"
windows_policy_stderr="$temporary_root/native-windows-policy.stderr"
if rustc \
  --edition 2024 \
  --crate-type lib \
  --cfg 'target_os="windows"' \
  -Aexplicit_builtin_cfgs_in_flags \
  "$repository_root/src/lib.rs" \
  -o "$windows_policy_output" \
  >"$windows_policy_stdout" \
  2>"$windows_policy_stderr"
then
  printf 'career-core unexpectedly compiled for native Windows\n' >&2
  exit 1
fi
grep -Fq \
  'native Windows is unsupported; build and run Career Core in WSL' \
  "$windows_policy_stderr" || {
  printf 'career-core native Windows rejection omitted exact WSL guidance\n' >&2
  exit 1
}
[[ ! -e "$windows_policy_output" ]] || {
  printf 'career-core native Windows rejection produced a library artifact\n' >&2
  exit 1
}

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
