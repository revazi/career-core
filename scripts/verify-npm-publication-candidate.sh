#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'npm publication candidate execution verification failed: %s\n' "$1" >&2
  exit 1
}

candidate_dir=""
reviewed_sha=""
expected_target=""
while (($# > 0)); do
  case "$1" in
    --candidate-dir) (($# >= 2)) || fail "--candidate-dir requires a value"; candidate_dir="$2"; shift 2 ;;
    --reviewed-sha) (($# >= 2)) || fail "--reviewed-sha requires a value"; reviewed_sha="$2"; shift 2 ;;
    --expected-target) (($# >= 2)) || fail "--expected-target requires a value"; expected_target="$2"; shift 2 ;;
    --help|-h)
      printf 'Usage: %s --candidate-dir <directory> --reviewed-sha <sha> --expected-target <approved-target>\n' "$0"
      exit 0
      ;;
    *) fail "unknown argument: $1" ;;
  esac
done

[[ -d "$candidate_dir" ]] || fail "candidate directory is missing"
[[ "$reviewed_sha" =~ ^[0-9a-f]{40}$ ]] || fail "reviewed SHA is invalid"
for command in cmp node npm python3; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done
[[ "$(node --version)" == "v22.19.0" ]] || fail "candidate verification requires exact Node v22.19.0"
[[ "$(npm --version)" == "11.6.2" ]] || fail "candidate verification requires exact npm 11.6.2"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
candidate_dir="$(cd "$candidate_dir" && pwd -P)"
release_version="$(node -e 'process.stdout.write(require(process.argv[1]).version)' "$repository_root/npm/career/package.json")"
[[ "$release_version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || \
  fail "source launcher version is not an exact stable SemVer"
"$script_dir/npm-publication-candidate.py" verify "$candidate_dir" \
  --source-sha "$reviewed_sha" \
  --repository-root "$repository_root"

case "$(uname -s):$(uname -m):$expected_target" in
  Darwin:arm64:aarch64-apple-darwin|Darwin:aarch64:aarch64-apple-darwin)
    package_name="@revazi/career-darwin-arm64"
    native_tarball="$candidate_dir/10-revazi-career-darwin-arm64-$release_version.tgz"
    ;;
  Linux:x86_64:x86_64-unknown-linux-gnu|Linux:amd64:x86_64-unknown-linux-gnu)
    package_name="@revazi/career-linux-x64-gnu"
    native_tarball="$candidate_dir/30-revazi-career-linux-x64-gnu-$release_version.tgz"
    getconf GNU_LIBC_VERSION 2>/dev/null | grep -Eq '^glibc [0-9]+\.[0-9]+' || \
      fail "candidate Linux execution requires confirmed glibc"
    ;;
  *) fail "expected target does not match an approved native host" ;;
esac
launcher_tarball="$candidate_dir/70-revazi-career-$release_version.tgz"

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-npm-publication-verify.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT
consumer="$temporary_root/consumer"
mkdir -p "$consumer"
python3 - "$consumer/package.json" "$launcher_tarball" "$native_tarball" "$package_name" <<'PY'
import json
import pathlib
import sys
manifest_path = pathlib.Path(sys.argv[1])
launcher = pathlib.Path(sys.argv[2]).resolve().as_uri()
native = pathlib.Path(sys.argv[3]).resolve().as_uri()
package_name = sys.argv[4]
manifest = {
    "name": "career-npm-publication-candidate-verification",
    "version": "0.0.0",
    "private": True,
    "dependencies": {
        "@revazi/career": launcher,
        package_name: native,
    },
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

export npm_config_audit=false
export npm_config_fund=false
export npm_config_ignore_scripts=true
export npm_config_offline=true
export npm_config_update_notifier=false
export npm_config_cache="$temporary_root/npm-cache"
if ! (
  cd "$consumer"
  npm install --offline --ignore-scripts --no-audit --no-fund --no-package-lock \
    >"$temporary_root/npm-install.stdout" 2>"$temporary_root/npm-install.stderr"
); then
  fail "candidate offline install failed"
fi

launcher="$consumer/node_modules/.bin/career"
native="$consumer/node_modules/${package_name}/career"
[[ -x "$launcher" && -x "$native" ]] || fail "candidate extracted install is incomplete"

python3 - "$consumer" "$package_name" <<'PY'
import pathlib
import sys
root = pathlib.Path(sys.argv[1]) / "node_modules"
package_name = sys.argv[2]
launcher = root / "@revazi" / "career"
native = root.joinpath(*package_name.split("/"))
expected_launcher = {
    "package.json", "bin/career.js", "targets.json", "README.md", "LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"
}
expected_native = {
    "package.json", "career", "provenance.json", "LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"
}
def entries(directory):
    return {
        path.relative_to(directory).as_posix()
        for path in directory.rglob("*")
        if path.is_file() or path.is_symlink()
    }
if entries(launcher) != expected_launcher or entries(native) != expected_native:
    raise SystemExit("candidate extracted install allowlist mismatch")
PY

result_dir="$temporary_root/results"
mkdir -p "$result_dir"
compare_command() {
  local name="$1"
  shift
  "$native" "$@" >"$result_dir/$name.native.stdout" 2>"$result_dir/$name.native.stderr"
  if ! "$launcher" "$@" >"$result_dir/$name.launcher.stdout" 2>"$result_dir/$name.launcher.stderr"; then
    cat "$result_dir/$name.launcher.stderr" >&2
    fail "$name launcher execution failed"
  fi
  cmp "$result_dir/$name.native.stdout" "$result_dir/$name.launcher.stdout"
  cmp "$result_dir/$name.native.stderr" "$result_dir/$name.launcher.stderr"
  [[ ! -s "$result_dir/$name.native.stderr" ]] || fail "$name wrote unexpected stderr"
  python3 - "$result_dir/$name.launcher.stdout" <<'PY'
import pathlib
import sys
data = pathlib.Path(sys.argv[1]).read_bytes()
if not data or len(data) > 33_554_432:
    raise SystemExit("candidate command violates the Phase 8 output bound")
PY
}

compare_golden() {
  local name="$1"
  local golden="$2"
  shift 2
  compare_command "$name" "$@"
  cmp "$result_dir/$name.launcher.stdout" "$repository_root/$golden"
}

compare_command version --version
compare_golden capabilities fixtures/managed-adapter/phase8/capabilities.pre-phase8.expected.json capabilities
compare_golden operations fixtures/managed-adapter/phase8/operation-catalog.expected.json operations
compare_command schema-list schema list
compare_command schema-export schema export --id career.job_match.v1 --format json-compact
compare_command schema-bundle schema bundle --id career.job_match_input.v1 --format json-compact
compare_golden resume-evaluate fixtures/resume/phase1/complete-sections.expected.json \
  resume evaluate --input "$repository_root/fixtures/resume/phase1/complete-sections.input.json"
compare_golden resume-analyze fixtures/resume/phase3/complete-analysis.expected.json \
  resume analyze --input "$repository_root/fixtures/resume/phase3/complete-analysis.input.json"
compare_golden analysis-suggestions fixtures/resume/phase7/complete-analysis-suggestion-review.expected.json \
  resume analysis-suggestions-review \
  --input "$repository_root/fixtures/resume/phase7/complete-analysis-suggestion-review.input.json"
compare_golden analysis-replacements fixtures/resume/phase7/complete-analysis-replacement-review.expected.json \
  resume analysis-replacements-review \
  --input "$repository_root/fixtures/resume/phase7/complete-analysis-replacement-review.input.json"
compare_golden resume-normalize fixtures/resume/phase2/complete-normalization.expected.json \
  resume normalize --input "$repository_root/fixtures/resume/phase2/complete-normalization.input.json"
compare_golden resume-enrich fixtures/resume/phase2/messy-unlabeled.enrichment-expected.json \
  resume enrich --input "$repository_root/fixtures/resume/phase2/messy-unlabeled.enrichment-input.json"
compare_golden variant-review fixtures/resume/phase7/complete-variant-review.expected.json \
  resume variant-review --input "$repository_root/fixtures/resume/phase7/complete-variant-review.input.json"
compare_golden variant-materialize fixtures/resume/phase7/selected-variant-materialization.expected.json \
  resume variant-materialize \
  --input "$repository_root/fixtures/resume/phase7/selected-variant-materialization.input.json"
compare_golden job-normalize fixtures/job/phase4a/complete-normalization.expected.json \
  job normalize --input "$repository_root/fixtures/job/phase4a/complete-normalization.input.json"
compare_golden job-match fixtures/job/phase4b/complete-match.expected.json \
  job match --input "$repository_root/fixtures/job/phase4b/complete-match.input.json"

(
  cd "$consumer"
  npm exec --offline --ignore-scripts -- career --version \
    >"$result_dir/npm-exec.stdout" 2>"$result_dir/npm-exec.stderr"
)
cmp "$result_dir/npm-exec.stdout" "$result_dir/version.native.stdout"
[[ ! -s "$result_dir/npm-exec.stderr" ]] || fail "synthetic npx-equivalent invocation wrote stderr"

printf 'Candidate extracted install, every-operation parity, and synthetic npx-equivalent bin invocation passed.\n'
