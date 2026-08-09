#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/prepare-npm-cli-packages.sh \
  --output-dir <directory> [--expected-target <triple>] [--allow-dirty]

Build, verify, stage, and locally pack the provisional private npm launcher plus
one matching native platform package. Output must be outside the checkout.
Clean source is required unless --allow-dirty is used for local tests; dirty
stages are marked non-publication candidates. This command never publishes.
EOF
}

fail() {
  printf 'npm CLI package preparation failed: %s\n' "$1" >&2
  exit 1
}

output_dir=""
expected_target=""
allow_dirty=false
while (($# > 0)); do
  case "$1" in
    --output-dir)
      (($# >= 2)) || fail "--output-dir requires a value"
      output_dir="$2"
      shift 2
      ;;
    --expected-target)
      (($# >= 2)) || fail "--expected-target requires a value"
      expected_target="$2"
      shift 2
      ;;
    --allow-dirty)
      allow_dirty=true
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

[[ -n "$output_dir" ]] || fail "--output-dir is required"
for command in cargo git node npm python3 rustc uname; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
cd "$repository_root"

kernel="$(uname -s)"
machine="$(uname -m)"
case "$kernel:$machine" in
  Darwin:arm64|Darwin:aarch64)
    platform_key="darwin-arm64"
    package_name="@revazi/career-darwin-arm64"
    target_triple="aarch64-apple-darwin"
    binary_format="mach-o-64-aarch64"
    runner_os="macOS"
    runner_arch="ARM64"
    runner_libc=""
    ;;
  Linux:x86_64|Linux:amd64)
    platform_key="linux-x64-gnu"
    package_name="@revazi/career-linux-x64-gnu"
    target_triple="x86_64-unknown-linux-gnu"
    binary_format="elf-64-x86_64"
    runner_os="Linux"
    runner_arch="X64"
    command -v getconf >/dev/null 2>&1 || fail "required command is unavailable: getconf"
    runner_libc="$(getconf GNU_LIBC_VERSION 2>/dev/null)" || fail "GNU libc version could not be determined"
    [[ "$runner_libc" =~ ^glibc\ [0-9]+\.[0-9]+ ]] || fail "native Linux package requires a confirmed glibc build host"
    ;;
  *)
    fail "unsupported native host: $kernel/$machine"
    ;;
esac

if [[ -n "$expected_target" && "$expected_target" != "$target_triple" ]]; then
  fail "native target mismatch: expected $expected_target, detected $target_triple"
fi
rust_host="$(rustc -vV | awk '/^host: / { print $2 }')"
[[ "$rust_host" == "$target_triple" ]] || fail "rustc host does not match the approved native target"

git_sha="$(git rev-parse HEAD)"
[[ "$git_sha" =~ ^[0-9a-f]{40}$ ]] || fail "Git SHA is not a full lowercase commit identifier"
git_dirty=false
if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  [[ "$allow_dirty" == true ]] || fail "source worktree is dirty; publication preparation requires a reviewed clean commit"
  git_dirty=true
fi

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
mkdir -p "$resolved_output_dir"
output_dir="$(cd "$resolved_output_dir" && pwd -P)"

launcher_source="$repository_root/npm/career"
platform_source="$repository_root/npm/platforms/$platform_key"
[[ -f "$launcher_source/package.json" && -f "$launcher_source/targets.json" && -f "$launcher_source/bin/career.js" && -f "$launcher_source/README.md" ]] || fail "launcher template is incomplete"
[[ -f "$platform_source/package.json" ]] || fail "native package template is missing"

package_version="$(node -e 'const p=require(process.argv[1]); process.stdout.write(p.version)' "$launcher_source/package.json")"
node - "$repository_root" "$launcher_source/package.json" "$launcher_source/targets.json" "$platform_source/package.json" "$package_name" "$package_version" <<'NODE'
const fs = require("node:fs");
const path = require("node:path");
const [root, launcherPath, catalogPath, platformPath, expectedPlatformName, version] = process.argv.slice(2);
const launcher = JSON.parse(fs.readFileSync(launcherPath, "utf8"));
const catalog = JSON.parse(fs.readFileSync(catalogPath, "utf8"));
const platform = JSON.parse(fs.readFileSync(platformPath, "utf8"));
if (catalog.schema_version !== "career.npm_target_catalog.v1" || !Array.isArray(catalog.targets) || catalog.targets.length !== 8) {
  throw new Error("reviewed target catalog is invalid");
}
const names = catalog.targets.map((target) => target.native_package);
const expectedOptional = Object.fromEntries(names.map((name) => [name, version]));
if (launcher.name !== "@revazi/career" || launcher.version !== version || launcher.private !== true) {
  throw new Error("launcher identity/private guard is invalid");
}
if (JSON.stringify(launcher.optionalDependencies) !== JSON.stringify(expectedOptional)) {
  throw new Error("launcher optionalDependencies are not exact, ordered, and lockstep");
}
if (JSON.stringify(launcher.career_launcher?.platform_packages) !== JSON.stringify(names)) {
  throw new Error("launcher platform_packages are not exact and ordered");
}
if (launcher.dependencies !== undefined || launcher.scripts !== undefined) {
  throw new Error("launcher runtime dependencies or lifecycle scripts are forbidden");
}
for (const target of catalog.targets) {
  const manifestPath = path.join(root, "npm", "platforms", target.platform_key, "package.json");
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  if (manifest.name !== target.native_package || manifest.version !== version || manifest.private !== true) {
    throw new Error("catalog platform identity/private guard is invalid");
  }
  if (manifest.dependencies !== undefined || manifest.optionalDependencies !== undefined || manifest.scripts !== undefined) {
    throw new Error("catalog platform dependencies or lifecycle scripts are forbidden");
  }
}
if (platform.name !== expectedPlatformName || platform.version !== version || platform.private !== true) {
  throw new Error("selected platform identity/private guard is invalid");
}
NODE

cargo build --release --locked -p career-cli --target "$target_triple"
target_root="${CARGO_TARGET_DIR:-$repository_root/target}"
if [[ "$target_root" != /* ]]; then
  target_root="$repository_root/$target_root"
fi
built_binary="$target_root/$target_triple/release/career"
[[ -f "$built_binary" && -x "$built_binary" ]] || fail "native release executable was not produced"

actual_version="$($built_binary --version)"
[[ "$actual_version" == "career $package_version" ]] || fail "native CLI and npm template versions are not lockstep"

verification_dir="$(mktemp -d "${TMPDIR:-/tmp}/career-npm-prepare.XXXXXX")"
trap 'rm -rf "$verification_dir"' EXIT
run_compact() {
  local output_name="$1"
  shift
  "$built_binary" "$@" >"$verification_dir/$output_name.json" 2>"$verification_dir/$output_name.stderr"
  [[ ! -s "$verification_dir/$output_name.stderr" ]] || fail "$output_name wrote unexpected stderr"
  python3 - "$verification_dir/$output_name.json" <<'PY'
import json
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
data = path.read_bytes()
if not data or len(data) > 33_554_432 or not data.endswith(b"\n"):
    raise SystemExit("native JSON framing/bound check failed")
json.loads(data)
PY
}

run_compact capabilities capabilities --format json-compact
run_compact operations operations --format json-compact
run_compact schema-bundle schema bundle --id career.job_match_input.v1 --format json-compact
run_compact resume-analysis resume analyze \
  --input fixtures/resume/phase3/complete-analysis.input.json --format json-compact
run_compact job-match job match \
  --input fixtures/job/phase4b/complete-match.input.json --format json-compact
"$built_binary" resume analyze \
  --input fixtures/resume/phase3/complete-analysis.input.json \
  >"$verification_dir/resume-analysis.pretty.json"
cmp "$verification_dir/resume-analysis.pretty.json" fixtures/resume/phase3/complete-analysis.expected.json
"$built_binary" job match \
  --input fixtures/job/phase4b/complete-match.input.json \
  >"$verification_dir/job-match.pretty.json"
cmp "$verification_dir/job-match.pretty.json" fixtures/job/phase4b/complete-match.expected.json

stage_root="$output_dir/stage"
launcher_stage="$stage_root/launcher"
platform_stage="$stage_root/$platform_key"
tarball_dir="$output_dir/tarballs"
mkdir -p "$launcher_stage/bin" "$platform_stage" "$tarball_dir"
cp "$launcher_source/package.json" "$launcher_stage/package.json"
cp "$launcher_source/bin/career.js" "$launcher_stage/bin/career.js"
cp "$launcher_source/targets.json" "$launcher_stage/targets.json"
cp "$launcher_source/README.md" "$launcher_stage/README.md"
chmod 0755 "$launcher_stage/bin/career.js"
cp LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md "$launcher_stage/"

cp "$platform_source/package.json" "$platform_stage/package.json"
cp "$built_binary" "$platform_stage/career"
chmod 0755 "$platform_stage/career"
cp LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md "$platform_stage/"

rustc_version="$(rustc --version)"
cargo_version="$(cargo --version)"
python3 - \
  "$launcher_source/targets.json" \
  "$platform_stage" \
  "$package_name" \
  "$package_version" \
  "$platform_key" \
  "$target_triple" \
  "$binary_format" \
  "$git_sha" \
  "$git_dirty" \
  "$rustc_version" \
  "$cargo_version" \
  "$runner_os" \
  "$runner_arch" \
  "$runner_libc" <<'PY'
import hashlib
import json
import pathlib
import stat
import sys
(
    catalog_arg,
    stage_arg,
    package_name,
    package_version,
    platform_key,
    target_triple,
    binary_format,
    git_sha,
    git_dirty_arg,
    rustc_version,
    cargo_version,
    runner_os,
    runner_arch,
    runner_libc,
) = sys.argv[1:]
catalog = json.loads(pathlib.Path(catalog_arg).read_text(encoding="utf-8"))
targets = [value for value in catalog["targets"] if value["platform_key"] == platform_key]
if len(targets) != 1:
    raise SystemExit("selected target is not unique in the reviewed catalog")
target = targets[0]
if target["native_package"] != package_name or target["rust_target"] != target_triple:
    raise SystemExit("selected target identity differs from the reviewed catalog")
if target["binary_format"] != binary_format:
    raise SystemExit("selected binary format differs from the reviewed catalog")
stage = pathlib.Path(stage_arg)
binary = stage / target["executable"]
binary_bytes = binary.read_bytes()
if not 1 <= len(binary_bytes) <= target["maximum_binary_size_bytes"]:
    raise SystemExit("native executable size is outside the catalog package bound")
if target["executable_mode"] == "0755" and stat.S_IMODE(binary.stat().st_mode) != 0o755:
    raise SystemExit("native executable mode is not exactly 0755")
provenance = {
    "schema_version": "career.npm_native_provenance.v2",
    "package": {
        "name": package_name,
        "version": package_version,
        "platform_key": platform_key,
        "node_platform": target["node_platform"],
        "node_arch": target["node_arch"],
        "libc_family": target["libc_family"],
        "rust_target": target_triple,
        "minimum_glibc_version": target["minimum_glibc_version"],
    },
    "source": {
        "repository": "https://github.com/revazi/career-core",
        "git_sha": git_sha,
        "git_ref": None,
        "git_tag": None,
        "git_dirty": git_dirty_arg == "true",
        "publication_candidate": False,
    },
    "build": {
        "command": [
            "cargo",
            "build",
            "--release",
            "--locked",
            "-p",
            "career-cli",
            "--target",
            target_triple,
        ],
        "profile": "release",
        "locked": True,
        "rustc_version": rustc_version,
        "cargo_version": cargo_version,
        "runner": {
            "os": runner_os,
            "arch": runner_arch,
            "image": "local",
            "libc": runner_libc or None,
        },
    },
    "executable": {
        "file_name": target["executable"],
        "binary_format": target["binary_format"],
        "binary_architecture": target["binary_architecture"],
        "file_invariant": target["file_invariant"],
        "archive_mode": target["archive_mode"],
        "mode": target["executable_mode"],
        "size_bytes": len(binary_bytes),
        "sha256": hashlib.sha256(binary_bytes).hexdigest(),
    },
    "integrity": {
        "npm_registry_integrity": "external_to_launcher_runtime",
        "package_contained_sha256": "consistency_only",
        "independent_signature": "absent",
    },
}
path = stage / "provenance.json"
path.write_text(json.dumps(provenance, indent=2, sort_keys=True) + "\n", encoding="utf-8")
if path.stat().st_size > 64 * 1024:
    raise SystemExit("provenance exceeds 64 KiB")
PY

python3 - "$launcher_stage" "$platform_stage" <<'PY'
import pathlib
import sys
launcher = pathlib.Path(sys.argv[1])
platform = pathlib.Path(sys.argv[2])
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
def regular_files(root):
    values = set()
    for path in root.rglob("*"):
        if path.is_symlink():
            raise SystemExit(f"staged symlink is forbidden: {path.name}")
        if path.is_file():
            values.add(path.relative_to(root).as_posix())
    return values
if regular_files(launcher) != expected_launcher:
    raise SystemExit("launcher stage allowlist mismatch")
if regular_files(platform) != expected_platform:
    raise SystemExit("platform stage allowlist mismatch")
PY

export npm_config_audit=false
export npm_config_fund=false
export npm_config_ignore_scripts=true
export npm_config_offline=true
export npm_config_update_notifier=false
export npm_config_cache="$verification_dir/npm-cache"
npm pack --ignore-scripts --json --pack-destination "$tarball_dir" "$platform_stage" \
  >"$verification_dir/platform-pack.json"
npm pack --ignore-scripts --json --pack-destination "$tarball_dir" "$launcher_stage" \
  >"$verification_dir/launcher-pack.json"

python3 - "$tarball_dir" "$platform_key" <<'PY'
import json
import pathlib
import stat
import sys
import tarfile
root = pathlib.Path(sys.argv[1])
platform_key = sys.argv[2]
tarballs = sorted(root.glob("*.tgz"))
if len(tarballs) != 2:
    raise SystemExit(f"expected two npm tarballs, found {len(tarballs)}")
expected = {
    "@revazi/career": {
        "package/package.json",
        "package/bin/career.js",
        "package/targets.json",
        "package/README.md",
        "package/LICENSE-MIT",
        "package/LICENSE-APACHE",
        "package/THIRD_PARTY_NOTICES.md",
    },
    f"@revazi/career-{platform_key}": {
        "package/package.json",
        "package/career",
        "package/provenance.json",
        "package/LICENSE-MIT",
        "package/LICENSE-APACHE",
        "package/THIRD_PARTY_NOTICES.md",
    },
}
seen = set()
for tarball in tarballs:
    if not 1 <= tarball.stat().st_size <= 32 * 1024 * 1024:
        raise SystemExit("npm tarball is outside the 32 MiB bound")
    with tarfile.open(tarball, "r:gz") as archive:
        members = archive.getmembers()
        if any(not member.isfile() for member in members):
            raise SystemExit("npm tarballs may contain only regular files")
        manifest_member = archive.getmember("package/package.json")
        manifest = json.load(archive.extractfile(manifest_member))
        name = manifest.get("name")
        lifecycle_names = {
            "preinstall",
            "install",
            "postinstall",
            "prepack",
            "prepare",
            "postpack",
            "publish",
            "postpublish",
        }
        if "scripts" in manifest or lifecycle_names.intersection(manifest):
            raise SystemExit(f"packed manifest contains lifecycle scripts: {name}")
        if manifest.get("private") is not True:
            raise SystemExit("private publication guard was removed")
        if name not in expected or name in seen:
            raise SystemExit(f"unexpected or duplicate package: {name}")
        names = {member.name for member in members}
        if names != expected[name]:
            raise SystemExit(f"tarball allowlist mismatch for {name}: {sorted(names)}")
        if name != "@revazi/career":
            binary = archive.getmember("package/career")
            if stat.S_IMODE(binary.mode) != 0o755:
                raise SystemExit("packed native executable mode is not 0755")
        seen.add(name)
if seen != set(expected):
    raise SystemExit("packed package set is incomplete")
PY

printf 'Prepared private npm launcher and native package for %s.\n' "$target_triple"
printf 'Source commit: %s (dirty=%s, publication_candidate=false)\n' "$git_sha" "$git_dirty"
printf 'Output: %s\n' "$output_dir"
