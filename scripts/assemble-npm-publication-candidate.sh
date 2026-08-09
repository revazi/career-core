#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'npm publication candidate assembly failed: %s\n' "$1" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Usage: scripts/assemble-npm-publication-candidate.sh \
  --output-dir <external-empty-directory> \
  --native-dir <directory-containing-eight-exact-native-tarballs> \
  --expected-ref refs/tags/v0.1.1 \
  --reviewed-sha <40-lowercase-hex>

Validate all eight exact native candidates, pack the public @revazi/career
launcher last, and emit exactly nine ordered tarballs plus one integrity
manifest. This command never queries npm, authenticates, publishes, or changes
the checkout.
EOF
}

output_dir=""
native_dir=""
expected_ref=""
reviewed_sha=""
while (($# > 0)); do
  case "$1" in
    --output-dir) (($# >= 2)) || fail "--output-dir requires a value"; output_dir="$2"; shift 2 ;;
    --native-dir) (($# >= 2)) || fail "--native-dir requires a value"; native_dir="$2"; shift 2 ;;
    --expected-ref) (($# >= 2)) || fail "--expected-ref requires a value"; expected_ref="$2"; shift 2 ;;
    --reviewed-sha) (($# >= 2)) || fail "--reviewed-sha requires a value"; reviewed_sha="$2"; shift 2 ;;
    --help|-h) usage; exit 0 ;;
    *) fail "unknown argument: $1" ;;
  esac
done

[[ -n "$output_dir" ]] || fail "--output-dir is required"
[[ -d "$native_dir" && ! -L "$native_dir" ]] || fail "native candidate directory is missing or unsafe"
[[ "$reviewed_sha" =~ ^[0-9a-f]{40}$ ]] || fail "--reviewed-sha must be a full lowercase commit"
for command in git node npm python3; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done
[[ "$(node --version)" == "v22.19.0" ]] || fail "publication candidates require exact Node v22.19.0"
[[ "$(npm --version)" == "11.6.2" ]] || fail "publication candidates require exact npm 11.6.2"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
"$script_dir/verify-npm-publication-source.sh" \
  --repository-root "$repository_root" \
  --expected-ref "$expected_ref" \
  --reviewed-sha "$reviewed_sha"

native_dir="$(cd "$native_dir" && pwd -P)"
platform_keys=(
  darwin-arm64
  darwin-x64
  linux-x64-gnu
  linux-arm64-gnu
  linux-x64-musl
  linux-arm64-musl
  win32-x64-msvc
  win32-arm64-msvc
)
native_files=(
  10-revazi-career-darwin-arm64-0.1.1.tgz
  20-revazi-career-darwin-x64-0.1.1.tgz
  30-revazi-career-linux-x64-gnu-0.1.1.tgz
  40-revazi-career-linux-arm64-gnu-0.1.1.tgz
  50-revazi-career-linux-x64-musl-0.1.1.tgz
  60-revazi-career-linux-arm64-musl-0.1.1.tgz
  70-revazi-career-win32-x64-msvc-0.1.1.tgz
  80-revazi-career-win32-arm64-msvc-0.1.1.tgz
)
python3 - "$native_dir" "${native_files[@]}" <<'PY'
import pathlib
import sys
root = pathlib.Path(sys.argv[1])
expected = set(sys.argv[2:])
entries = list(root.iterdir())
if {path.name for path in entries} != expected:
    raise SystemExit("native candidate directory allowlist mismatch")
if any(path.is_symlink() or not path.is_file() for path in entries):
    raise SystemExit("native candidates must be flat regular files")
PY
for index in "${!platform_keys[@]}"; do
  "$script_dir/npm-publication-candidate.py" verify-native \
    "$native_dir/${native_files[$index]}" \
    --platform-key "${platform_keys[$index]}" \
    --source-sha "$reviewed_sha"
done

python3 - "$repository_root" "$native_dir" "${native_files[@]}" <<'PY'
import pathlib
import sys
import tarfile
root = pathlib.Path(sys.argv[1])
native_dir = pathlib.Path(sys.argv[2])
for filename in sys.argv[3:]:
    with tarfile.open(native_dir / filename, "r:gz") as archive:
        for name in ("LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"):
            member = archive.extractfile(f"package/{name}")
            if member is None or member.read() != (root / name).read_bytes():
                raise SystemExit(f"native candidate {name} differs from reviewed source")
PY

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
mkdir -p "$resolved_output_dir/work/launcher/bin"
output_dir="$(cd "$resolved_output_dir" && pwd -P)"

for filename in "${native_files[@]}"; do
  cp "$native_dir/$filename" "$output_dir/$filename"
done
launcher_stage="$output_dir/work/launcher"
"$script_dir/npm-publication-candidate.py" public-manifest \
  "$repository_root/npm/career/package.json" "$launcher_stage/package.json"
cp "$repository_root/npm/career/bin/career.js" "$launcher_stage/bin/career.js"
cp "$repository_root/npm/career/targets.json" "$launcher_stage/targets.json"
cp "$repository_root/npm/career/README.md" "$launcher_stage/README.md"
chmod 0755 "$launcher_stage/bin/career.js"
cp "$repository_root/LICENSE-MIT" "$repository_root/LICENSE-APACHE" \
  "$repository_root/THIRD_PARTY_NOTICES.md" "$launcher_stage/"

export npm_config_audit=false
export npm_config_fund=false
export npm_config_ignore_scripts=true
export npm_config_offline=true
export npm_config_update_notifier=false
export npm_config_cache="$output_dir/work/npm-cache"
npm pack --ignore-scripts --json --pack-destination "$output_dir/work" "$launcher_stage" \
  >"$output_dir/work/launcher-pack.json"
packed_name="$(python3 - "$output_dir/work/launcher-pack.json" <<'PY'
import json
import pathlib
import sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
if not isinstance(value, list) or len(value) != 1 or not isinstance(value[0].get("filename"), str):
    raise SystemExit("npm pack did not return exactly one launcher tarball")
print(value[0]["filename"])
PY
)"
[[ -f "$output_dir/work/$packed_name" ]] || fail "npm pack launcher tarball is missing"
mv "$output_dir/work/$packed_name" "$output_dir/90-revazi-career-0.1.1.tgz"
rm -rf "$output_dir/work"

"$script_dir/npm-publication-candidate.py" write-manifest "$output_dir" --source-sha "$reviewed_sha"
"$script_dir/npm-publication-candidate.py" verify "$output_dir" \
  --source-sha "$reviewed_sha" \
  --repository-root "$repository_root"
[[ "$(find "$output_dir" -mindepth 1 -maxdepth 1 -type f | wc -l | tr -d ' ')" == "10" ]] || \
  fail "assembled publication candidate output allowlist mismatch"
printf 'Assembled ordered public npm candidate for v0.1.1 at %s; no publication performed.\n' \
  "$reviewed_sha"
