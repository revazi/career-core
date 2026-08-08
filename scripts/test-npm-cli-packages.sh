#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
prepare_script="$script_dir/prepare-npm-cli-packages.sh"

case "$(uname -s):$(uname -m)" in
  Darwin:arm64|Darwin:aarch64)
    platform_key="darwin-arm64"
    package_name="@revazi/career-darwin-arm64"
    expected_target="aarch64-apple-darwin"
    wrong_target="x86_64-unknown-linux-gnu"
    ;;
  Linux:x86_64|Linux:amd64)
    platform_key="linux-x64-gnu"
    package_name="@revazi/career-linux-x64-gnu"
    expected_target="x86_64-unknown-linux-gnu"
    wrong_target="aarch64-apple-darwin"
    ;;
  *)
    printf 'npm CLI package test requires an approved native host\n' >&2
    exit 1
    ;;
esac

for command in cmp git node npm python3 tar; do
  command -v "$command" >/dev/null 2>&1 || {
    printf 'npm CLI package test requires %s\n' "$command" >&2
    exit 1
  }
done

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-npm-package-test.XXXXXX")"
inside_output="$repository_root/.npm-package-test-inside-$$"
inside_link="$temporary_root/repository-link"
inside_argument="$inside_link/$(basename "$inside_output")"
trap 'rm -rf "$temporary_root" "$inside_output"' EXIT
ln -s "$repository_root" "$inside_link"

if "$prepare_script" \
  --output-dir "$inside_argument" \
  --expected-target "$expected_target" \
  --allow-dirty >"$temporary_root/inside.stdout" 2>"$temporary_root/inside.stderr"; then
  printf 'npm CLI package test expected in-checkout output rejection\n' >&2
  exit 1
fi
grep -Fq 'output directory must be outside the source checkout' "$temporary_root/inside.stderr"
[[ ! -e "$inside_output" ]] || {
  printf 'npm CLI package test found an in-checkout staging side effect\n' >&2
  exit 1
}

if "$prepare_script" \
  --output-dir "$temporary_root/wrong-target" \
  --expected-target "$wrong_target" \
  --allow-dirty >"$temporary_root/wrong.stdout" 2>"$temporary_root/wrong.stderr"; then
  printf 'npm CLI package test expected target mismatch rejection\n' >&2
  exit 1
fi
grep -Fq 'native target mismatch' "$temporary_root/wrong.stderr"

if [[ -n "$(git -C "$repository_root" status --porcelain --untracked-files=normal)" ]]; then
  if "$prepare_script" \
    --output-dir "$temporary_root/dirty-rejected" \
    --expected-target "$expected_target" \
    >"$temporary_root/dirty.stdout" 2>"$temporary_root/dirty.stderr"; then
    printf 'npm CLI package test expected dirty preparation rejection\n' >&2
    exit 1
  fi
  grep -Fq 'source worktree is dirty' "$temporary_root/dirty.stderr"
fi

output_dir="$temporary_root/prepared"
"$prepare_script" \
  --output-dir "$output_dir" \
  --expected-target "$expected_target" \
  --allow-dirty

launcher_stage="$output_dir/stage/launcher"
platform_stage="$output_dir/stage/$platform_key"
tarball_dir="$output_dir/tarballs"
[[ -x "$platform_stage/career" ]]
[[ -x "$launcher_stage/bin/career.js" ]]

python3 - \
  "$repository_root" \
  "$launcher_stage" \
  "$platform_stage" \
  "$tarball_dir" \
  "$package_name" \
  "$expected_target" <<'PY'
import hashlib
import json
import pathlib
import re
import stat
import sys
(
    root_arg,
    launcher_arg,
    platform_arg,
    tarballs_arg,
    expected_package,
    expected_target,
) = sys.argv[1:]
root = pathlib.Path(root_arg)
launcher = pathlib.Path(launcher_arg)
platform = pathlib.Path(platform_arg)
tarballs = pathlib.Path(tarballs_arg)
launcher_manifest = json.loads((launcher / "package.json").read_text())
platform_manifest = json.loads((platform / "package.json").read_text())
provenance = json.loads((platform / "provenance.json").read_text())
assert launcher_manifest["name"] == "@revazi/career"
assert launcher_manifest["private"] is True
assert launcher_manifest["bin"] == {"career": "bin/career.js"}
assert launcher_manifest["optionalDependencies"] == {
    "@revazi/career-darwin-arm64": launcher_manifest["version"],
    "@revazi/career-linux-x64-gnu": launcher_manifest["version"],
}
assert platform_manifest["name"] == expected_package
assert platform_manifest["version"] == launcher_manifest["version"]
assert platform_manifest["private"] is True
assert provenance["schema_version"] == "career.npm_native_provenance.v1"
assert provenance["package"]["name"] == expected_package
assert provenance["package"]["version"] == launcher_manifest["version"]
assert provenance["package"]["rust_target"] == expected_target
assert provenance["package"]["minimum_glibc_version"] == (
    "2.35" if expected_target == "x86_64-unknown-linux-gnu" else None
)
assert provenance["source"]["git_ref"] is None
assert provenance["source"]["git_tag"] is None
assert provenance["source"]["publication_candidate"] is False
assert provenance["build"]["runner"]["image"] == "local"
if expected_target == "x86_64-unknown-linux-gnu":
    assert provenance["build"]["runner"]["libc"].startswith("glibc ")
else:
    assert provenance["build"]["runner"]["libc"] is None
assert provenance["integrity"] == {
    "independent_signature": "absent",
    "npm_registry_integrity": "external_to_launcher_runtime",
    "package_contained_sha256": "consistency_only",
}
binary = (platform / "career").read_bytes()
assert provenance["executable"]["size_bytes"] == len(binary)
assert provenance["executable"]["sha256"] == hashlib.sha256(binary).hexdigest()
assert stat.S_IMODE((platform / "career").stat().st_mode) == 0o755
assert (launcher / "README.md").read_bytes() == (root / "npm/career/README.md").read_bytes()
for package_dir in (launcher, platform):
    for license_name in ("LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"):
        assert (package_dir / license_name).read_bytes() == (root / license_name).read_bytes()
assert len(list(tarballs.glob("*.tgz"))) == 2
tracked = []
for path in (root / "npm").rglob("*"):
    if path.is_file() and (path.name == "career" or path.name == "provenance.json"):
        tracked.append(path)
assert tracked == [], tracked
assert re.fullmatch(r"[0-9a-f]{40}", provenance["source"]["git_sha"])
PY

launcher_tarball=""
platform_tarball=""
for tarball in "$tarball_dir"/*.tgz; do
  name="$(tar -xOf "$tarball" package/package.json | node -e 'const fs=require("fs"); process.stdout.write(JSON.parse(fs.readFileSync(0,"utf8")).name)')"
  case "$name" in
    @revazi/career) launcher_tarball="$tarball" ;;
    "$package_name") platform_tarball="$tarball" ;;
    *) printf 'unexpected npm package in %s: %s\n' "$tarball" "$name" >&2; exit 1 ;;
  esac
done
[[ -f "$launcher_tarball" && -f "$platform_tarball" ]]

consumer="$temporary_root/consumer-outside-checkout"
mkdir -p "$consumer"
python3 - "$consumer/package.json" "$launcher_tarball" "$platform_tarball" "$package_name" <<'PY'
import json
import pathlib
import sys
manifest_path = pathlib.Path(sys.argv[1])
launcher = pathlib.Path(sys.argv[2]).resolve().as_uri()
platform = pathlib.Path(sys.argv[3]).resolve().as_uri()
package_name = sys.argv[4]
manifest = {
    "name": "career-npm-offline-extracted-test",
    "version": "0.0.0",
    "private": True,
    "dependencies": {
        "@revazi/career": launcher,
        package_name: platform,
    },
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
PY

export npm_config_audit=false
export npm_config_fund=false
export npm_config_ignore_scripts=true
export npm_config_offline=true
export npm_config_update_notifier=false
export npm_config_cache="$temporary_root/npm-cache"
(
  cd "$consumer"
  npm install --offline --ignore-scripts --no-audit --no-fund --no-package-lock \
    >"$temporary_root/npm-install.stdout" 2>"$temporary_root/npm-install.stderr"
)

launcher="$consumer/node_modules/.bin/career"
native="$consumer/node_modules/${package_name}/career"
[[ -x "$launcher" && -x "$native" ]]

python3 - "$consumer" "$package_name" <<'PY'
import pathlib
import sys
root = pathlib.Path(sys.argv[1]) / "node_modules"
package_name = sys.argv[2]
launcher = root / "@revazi" / "career"
platform = root.joinpath(*package_name.split("/"))
expected_launcher = {
    "package.json",
    "bin/career.js",
    "README.md",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "THIRD_PARTY_NOTICES.md",
}
expected_platform = {
    "package.json",
    "career",
    "provenance.json",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "THIRD_PARTY_NOTICES.md",
}
def files(directory):
    return {
        path.relative_to(directory).as_posix()
        for path in directory.rglob("*")
        if path.is_file() or path.is_symlink()
    }
assert files(launcher) == expected_launcher
assert files(platform) == expected_platform
PY

result_dir="$temporary_root/results"
mkdir -p "$result_dir"
compare_command() {
  local name="$1"
  shift
  "$native" "$@" >"$result_dir/$name.native.stdout" 2>"$result_dir/$name.native.stderr"
  "$launcher" "$@" >"$result_dir/$name.launcher.stdout" 2>"$result_dir/$name.launcher.stderr"
  cmp "$result_dir/$name.native.stdout" "$result_dir/$name.launcher.stdout"
  cmp "$result_dir/$name.native.stderr" "$result_dir/$name.launcher.stderr"
  [[ ! -s "$result_dir/$name.native.stderr" ]]
  python3 - "$result_dir/$name.launcher.stdout" <<'PY'
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
data = path.read_bytes()
if not data or len(data) > 33_554_432:
    raise SystemExit("packaged command output violates the Phase 8 bound")
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

python3 - "$result_dir" <<'PY'
import json
import pathlib
import sys
root = pathlib.Path(sys.argv[1])
for path in root.glob("*.launcher.stdout"):
    if path.name == "version.launcher.stdout":
        assert path.read_text() == "career 0.1.0\n"
        continue
    json.loads(path.read_bytes())
PY

printf 'Private npm tarball, offline extracted-install, native parity, and allowlist tests passed.\n'
