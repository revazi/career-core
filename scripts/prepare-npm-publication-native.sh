#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'npm native publication candidate preparation failed: %s\n' "$1" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Usage: scripts/prepare-npm-publication-native.sh \
  --output-dir <external-empty-directory> \
  --expected-target <approved-rust-target> \
  --expected-ref refs/tags/v0.1.0 \
  --reviewed-sha <40-lowercase-hex> \
  --runner-os <Linux|macOS> \
  --runner-arch <X64|ARM64> \
  --runner-image <bounded-runner-image>

Create one public native v0.1.0 candidate tarball from the exact clean annotated
tag on origin/main. Source templates remain private. This command never queries
npm, authenticates, publishes, creates a tag, or changes the checkout.
EOF
}

output_dir=""
expected_target=""
expected_ref=""
reviewed_sha=""
runner_os=""
runner_arch=""
runner_image=""
while (($# > 0)); do
  case "$1" in
    --output-dir) (($# >= 2)) || fail "--output-dir requires a value"; output_dir="$2"; shift 2 ;;
    --expected-target) (($# >= 2)) || fail "--expected-target requires a value"; expected_target="$2"; shift 2 ;;
    --expected-ref) (($# >= 2)) || fail "--expected-ref requires a value"; expected_ref="$2"; shift 2 ;;
    --reviewed-sha) (($# >= 2)) || fail "--reviewed-sha requires a value"; reviewed_sha="$2"; shift 2 ;;
    --runner-os) (($# >= 2)) || fail "--runner-os requires a value"; runner_os="$2"; shift 2 ;;
    --runner-arch) (($# >= 2)) || fail "--runner-arch requires a value"; runner_arch="$2"; shift 2 ;;
    --runner-image) (($# >= 2)) || fail "--runner-image requires a value"; runner_image="$2"; shift 2 ;;
    --help|-h) usage; exit 0 ;;
    *) fail "unknown argument: $1" ;;
  esac
done

[[ -n "$output_dir" ]] || fail "--output-dir is required"
[[ -n "$expected_target" ]] || fail "--expected-target is required"
[[ -n "$expected_ref" ]] || fail "--expected-ref is required"
[[ "$reviewed_sha" =~ ^[0-9a-f]{40}$ ]] || fail "--reviewed-sha must be a full lowercase commit"
[[ "$runner_image" =~ ^[A-Za-z0-9._/+:-]{1,128}$ ]] || fail "--runner-image is invalid"
for command in cargo git node npm python3 rustc; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done
[[ "$(node --version)" == "v22.19.0" ]] || fail "publication candidates require exact Node v22.19.0"
[[ "$(npm --version)" == "11.6.2" ]] || fail "publication candidates require exact npm 11.6.2"
[[ "$(rustc --version)" == rustc\ 1.97.1\ * ]] || fail "publication candidates require exact rustc 1.97.1"
[[ "$(cargo --version)" == cargo\ 1.97.1\ * ]] || fail "publication candidates require exact Cargo 1.97.1"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
"$script_dir/verify-npm-publication-source.sh" \
  --repository-root "$repository_root" \
  --expected-ref "$expected_ref" \
  --reviewed-sha "$reviewed_sha"

case "$expected_target" in
  aarch64-apple-darwin)
    platform_key="darwin-arm64"
    expected_runner_os="macOS"
    expected_runner_arch="ARM64"
    final_name="10-revazi-career-darwin-arm64-0.1.0.tgz"
    ;;
  x86_64-unknown-linux-gnu)
    platform_key="linux-x64-gnu"
    expected_runner_os="Linux"
    expected_runner_arch="X64"
    final_name="20-revazi-career-linux-x64-gnu-0.1.0.tgz"
    ;;
  *) fail "expected target is not an approved native publication target" ;;
esac
[[ "$runner_os" == "$expected_runner_os" ]] || fail "runner OS does not match the approved native target"
[[ "$runner_arch" == "$expected_runner_arch" ]] || fail "runner architecture does not match the approved native target"

resolved_output_dir="$(python3 - "$output_dir" <<'PY'
import pathlib
import sys
print(pathlib.Path(sys.argv[1]).expanduser().resolve(strict=False))
PY
)"
case "$resolved_output_dir/" in
  "$repository_root/"*) fail "output directory must be outside the source checkout" ;;
esac
if [[ -e "$resolved_output_dir" && -n "$(find "$resolved_output_dir" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
  fail "output directory must be empty"
fi
mkdir -p "$resolved_output_dir/work" "$resolved_output_dir/tarballs"
output_dir="$(cd "$resolved_output_dir" && pwd -P)"

private_output="$output_dir/work/private"
"$script_dir/prepare-npm-cli-packages.sh" \
  --output-dir "$private_output" \
  --expected-target "$expected_target"

platform_stage="$private_output/stage/$platform_key"
if [[ "$platform_key" == "linux-x64-gnu" ]]; then
  command -v getconf >/dev/null 2>&1 || fail "Linux candidate requires getconf"
  command -v readelf >/dev/null 2>&1 || fail "Linux candidate requires readelf"
  [[ "$(getconf GNU_LIBC_VERSION 2>/dev/null)" == "glibc 2.35" ]] || \
    fail "Linux publication candidate must build on exact glibc 2.35"
  maximum_required="$(
    readelf --version-info "$platform_stage/career" |
      sed -n 's/.*Name: GLIBC_\([0-9.]*\).*/\1/p' |
      sort -Vu |
      tail -n1
  )"
  [[ -n "$maximum_required" ]] || fail "Linux candidate has no inspectable GLIBC requirement"
  highest="$(printf '%s\n%s\n' "$maximum_required" "2.35" | sort -V | tail -n1)"
  [[ "$highest" == "2.35" ]] || fail "Linux candidate requires GLIBC_$maximum_required above the supported 2.35 floor"
fi
"$script_dir/npm-publication-candidate.py" public-manifest \
  "$repository_root/npm/platforms/$platform_key/package.json" \
  "$platform_stage/package.json"
"$script_dir/npm-publication-candidate.py" promote-provenance \
  "$platform_stage/provenance.json" \
  --source-sha "$reviewed_sha" \
  --runner-os "$runner_os" \
  --runner-arch "$runner_arch" \
  --runner-image "$runner_image"

export npm_config_audit=false
export npm_config_fund=false
export npm_config_ignore_scripts=true
export npm_config_offline=true
export npm_config_update_notifier=false
export npm_config_cache="$output_dir/work/npm-cache"
npm pack --ignore-scripts --json --pack-destination "$output_dir/work" "$platform_stage" \
  >"$output_dir/work/native-pack.json"
packed_name="$(python3 - "$output_dir/work/native-pack.json" <<'PY'
import json
import pathlib
import sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
if not isinstance(value, list) or len(value) != 1 or not isinstance(value[0].get("filename"), str):
    raise SystemExit("npm pack did not return exactly one candidate tarball")
print(value[0]["filename"])
PY
)"
[[ -f "$output_dir/work/$packed_name" ]] || fail "npm pack candidate tarball is missing"
mv "$output_dir/work/$packed_name" "$output_dir/tarballs/$final_name"
"$script_dir/npm-publication-candidate.py" verify-native \
  "$output_dir/tarballs/$final_name" \
  --platform-key "$platform_key" \
  --source-sha "$reviewed_sha"
rm -rf "$output_dir/work"

[[ "$(find "$output_dir" -type f | wc -l | tr -d ' ')" == "1" ]] || fail "native candidate output allowlist mismatch"
printf 'Prepared public internal native candidate %s from %s; no publication performed.\n' \
  "$final_name" "$reviewed_sha"
