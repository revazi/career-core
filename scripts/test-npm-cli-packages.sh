#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
prepare_script="$script_dir/prepare-npm-cli-packages.sh"
inspection_script="$script_dir/inspect-npm-native-binary.py"

case "$(uname -s):$(uname -m)" in
  Darwin:arm64|Darwin:aarch64)
    platform_key="darwin-arm64"
    package_name="@revazi/career-darwin-arm64"
    expected_target="aarch64-apple-darwin"
    wrong_target="x86_64-unknown-linux-gnu"
    ;;
  Darwin:x86_64|Darwin:amd64)
    platform_key="darwin-x64"
    package_name="@revazi/career-darwin-x64"
    expected_target="x86_64-apple-darwin"
    wrong_target="aarch64-unknown-linux-gnu"
    ;;
  Linux:x86_64|Linux:amd64)
    platform_key="linux-x64-gnu"
    package_name="@revazi/career-linux-x64-gnu"
    expected_target="x86_64-unknown-linux-gnu"
    wrong_target="aarch64-apple-darwin"
    ;;
  Linux:aarch64|Linux:arm64)
    platform_key="linux-arm64-gnu"
    package_name="@revazi/career-linux-arm64-gnu"
    expected_target="aarch64-unknown-linux-gnu"
    wrong_target="x86_64-apple-darwin"
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
inside_inspection="$repository_root/.npm-native-inspection-inside-$$.json"
inside_link="$temporary_root/repository-link"
inside_argument="$inside_link/$(basename "$inside_output")"
trap 'rm -rf "$temporary_root" "$inside_output" "$inside_inspection"' EXIT
ln -s "$repository_root" "$inside_link"

if "$prepare_script" \
  --output-dir "$inside_argument" \
  --expected-target "$expected_target" \
  --allow-dirty >"$temporary_root/inside.stdout" 2>"$temporary_root/inside.stderr"; then
  printf 'npm CLI package test expected in-checkout output rejection\n' >&2
  exit 1
fi
if ! grep -Fq 'output directory must be outside the source checkout' "$temporary_root/inside.stderr"; then
  printf 'unexpected in-checkout rejection: ' >&2
  head -c 512 "$temporary_root/inside.stderr" >&2 || true
  printf '\n' >&2
  exit 1
fi
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
catalog = json.loads((root / "npm/career/targets.json").read_text())
targets = [value for value in catalog["targets"] if value["rust_target"] == expected_target]
assert len(targets) == 1
target = targets[0]
platform_packages = [value["native_package"] for value in catalog["targets"]]
assert launcher_manifest["name"] == "@revazi/career"
assert launcher_manifest["version"] == "0.1.1"
assert launcher_manifest["private"] is True
assert launcher_manifest["bin"] == {"career": "bin/career.js"}
assert list(launcher_manifest["optionalDependencies"]) == platform_packages
assert launcher_manifest["optionalDependencies"] == {
    name: launcher_manifest["version"] for name in platform_packages
}
assert launcher_manifest["career_launcher"]["platform_packages"] == platform_packages
assert platform_manifest["name"] == expected_package == target["native_package"]
assert platform_manifest["version"] == launcher_manifest["version"]
assert platform_manifest["private"] is True
assert provenance["schema_version"] == "career.npm_native_provenance.v2"
assert provenance["package"] == {
    "name": expected_package,
    "version": launcher_manifest["version"],
    "platform_key": target["platform_key"],
    "node_platform": target["node_platform"],
    "node_arch": target["node_arch"],
    "libc_family": target["libc_family"],
    "rust_target": expected_target,
    "minimum_glibc_version": target["minimum_glibc_version"],
}
assert provenance["source"]["git_ref"] is None
assert provenance["source"]["git_tag"] is None
assert provenance["source"]["publication_candidate"] is False
assert provenance["build"]["runner"]["image"] == "local"
if target["libc_family"] == "glibc":
    assert provenance["build"]["runner"]["libc"].startswith("glibc ")
elif target["libc_family"] == "musl":
    assert provenance["build"]["runner"]["libc"] == "musl"
else:
    assert provenance["build"]["runner"]["libc"] is None
assert provenance["integrity"] == {
    "independent_signature": "absent",
    "npm_registry_integrity": "external_to_launcher_runtime",
    "package_contained_sha256": "consistency_only",
}
binary_path = platform / target["executable"]
binary = binary_path.read_bytes()
assert provenance["executable"] == {
    "file_name": target["executable"],
    "binary_format": target["binary_format"],
    "binary_architecture": target["binary_architecture"],
    "file_invariant": target["file_invariant"],
    "archive_mode": target["archive_mode"],
    "mode": target["executable_mode"],
    "size_bytes": len(binary),
    "sha256": hashlib.sha256(binary).hexdigest(),
}
assert stat.S_IMODE(binary_path.stat().st_mode) == 0o755
assert (launcher / "targets.json").read_bytes() == (root / "npm/career/targets.json").read_bytes()
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

inspection="$temporary_root/native-inspection.json"
"$inspection_script" \
  --repository-root "$repository_root" \
  --binary "$platform_stage/career" \
  --target "$expected_target" \
  --source-sha "$(git -C "$repository_root" rev-parse HEAD)" \
  --runner-image local-package-test \
  --evidence-kind local_policy \
  --output "$inspection" \
  >"$temporary_root/native-inspection.stdout"
python3 - "$inspection" "$platform_key" "$expected_target" <<'PY'
import json
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
platform_key = sys.argv[2]
target = sys.argv[3]
value = json.loads(path.read_text())
assert value["schema_version"] == "career.npm_native_inspection.v1"
assert value["evidence_kind"] == "local_policy"
assert value["target"]["platform_key"] == platform_key
assert value["target"]["rust_target"] == target
assert value["binary"]["observed_version"] == "career 0.1.1"
assert value["linkage"]["dynamic_imports"]
assert path.stat().st_size <= 64 * 1024
PY

if "$inspection_script" \
  --repository-root "$repository_root" \
  --binary "$platform_stage/career" \
  --target "$expected_target" \
  --source-sha "0000000000000000000000000000000000000000" \
  --runner-image local-package-test \
  --evidence-kind local_policy \
  --output "$temporary_root/wrong-source-inspection.json" \
  >"$temporary_root/wrong-source-inspection.stdout" \
  2>"$temporary_root/wrong-source-inspection.stderr"; then
  printf 'native inspection accepted an incorrect source SHA\n' >&2
  exit 1
fi
grep -Fq 'source SHA does not match' "$temporary_root/wrong-source-inspection.stderr"
[[ ! -e "$temporary_root/wrong-source-inspection.json" ]]

if "$inspection_script" \
  --repository-root "$repository_root" \
  --binary "$platform_stage/career" \
  --target "$expected_target" \
  --source-sha "$(git -C "$repository_root" rev-parse HEAD)" \
  --runner-image local-package-test \
  --evidence-kind local_policy \
  --output "$inside_inspection" \
  >"$temporary_root/inside-inspection.stdout" \
  2>"$temporary_root/inside-inspection.stderr"; then
  printf 'native inspection accepted in-checkout evidence output\n' >&2
  exit 1
fi
grep -Fq 'must be written outside the checkout' "$temporary_root/inside-inspection.stderr"
[[ ! -e "$inside_inspection" ]]

inspection_symlink="$temporary_root/symlinked-native"
ln -s "$platform_stage/career" "$inspection_symlink"
if "$inspection_script" \
  --repository-root "$repository_root" \
  --binary "$inspection_symlink" \
  --target "$expected_target" \
  --source-sha "$(git -C "$repository_root" rev-parse HEAD)" \
  --runner-image local-package-test \
  --evidence-kind local_policy \
  --output "$temporary_root/symlink-inspection.json" \
  >"$temporary_root/symlink-inspection.stdout" \
  2>"$temporary_root/symlink-inspection.stderr"; then
  printf 'native inspection accepted a symlinked executable path\n' >&2
  exit 1
fi
grep -Fq 'must not be a symbolic link' "$temporary_root/symlink-inspection.stderr"
[[ ! -e "$temporary_root/symlink-inspection.json" ]]

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
    "targets.json",
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
        assert path.read_text() == "career 0.1.1\n"
        continue
    json.loads(path.read_bytes())
PY

printf 'Private npm tarball, offline extracted-install, native parity, and allowlist tests passed.\n'
