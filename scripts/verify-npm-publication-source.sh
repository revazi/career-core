#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'npm publication source verification failed: %s\n' "$1" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Usage: scripts/verify-npm-publication-source.sh \
  --repository-root <directory> \
  --expected-ref refs/tags/vX.Y.Z \
  --reviewed-sha <40-lowercase-hex>

Verify an exact clean, annotated, unmoved stable-SemVer npm release source. The
tag commit must also be the fetched origin/main commit, and all Cargo/npm
package versions must match the tag. This command does not build, pack, query a
registry, authenticate, or publish.
EOF
}

repository_root=""
expected_ref=""
reviewed_sha=""
while (($# > 0)); do
  case "$1" in
    --repository-root)
      (($# >= 2)) || fail "--repository-root requires a value"
      repository_root="$2"
      shift 2
      ;;
    --expected-ref)
      (($# >= 2)) || fail "--expected-ref requires a value"
      expected_ref="$2"
      shift 2
      ;;
    --reviewed-sha)
      (($# >= 2)) || fail "--reviewed-sha requires a value"
      reviewed_sha="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *) fail "unknown argument: $1" ;;
  esac
done

[[ -n "$repository_root" ]] || fail "--repository-root is required"
if [[ "$expected_ref" =~ ^refs/tags/v((0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*))$ ]]; then
  release_version="${BASH_REMATCH[1]}"
else
  fail "expected ref must be an exact stable SemVer tag: refs/tags/vX.Y.Z"
fi
tag_name="v$release_version"
tag_ref="refs/tags/$tag_name"
[[ "$reviewed_sha" =~ ^[0-9a-f]{40}$ ]] || fail "reviewed SHA must be 40 lowercase hexadecimal characters"
for command in git python3; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done

repository_root="$(cd "$repository_root" && pwd -P)"
cd "$repository_root"
actual_root="$(git rev-parse --show-toplevel 2>/dev/null)" || fail "repository root is not a Git worktree"
actual_root="$(cd "$actual_root" && pwd -P)"
[[ "$actual_root" == "$repository_root" ]] || fail "repository root does not match the Git worktree root"

head_sha="$(git rev-parse HEAD)"
[[ "$head_sha" == "$reviewed_sha" ]] || fail "HEAD does not match the explicitly reviewed SHA"
[[ -z "$(git status --porcelain --untracked-files=normal)" ]] || fail "source worktree is dirty"

origin_url="$(git remote get-url origin 2>/dev/null)" || fail "origin remote is missing"
case "$origin_url" in
  https://github.com/revazi/career-core|https://github.com/revazi/career-core.git) ;;
  *) fail "origin remote is not the canonical credential-free HTTPS Career Core repository" ;;
esac
origin_main_sha="$(git rev-parse refs/remotes/origin/main 2>/dev/null)" || fail "fetched origin/main is missing"
[[ "$origin_main_sha" == "$head_sha" ]] || fail "tagged source commit is not the exact fetched origin/main commit"

git show-ref --verify --quiet "$tag_ref" || fail "exact $tag_name tag is missing"
tag_type="$(git cat-file -t "$tag_ref" 2>/dev/null)" || fail "$tag_name tag object is unreadable"
[[ "$tag_type" == "tag" ]] || fail "$tag_name must be an annotated tag, not a lightweight tag"
tag_commit="$(git rev-parse "$tag_ref^{commit}")"
[[ "$tag_commit" == "$head_sha" ]] || fail "$tag_name does not resolve to the reviewed HEAD commit"

python3 - "$repository_root" "$release_version" <<'PY'
import json
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
version = sys.argv[2]
launcher_name = "@revazi/career"
catalog = json.loads((root / "npm/career/targets.json").read_text(encoding="utf-8"))
if catalog.get("schema_version") != "career.npm_target_catalog.v1" or len(catalog.get("targets", [])) != 6:
    raise SystemExit("reviewed target catalog is invalid")
native_names = [target["native_package"] for target in catalog["targets"]]
if len(native_names) != len(set(native_names)):
    raise SystemExit("reviewed target catalog contains duplicate native package names")

def load_json(path):
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise SystemExit(f"manifest must be an object: {path.name}")
    return value

cargo_versions = {}
section = None
for raw_line in (root / "Cargo.toml").read_text(encoding="utf-8").splitlines():
    line = raw_line.strip()
    if line.startswith("[") and line.endswith("]"):
        section = line[1:-1]
        continue
    if section in {"package", "workspace.package"} and line.startswith("version = "):
        cargo_versions[section] = line.split("=", 1)[1].strip().strip('"')
if cargo_versions.get("package") != version or cargo_versions.get("workspace.package") != version:
    raise SystemExit("root Cargo package/workspace versions do not match the release tag")

lock_packages = {}
for block in (root / "Cargo.lock").read_text(encoding="utf-8").split("[[package]]"):
    name_match = re.search(r'^name = "([^"]+)"$', block, re.MULTILINE)
    version_match = re.search(r'^version = "([^"]+)"$', block, re.MULTILINE)
    if name_match and version_match and name_match.group(1) in {"career-core", "career-cli", "career-swift"}:
        lock_packages[name_match.group(1)] = version_match.group(1)
if lock_packages != {name: version for name in ("career-core", "career-cli", "career-swift")}:
    raise SystemExit("Cargo.lock release package versions do not match the release tag")

launcher = load_json(root / "npm/career/package.json")
platforms = [
    load_json(root / "npm/platforms" / target["platform_key"] / "package.json")
    for target in catalog["targets"]
]
if launcher.get("name") != launcher_name or launcher.get("version") != version:
    raise SystemExit("launcher name/version does not match the release tag")
if launcher.get("private") is not True:
    raise SystemExit("source launcher template must retain private: true")
if launcher.get("os") != ["darwin", "linux"]:
    raise SystemExit("source launcher must declare the exact macOS/Linux OS boundary")
expected_optional = {name: version for name in native_names}
if launcher.get("optionalDependencies") != expected_optional:
    raise SystemExit("launcher optional dependencies are not exact and lockstep")
for manifest, expected_name in zip(platforms, native_names):
    if manifest.get("name") != expected_name or manifest.get("version") != version:
        raise SystemExit("native template name/version does not match the release tag")
    if manifest.get("private") is not True:
        raise SystemExit("source native templates must retain private: true")
PY

printf 'Verified exact clean annotated npm release source %s at %s on origin/main.\n' \
  "$tag_name" "$head_sha"
