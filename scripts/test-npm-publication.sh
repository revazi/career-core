#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'npm publication test failed: %s\n' "$1" >&2
  exit 1
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
source_gate="$script_dir/verify-npm-publication-source.sh"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/career-npm-publication-test.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT

for command in cargo git node npm python3 rustc tar; do
  command -v "$command" >/dev/null 2>&1 || fail "required command is unavailable: $command"
done
[[ "$(node --version)" == "v22.19.0" ]] || fail "tests require exact Node v22.19.0"
real_npm="$(command -v npm)"
real_rustc="$(command -v rustc)"
real_cargo="$(command -v cargo)"
[[ "$("$real_npm" --version)" == "10.9.3" || "$("$real_npm" --version)" == "11.6.2" ]] || \
  fail "tests require reviewed local npm 10.9.3 or publication npm 11.6.2"

make_minimal_fixture() {
  local root="$1"
  mkdir -p "$root/npm/career" "$root/npm/platforms"
  cp "$repository_root/Cargo.toml" "$root/Cargo.toml"
  cp "$repository_root/npm/career/package.json" "$root/npm/career/package.json"
  cp "$repository_root/npm/career/targets.json" "$root/npm/career/targets.json"
  for platform_dir in "$repository_root"/npm/platforms/*; do
    mkdir -p "$root/npm/platforms/$(basename "$platform_dir")"
    cp "$platform_dir/package.json" "$root/npm/platforms/$(basename "$platform_dir")/package.json"
  done
  git -C "$root" init -q
  git -C "$root" config user.name "Publication Fixture"
  git -C "$root" config user.email "publication-fixture@example.invalid"
  git -C "$root" add .
  git -C "$root" commit -q -m "fixture source"
  git -C "$root" remote add origin https://github.com/revazi/career-core.git
  git -C "$root" update-ref refs/remotes/origin/main "$(git -C "$root" rev-parse HEAD)"
  git -C "$root" tag -a v0.1.1 -m "fixture v0.1.1"
}

verify_fixture() {
  local root="$1"
  "$source_gate" \
    --repository-root "$root" \
    --expected-ref refs/tags/v0.1.1 \
    --reviewed-sha "$(git -C "$root" rev-parse HEAD)"
}

expect_source_failure() {
  local label="$1"
  local expected="$2"
  local root="$3"
  shift 3
  if "$source_gate" \
    --repository-root "$root" \
    --expected-ref refs/tags/v0.1.1 \
    --reviewed-sha "$(git -C "$root" rev-parse HEAD)" \
    >"$temporary_root/$label.stdout" 2>"$temporary_root/$label.stderr"; then
    fail "source gate accepted $label"
  fi
  grep -Fq "$expected" "$temporary_root/$label.stderr" || fail "unexpected $label rejection"
}

baseline="$temporary_root/source-baseline"
make_minimal_fixture "$baseline"
verify_fixture "$baseline"

fixture="$temporary_root/source-origin-without-dot-git"
make_minimal_fixture "$fixture"
git -C "$fixture" remote set-url origin https://github.com/revazi/career-core
verify_fixture "$fixture"

fixture="$temporary_root/source-credential-origin"
make_minimal_fixture "$fixture"
git -C "$fixture" remote set-url origin https://synthetic-token@github.com/revazi/career-core.git
expect_source_failure credential-origin "canonical credential-free HTTPS" "$fixture"

fixture="$temporary_root/source-dirty"
make_minimal_fixture "$fixture"
printf 'dirty\n' >>"$fixture/Cargo.toml"
expect_source_failure dirty "source worktree is dirty" "$fixture"

fixture="$temporary_root/source-missing-tag"
make_minimal_fixture "$fixture"
git -C "$fixture" tag -d v0.1.1 >/dev/null
expect_source_failure missing-tag "exact v0.1.1 tag is missing" "$fixture"

fixture="$temporary_root/source-lightweight-tag"
make_minimal_fixture "$fixture"
git -C "$fixture" tag -d v0.1.1 >/dev/null
git -C "$fixture" tag v0.1.1
expect_source_failure lightweight-tag "must be an annotated tag" "$fixture"

fixture="$temporary_root/source-moved-tag"
make_minimal_fixture "$fixture"
git -C "$fixture" commit -q --allow-empty -m "moved head"
git -C "$fixture" update-ref refs/remotes/origin/main "$(git -C "$fixture" rev-parse HEAD)"
expect_source_failure moved-tag "does not resolve to the reviewed HEAD" "$fixture"

fixture="$temporary_root/source-non-main"
make_minimal_fixture "$fixture"
old_main="$(git -C "$fixture" rev-parse refs/remotes/origin/main)"
git -C "$fixture" commit -q --allow-empty -m "non-main tag"
git -C "$fixture" tag -f -a v0.1.1 -m "non-main v0.1.1" >/dev/null
[[ "$(git -C "$fixture" rev-parse refs/remotes/origin/main)" == "$old_main" ]]
expect_source_failure non-main "not the exact fetched origin/main" "$fixture"

fixture="$temporary_root/source-version"
make_minimal_fixture "$fixture"
python3 - "$fixture/npm/career/package.json" <<'PY'
import json
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
value = json.loads(path.read_text())
value["version"] = "0.1.2"
path.write_text(json.dumps(value, indent=2) + "\n")
PY
git -C "$fixture" add .
git -C "$fixture" commit -q -m "wrong version"
git -C "$fixture" update-ref refs/remotes/origin/main "$(git -C "$fixture" rev-parse HEAD)"
git -C "$fixture" tag -f -a v0.1.1 -m "wrong version tag" >/dev/null
expect_source_failure version "launcher name/version is not the approved patch release" "$fixture"

if "$source_gate" \
  --repository-root "$baseline" \
  --expected-ref refs/tags/v0.1.2 \
  --reviewed-sha "$(git -C "$baseline" rev-parse HEAD)" \
  >"$temporary_root/wrong-ref.stdout" 2>"$temporary_root/wrong-ref.stderr"; then
  fail "source gate accepted the wrong tag/ref"
fi
grep -Fq 'expected ref must be exactly refs/tags/v0.1.1' "$temporary_root/wrong-ref.stderr"

if "$source_gate" \
  --repository-root "$baseline" \
  --expected-ref refs/tags/v0.1.1 \
  --reviewed-sha "0000000000000000000000000000000000000000" \
  >"$temporary_root/wrong-sha.stdout" 2>"$temporary_root/wrong-sha.stderr"; then
  fail "source gate accepted an unreviewed SHA"
fi
grep -Fq 'HEAD does not match the explicitly reviewed SHA' "$temporary_root/wrong-sha.stderr"

# Create a complete synthetic reviewed source from the uncommitted implementation.
fixture_repository="$temporary_root/repository"
python3 - "$repository_root" "$fixture_repository" <<'PY'
import pathlib
import shutil
import sys
source = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2])
ignored = {".git", ".fallow", "target", "__pycache__", ".build", ".swiftpm"}
shutil.copytree(source, destination, ignore=lambda _path, names: [name for name in names if name in ignored])
PY
git -C "$fixture_repository" init -q
git -C "$fixture_repository" config user.name "Publication Fixture"
git -C "$fixture_repository" config user.email "publication-fixture@example.invalid"
git -C "$fixture_repository" add .
git -C "$fixture_repository" commit -q -m "reviewed publication candidate"
git -C "$fixture_repository" remote add origin https://github.com/revazi/career-core.git
fixture_sha="$(git -C "$fixture_repository" rev-parse HEAD)"
git -C "$fixture_repository" update-ref refs/remotes/origin/main "$fixture_sha"
git -C "$fixture_repository" tag -a v0.1.1 -m "Career Core v0.1.1 fixture"

# Production scripts require npm 11.6.2. The local deterministic fixture keeps
# npm 10.9.3 compatibility by presenting the reviewed version gate while
# forwarding offline pack/install/exec behavior to the local reviewed npm.
shim_dir="$temporary_root/npm-shim"
mkdir -p "$shim_dir"
cat >"$shim_dir/npm" <<'SH'
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
  printf '11.6.2\n'
  exit 0
fi
exec "$CAREER_TEST_REAL_NPM" "$@"
SH
cat >"$shim_dir/rustc" <<'SH'
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
  printf 'rustc 1.97.1 (synthetic publication fixture)\n'
  exit 0
fi
exec "$CAREER_TEST_REAL_RUSTC" "$@"
SH
cat >"$shim_dir/cargo" <<'SH'
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
  printf 'cargo 1.97.1 (synthetic publication fixture)\n'
  exit 0
fi
exec "$CAREER_TEST_REAL_CARGO" "$@"
SH
cat >"$shim_dir/sleep" <<'SH'
#!/usr/bin/env bash
exit 0
SH
chmod 0755 "$shim_dir/npm" "$shim_dir/rustc" "$shim_dir/cargo" "$shim_dir/sleep"
export CAREER_TEST_REAL_NPM="$real_npm"
export CAREER_TEST_REAL_RUSTC="$real_rustc"
export CAREER_TEST_REAL_CARGO="$real_cargo"
export PATH="$shim_dir:$PATH"
[[ "$(npm --version)" == "11.6.2" ]]
[[ "$(rustc --version)" == rustc\ 1.97.1\ * ]]
[[ "$(cargo --version)" == cargo\ 1.97.1\ * ]]

case "$(uname -s):$(uname -m)" in
  Darwin:arm64|Darwin:aarch64)
    current_key="darwin-arm64"
    current_target="aarch64-apple-darwin"
    runner_os="macOS"
    runner_arch="ARM64"
    runner_image="macos-14"
    current_file="10-revazi-career-darwin-arm64-0.1.1.tgz"
    ;;
  Linux:x86_64|Linux:amd64)
    current_key="linux-x64-gnu"
    current_target="x86_64-unknown-linux-gnu"
    runner_os="Linux"
    runner_arch="X64"
    runner_image="ubuntu-22.04"
    current_file="30-revazi-career-linux-x64-gnu-0.1.1.tgz"
    ;;
  *) fail "candidate test requires an approved native host" ;;
esac

native_output="$temporary_root/current-native"
CARGO_TARGET_DIR="$repository_root/target" \
  "$fixture_repository/scripts/prepare-npm-publication-native.sh" \
  --output-dir "$native_output" \
  --expected-target "$current_target" \
  --expected-ref refs/tags/v0.1.1 \
  --reviewed-sha "$fixture_sha" \
  --runner-os "$runner_os" \
  --runner-arch "$runner_arch" \
  --runner-image "$runner_image"
current_tarball="$native_output/tarballs/$current_file"
[[ -f "$current_tarball" ]]

# Non-host tarballs are format-correct synthetic policy fixtures only. They are
# never native execution evidence; each release target remains blocked until its
# final package executes on the exact native OS and architecture.
native_dir="$temporary_root/native-candidates"
mkdir -p "$native_dir"
cp "$current_tarball" "$native_dir/$current_file"
python3 - "$fixture_repository" "$native_dir" "$current_key" "$fixture_sha" <<'PY'
import hashlib
import io
import json
import pathlib
import sys
import tarfile
root = pathlib.Path(sys.argv[1])
native_dir = pathlib.Path(sys.argv[2])
current_key = sys.argv[3]
sha = sys.argv[4]
catalog = json.loads((root / "npm/career/targets.json").read_text())

def synthetic_binary(target):
    binary = bytearray(512)
    architecture = target["binary_architecture"]
    if target["binary_format"].startswith("mach-o-64-"):
        binary[0:4] = (0xFEEDFACF).to_bytes(4, "little")
        binary[4:8] = (0x0100000C if architecture == "aarch64" else 0x01000007).to_bytes(4, "little")
    elif target["binary_format"].startswith("elf-64-"):
        binary[0:6] = bytes([0x7F, 0x45, 0x4C, 0x46, 2, 1])
        binary[18:20] = (0xB7 if architecture == "aarch64" else 0x3E).to_bytes(2, "little")
    else:
        binary[0:2] = b"MZ"
        binary[0x3C:0x40] = (0x80).to_bytes(4, "little")
        binary[0x80:0x84] = b"PE\0\0"
        binary[0x84:0x86] = (0xAA64 if architecture == "aarch64" else 0x8664).to_bytes(2, "little")
        binary[0x98:0x9A] = (0x020B).to_bytes(2, "little")
    binary[-1] = 1
    return bytes(binary)

def add_file(archive, name, data, mode):
    member = tarfile.TarInfo(name)
    member.size = len(data)
    member.mode = mode
    archive.addfile(member, io.BytesIO(data))

runner_images = {
    "darwin-arm64": "macos-14",
    "darwin-x64": "macos-15-intel",
    "linux-x64-gnu": "ubuntu-22.04",
    "linux-arm64-gnu": "ubuntu-24.04-arm+ubuntu:22.04",
    "linux-x64-musl": "ubuntu-22.04+node:22.19.0-alpine3.22+sha256:d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9",
    "linux-arm64-musl": "ubuntu-24.04-arm+node:22.19.0-alpine3.22+sha256:d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9",
    "win32-x64-msvc": "windows-2025",
    "win32-arm64-msvc": "windows-11-arm",
}

for index, target in enumerate(catalog["targets"], start=1):
    key = target["platform_key"]
    if key == current_key:
        continue
    manifest = json.loads((root / "npm/platforms" / key / "package.json").read_text())
    del manifest["private"]
    manifest["publishConfig"] = {"access": "public", "provenance": True}
    binary = synthetic_binary(target)
    runner_libc = (
        f"glibc {target['minimum_glibc_version']}"
        if target["libc_family"] == "glibc"
        else "musl" if target["libc_family"] == "musl" else None
    )
    provenance = {
        "schema_version": "career.npm_native_provenance.v2",
        "package": {
            "name": target["native_package"], "version": "0.1.1", "platform_key": key,
            "node_platform": target["node_platform"], "node_arch": target["node_arch"],
            "libc_family": target["libc_family"], "rust_target": target["rust_target"],
            "minimum_glibc_version": target["minimum_glibc_version"],
        },
        "source": {
            "repository": "https://github.com/revazi/career-core", "git_sha": sha,
            "git_ref": "refs/tags/v0.1.1", "git_tag": "v0.1.1",
            "git_dirty": False, "publication_candidate": True,
        },
        "build": {
            "command": ["cargo", "build", "--release", "--locked", "-p", "career-cli", "--target", target["rust_target"]],
            "profile": "release", "locked": True,
            "rustc_version": "rustc 1.97.1 (synthetic fixture)",
            "cargo_version": "cargo 1.97.1 (synthetic fixture)",
            "runner": {
                "os": target["runner_os"], "arch": target["runner_arch"],
                "image": runner_images[key], "libc": runner_libc,
            },
        },
        "executable": {
            "file_name": target["executable"], "binary_format": target["binary_format"],
            "binary_architecture": target["binary_architecture"],
            "file_invariant": target["file_invariant"], "archive_mode": target["archive_mode"],
            "mode": target["executable_mode"], "size_bytes": len(binary),
            "sha256": hashlib.sha256(binary).hexdigest(),
        },
        "integrity": {
            "npm_registry_integrity": "external_to_launcher_runtime",
            "package_contained_sha256": "consistency_only", "independent_signature": "absent",
        },
    }
    filename = f"{index * 10:02d}-revazi-career-{key}-0.1.1.tgz"
    with tarfile.open(native_dir / filename, "w:gz") as archive:
        add_file(archive, "package/package.json", (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(), 0o644)
        add_file(archive, f"package/{target['executable']}", binary, int(target["archive_mode"], 8))
        add_file(archive, "package/provenance.json", (json.dumps(provenance, indent=2, sort_keys=True) + "\n").encode(), 0o644)
        for name in ("LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"):
            add_file(archive, f"package/{name}", (root / name).read_bytes(), 0o644)
PY

candidate_dir="$temporary_root/candidate"
CARGO_TARGET_DIR="$repository_root/target" \
  "$fixture_repository/scripts/assemble-npm-publication-candidate.sh" \
  --output-dir "$candidate_dir" \
  --native-dir "$native_dir" \
  --expected-ref refs/tags/v0.1.1 \
  --reviewed-sha "$fixture_sha"
"$fixture_repository/scripts/verify-npm-publication-candidate.sh" \
  --candidate-dir "$candidate_dir" \
  --reviewed-sha "$fixture_sha" \
  --expected-target "$current_target"

acceptance_consumer="$temporary_root/public-acceptance-consumer"
mkdir -p "$acceptance_consumer"
python3 - "$acceptance_consumer/package.json" "$candidate_dir/90-revazi-career-0.1.1.tgz" "$current_tarball" "$current_key" <<'PY'
import json
import pathlib
import sys
package_names = {
    "darwin-arm64": "@revazi/career-darwin-arm64",
    "linux-x64-gnu": "@revazi/career-linux-x64-gnu",
}
path = pathlib.Path(sys.argv[1])
launcher = pathlib.Path(sys.argv[2]).resolve().as_uri()
native = pathlib.Path(sys.argv[3]).resolve().as_uri()
manifest = {
    "name": "career-public-acceptance-fixture",
    "version": "0.0.0",
    "private": True,
    "dependencies": {
        "@revazi/career": launcher,
        package_names[sys.argv[4]]: native,
    },
}
path.write_text(json.dumps(manifest) + "\n", encoding="utf-8")
PY
(
  cd "$acceptance_consumer"
  npm install --offline --ignore-scripts --no-audit --no-fund --no-package-lock >/dev/null
)
python3 "$fixture_repository/scripts/verify-npm-public-package.py" \
  --consumer-dir "$acceptance_consumer" \
  --repository-root "$fixture_repository" \
  --expected-target "$current_target"

mutate_candidate() {
  local kind="$1"
  local destination="$2"
  cp -R "$candidate_dir" "$destination"
  python3 - "$destination" "$kind" "$current_file" <<'PY'
import copy
import io
import json
import pathlib
import sys
import tarfile
root = pathlib.Path(sys.argv[1])
kind = sys.argv[2]
current_file = sys.argv[3]

def rewrite(
    tarball,
    member_mutator=None,
    add_extra=False,
    unsafe_mode_member=None,
    oversized_pax_header=False,
):
    source = tarball.with_suffix(".source")
    tarball.rename(source)
    with tarfile.open(source, "r:gz") as archive:
        entries = [(copy.copy(member), archive.extractfile(member).read()) for member in archive.getmembers()]
    options = {}
    if oversized_pax_header:
        options = {
            "format": tarfile.PAX_FORMAT,
            "pax_headers": {"comment": "x" * (20 * 1024 * 1024)},
        }
    with tarfile.open(tarball, "w:gz", **options) as archive:
        for member, data in entries:
            if member_mutator is not None:
                data = member_mutator(member.name, data)
            if member.name == unsafe_mode_member:
                member.mode = 0o666
            member.size = len(data)
            archive.addfile(member, io.BytesIO(data))
        if add_extra:
            info = tarfile.TarInfo("package/unreviewed-extra")
            data = b"extra\n"
            info.size = len(data)
            info.mode = 0o644
            archive.addfile(info, io.BytesIO(data))
    source.unlink()

def mutate_json_member(member_name, expected, mutate):
    def apply(name, data):
        if name == member_name:
            value = json.loads(data)
            mutate(value)
            return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()
        return data
    return apply

def reorder_launcher_packages(name, data):
    if name != "package/package.json":
        return data
    value = json.loads(data)
    optional = value["optionalDependencies"]
    value["optionalDependencies"] = {key: optional[key] for key in reversed(optional)}
    return (json.dumps(value, indent=2) + "\n").encode()

if kind == "wrong-sha":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["source"].update(git_sha="f" * 40)))
elif kind == "wrong-target":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["package"].update(rust_target="x86_64-pc-windows-msvc")))
elif kind == "provenance-mismatch":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["source"].update(publication_candidate=False)))
elif kind == "wrong-toolchain":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["build"].update(rustc_version="rustc 1.96.0 (wrong)")))
elif kind == "glibc-floor":
    linux = root / "30-revazi-career-linux-x64-gnu-0.1.1.tgz"
    rewrite(linux, mutate_json_member("package/package.json", None, lambda v: v["career_native"].update(minimum_glibc_version="2.34")))
elif kind == "wrong-version":
    rewrite(root / "90-revazi-career-0.1.1.tgz", mutate_json_member("package/package.json", None, lambda v: v.update(version="0.1.2")))
elif kind == "lifecycle":
    rewrite(root / "90-revazi-career-0.1.1.tgz", mutate_json_member("package/package.json", None, lambda v: v.update(scripts={"postinstall": "node download.js"})))
elif kind == "dependency":
    rewrite(root / current_file, mutate_json_member("package/package.json", None, lambda v: v.update(dependencies={"download": "1.0.0"})))
elif kind == "allowlist":
    rewrite(root / current_file, add_extra=True)
elif kind == "oversized-member":
    rewrite(
        root / "10-revazi-career-darwin-arm64-0.1.1.tgz",
        lambda name, data: b"x" * (16 * 1024 * 1024 + 1)
        if name == "package/career"
        else data,
    )
elif kind == "oversized-json":
    rewrite(
        root / "90-revazi-career-0.1.1.tgz",
        mutate_json_member(
            "package/package.json", None, lambda value: value.update(description="x" * (40 * 1024))
        ),
    )
elif kind == "compressed-bomb":
    rewrite(root / current_file, oversized_pax_header=True)
elif kind == "extra-property":
    rewrite(root / "90-revazi-career-0.1.1.tgz", mutate_json_member("package/package.json", None, lambda v: v.update(unreviewed=True)))
elif kind == "unsafe-mode":
    rewrite(root / "90-revazi-career-0.1.1.tgz", unsafe_mode_member="package/LICENSE-MIT")
elif kind == "launcher-bytes":
    rewrite(root / "90-revazi-career-0.1.1.tgz", lambda name, data: b"tampered\n" if name == "package/bin/career.js" else data)
elif kind == "readme-bytes":
    rewrite(root / "90-revazi-career-0.1.1.tgz", lambda name, data: b"tampered\n" if name == "package/README.md" else data)
elif kind == "launcher-package-order":
    rewrite(root / "90-revazi-career-0.1.1.tgz", reorder_launcher_packages)
elif kind == "launcher-order":
    path = root / "publication-manifest.json"
    value = json.loads(path.read_text())
    value["packages"] = [value["packages"][2], value["packages"][0], value["packages"][1]]
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
elif kind == "oversized-publication-manifest":
    path = root / "publication-manifest.json"
    path.write_text(json.dumps({"oversized": "x" * (70 * 1024)}) + "\n")
elif kind == "extra-directory":
    (root / "nested").mkdir()
elif kind == "extra-fifo":
    import os
    os.mkfifo(root / "unexpected.fifo")
else:
    raise SystemExit(f"unsupported mutation: {kind}")
PY
}

for mutation in \
  wrong-sha wrong-target provenance-mismatch wrong-toolchain glibc-floor wrong-version lifecycle dependency allowlist \
  oversized-member oversized-json compressed-bomb extra-property unsafe-mode launcher-bytes readme-bytes \
  launcher-package-order launcher-order oversized-publication-manifest extra-directory extra-fifo
do
  mutated="$temporary_root/mutated-$mutation"
  mutate_candidate "$mutation" "$mutated"
  if "$fixture_repository/scripts/npm-publication-candidate.py" verify \
    "$mutated" --source-sha "$fixture_sha" --repository-root "$fixture_repository" \
    >"$temporary_root/$mutation.stdout" 2>"$temporary_root/$mutation.stderr"; then
    fail "candidate verification accepted mutation: $mutation"
  fi
done

# The independently checked publication driver must reject bounded-archive
# attacks before invoking any npm registry command.
driver_guard_bin="$temporary_root/driver-guard-bin"
driver_guard_marker="$temporary_root/driver-guard-contacted"
mkdir -p "$driver_guard_bin"
cat >"$driver_guard_bin/npm" <<'SH'
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
  printf '11.6.2\n'
  exit 0
fi
: >"$CAREER_TEST_NPM_GUARD_MARKER"
exit 97
SH
chmod 0755 "$driver_guard_bin/npm"
for mutation in allowlist oversized-member oversized-json compressed-bomb oversized-publication-manifest; do
  rm -f "$driver_guard_marker"
  if PATH="$driver_guard_bin:$PATH" \
    NODE_AUTH_TOKEN=synthetic-test-only \
    CAREER_TEST_NPM_GUARD_MARKER="$driver_guard_marker" \
    "$fixture_repository/scripts/publish-npm-publication-candidate.sh" \
      --mode bootstrap \
      --candidate-dir "$temporary_root/mutated-$mutation" \
      --reviewed-sha "$fixture_sha" \
      >"$temporary_root/driver-$mutation.stdout" \
      2>"$temporary_root/driver-$mutation.stderr"; then
    fail "publication driver accepted bounded-archive mutation: $mutation"
  fi
  [[ ! -e "$driver_guard_marker" ]] || fail "publication driver contacted npm before rejecting $mutation"
done

cat >"$shim_dir/npm" <<'PY'
#!/usr/bin/env python3
import base64
import hashlib
import json
import os
import pathlib
import sys
import tarfile

state_path = pathlib.Path(os.environ["FAKE_NPM_STATE"])
log_path = pathlib.Path(os.environ["FAKE_NPM_LOG"])
attempt_path = pathlib.Path(os.environ["FAKE_NPM_ATTEMPTS"])
args = sys.argv[1:]
if args == ["--version"]:
    print("11.6.2")
    raise SystemExit(0)
userconfig = pathlib.Path(os.environ["NPM_CONFIG_USERCONFIG"])
if not userconfig.is_file() or (userconfig.stat().st_mode & 0o777) != 0o600:
    print("npm ERR! unsafe userconfig", file=sys.stderr)
    raise SystemExit(90)
config = userconfig.read_text()
if os.environ.get("NODE_AUTH_TOKEN"):
    if "_authToken=${NODE_AUTH_TOKEN}" not in config or os.environ["NODE_AUTH_TOKEN"] in config:
        print("npm ERR! unsafe bootstrap token config", file=sys.stderr)
        raise SystemExit(91)
elif "_authToken" in config:
    print("npm ERR! OIDC mode received an auth token npmrc", file=sys.stderr)
    raise SystemExit(92)
state = json.loads(state_path.read_text())
packages = state.setdefault("packages", {})
with log_path.open("a") as log:
    log.write("command\t" + "\t".join(args[:3]) + "\n")
if args and args[0] == "view":
    spec = args[1]
    field = args[2]
    if field == "name":
        if spec in packages:
            print(json.dumps(spec))
            raise SystemExit(0)
    elif field in {"dist.integrity", "dist.attestations"}:
        name, version = spec.rsplit("@", 1)
        record = packages.get(name, {}).get(version)
        if isinstance(record, dict):
            selected = json.loads(os.environ.get("FAKE_NPM_BEHAVIOR", "{}")).get(name)
            if field == "dist.integrity" and selected in {"integrity_delayed_six", "integrity_never"}:
                attempts = json.loads(attempt_path.read_text()) if attempt_path.exists() else {}
                key = f"integrity:{name}"
                attempts[key] = attempts.get(key, 0) + 1
                attempt_path.write_text(json.dumps(attempts))
                if selected == "integrity_never" or attempts[key] <= 6:
                    print("npm ERR! code E404", file=sys.stderr)
                    raise SystemExit(1)
            attestation_delays = {"attestation_delayed_once": 1, "attestation_delayed_six": 6}
            if field == "dist.attestations" and selected in {*attestation_delays, "attestation_never"}:
                attempts = json.loads(attempt_path.read_text()) if attempt_path.exists() else {}
                key = f"attestation:{name}"
                attempts[key] = attempts.get(key, 0) + 1
                attempt_path.write_text(json.dumps(attempts))
                if selected == "attestation_never" or attempts[key] <= attestation_delays[selected]:
                    print("npm ERR! code E404", file=sys.stderr)
                    raise SystemExit(1)
            value = record.get("integrity" if field == "dist.integrity" else "attestations")
            if value is not None:
                print(json.dumps(value))
                raise SystemExit(0)
    print("npm ERR! code E404", file=sys.stderr)
    raise SystemExit(1)
if args and args[0] == "publish":
    tarball = pathlib.Path(args[1])
    with tarfile.open(tarball, "r:gz") as archive:
        manifest = json.load(archive.extractfile("package/package.json"))
    name = manifest["name"]
    version = manifest["version"]
    with log_path.open("a") as log:
        log.write(f"publish\t{name}\n")
    attempts = json.loads(attempt_path.read_text()) if attempt_path.exists() else {}
    attempts[name] = attempts.get(name, 0) + 1
    attempt_path.write_text(json.dumps(attempts))
    behavior = json.loads(os.environ.get("FAKE_NPM_BEHAVIOR", "{}"))
    selected = behavior.get(name)
    if selected == "transient_once" and attempts[name] == 1:
        print("npm ERR! 503 Service Unavailable")
        raise SystemExit(1)
    if selected == "auth":
        print("npm ERR! code E401")
        raise SystemExit(1)
    if selected == "permanent":
        print("npm ERR! permanent publication failure")
        raise SystemExit(1)
    data = tarball.read_bytes()
    integrity = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode("ascii")
    packages.setdefault(name, {})[version] = {
        "integrity": integrity,
        "attestations": {
            "url": f"https://registry.npmjs.org/-/npm/v1/attestations/{name}@{version}",
            "provenance": {"predicateType": "https://slsa.dev/provenance/v1"},
        },
    }
    state_path.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n")
    print(f"+ {name}@{version}")
    raise SystemExit(0)
print("npm ERR! unsupported fake command", file=sys.stderr)
raise SystemExit(93)
PY
chmod 0755 "$shim_dir/npm"

publication_driver="$repository_root/scripts/publish-npm-publication-candidate.sh"
driver_root="$temporary_root/driver"
mkdir -p "$driver_root"
write_driver_state() {
  local scenario="$1" destination="$2"
  python3 - "$candidate_dir/publication-manifest.json" "$scenario" "$destination" <<'PY'
import json
import pathlib
import sys
manifest = json.loads(pathlib.Path(sys.argv[1]).read_text())
scenario = sys.argv[2]
path = pathlib.Path(sys.argv[3])
exact = {
    row["name"]: {
        row["version"]: {
            "integrity": row["integrity"],
            "attestations": {
                "url": f"https://registry.npmjs.org/-/npm/v1/attestations/{row['name']}@{row['version']}",
                "provenance": {"predicateType": "https://slsa.dev/provenance/v1"},
            },
        }
    }
    for row in manifest["packages"]
}
if scenario == "absent": packages = {}
elif scenario == "exact": packages = exact
elif scenario == "partial-first": packages = {manifest["packages"][0]["name"]: exact[manifest["packages"][0]["name"]]}
elif scenario == "partial-second": packages = {row["name"]: exact[row["name"]] for row in manifest["packages"][:2]}
elif scenario == "name-without-version": packages = {manifest["packages"][0]["name"]: {}}
elif scenario == "conflict":
    packages = {manifest["packages"][0]["name"]: {"0.1.1": {"integrity": "sha512-conflict", "attestations": {}}}}
elif scenario == "oidc-ready": packages = {row["name"]: {} for row in manifest["packages"]}
elif scenario == "missing-attestation":
    packages = exact
    packages[manifest["packages"][0]["name"]]["0.1.1"]["attestations"] = None
elif scenario == "malformed-attestation":
    packages = exact
    packages[manifest["packages"][0]["name"]]["0.1.1"]["attestations"] = {
        "url": "http://registry.npmjs.org/untrusted",
        "provenance": {"predicateType": "https://example.invalid/not-slsa"},
    }
else: raise SystemExit(f"unknown scenario: {scenario}")
path.write_text(json.dumps({"packages": packages}, indent=2, sort_keys=True) + "\n")
PY
}

run_driver() {
  local label="$1" mode="$2" state_scenario="$3" behavior="$4" token_mode="$5"
  local case_root="$driver_root/$label"
  mkdir -p "$case_root"
  export FAKE_NPM_STATE="$case_root/state.json"
  export FAKE_NPM_LOG="$case_root/npm.log"
  export FAKE_NPM_ATTEMPTS="$case_root/attempts.json"
  export FAKE_NPM_BEHAVIOR="$behavior"
  : >"$FAKE_NPM_LOG"
  write_driver_state "$state_scenario" "$FAKE_NPM_STATE"
  if [[ "$token_mode" == "token" ]]; then
    NODE_AUTH_TOKEN="synthetic-bootstrap-token" "$publication_driver" \
      --mode "$mode" --candidate-dir "$candidate_dir" --reviewed-sha "$fixture_sha"
  else
    env -u NODE_AUTH_TOKEN "$publication_driver" \
      --mode "$mode" --candidate-dir "$candidate_dir" --reviewed-sha "$fixture_sha"
  fi
}

run_driver bootstrap-all bootstrap absent '{}' token
python3 - "$driver_root/bootstrap-all/npm.log" <<'PY'
import pathlib, sys
published = [line.split("\t", 1)[1] for line in pathlib.Path(sys.argv[1]).read_text().splitlines() if line.startswith("publish\t")]
assert published == [
    "@revazi/career-darwin-arm64",
    "@revazi/career-darwin-x64",
    "@revazi/career-linux-x64-gnu",
    "@revazi/career-linux-arm64-gnu",
    "@revazi/career-linux-x64-musl",
    "@revazi/career-linux-arm64-musl",
    "@revazi/career-win32-x64-msvc",
    "@revazi/career-win32-arm64-msvc",
    "@revazi/career",
]
PY
run_driver bootstrap-idempotent bootstrap exact '{}' token
! grep -q '^publish' "$driver_root/bootstrap-idempotent/npm.log"
run_driver attestation-eventual bootstrap exact \
  '{"@revazi/career-darwin-arm64":"attestation_delayed_once"}' token
python3 - "$driver_root/attestation-eventual/attempts.json" <<'PY'
import json, pathlib, sys
attempts = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert attempts["attestation:@revazi/career-darwin-arm64"] >= 2
PY
run_driver integrity-eventual bootstrap absent \
  '{"@revazi/career-darwin-arm64":"integrity_delayed_six"}' token
python3 - "$driver_root/integrity-eventual/attempts.json" <<'PY'
import json, pathlib, sys
attempts = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert attempts["integrity:@revazi/career-darwin-arm64"] >= 7
PY
if run_driver integrity-never bootstrap absent \
  '{"@revazi/career-darwin-arm64":"integrity_never"}' token \
  >"$driver_root/integrity-never.stdout" 2>"$driver_root/integrity-never.stderr"; then
  fail "publication driver accepted registry integrity that never became visible"
fi
python3 - "$driver_root/integrity-never/attempts.json" "$driver_root/integrity-never/npm.log" <<'PY'
import json, pathlib, sys
attempts = json.loads(pathlib.Path(sys.argv[1]).read_text())
published = [line.split("\t", 1)[1] for line in pathlib.Path(sys.argv[2]).read_text().splitlines() if line.startswith("publish\t")]
assert attempts["integrity:@revazi/career-darwin-arm64"] == 61
assert published == ["@revazi/career-darwin-arm64"]
PY
run_driver provenance-eventual bootstrap exact \
  '{"@revazi/career-darwin-arm64":"attestation_delayed_six"}' token
python3 - "$driver_root/provenance-eventual/attempts.json" <<'PY'
import json, pathlib, sys
attempts = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert attempts["attestation:@revazi/career-darwin-arm64"] >= 7
PY
if run_driver provenance-never bootstrap exact \
  '{"@revazi/career-darwin-arm64":"attestation_never"}' token \
  >"$driver_root/provenance-never.stdout" 2>"$driver_root/provenance-never.stderr"; then
  fail "publication driver accepted registry provenance that never became visible"
fi
python3 - "$driver_root/provenance-never/attempts.json" "$driver_root/provenance-never/npm.log" <<'PY'
import json, pathlib, sys
attempts = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert attempts["attestation:@revazi/career-darwin-arm64"] == 61
assert not any(line.startswith("publish\t") for line in pathlib.Path(sys.argv[2]).read_text().splitlines())
PY
for scenario in missing-attestation malformed-attestation; do
  if run_driver "$scenario" bootstrap "$scenario" '{}' token \
    >"$driver_root/$scenario.stdout" 2>"$driver_root/$scenario.stderr"; then
    fail "publication driver accepted invalid registry provenance: $scenario"
  fi
  ! grep -q '^publish' "$driver_root/$scenario/npm.log"
done
run_driver bootstrap-partial-first bootstrap partial-first '{}' token
python3 - "$driver_root/bootstrap-partial-first/npm.log" <<'PY'
import pathlib, sys
published = [line.split("\t", 1)[1] for line in pathlib.Path(sys.argv[1]).read_text().splitlines() if line.startswith("publish\t")]
assert published == [
    "@revazi/career-darwin-x64",
    "@revazi/career-linux-x64-gnu",
    "@revazi/career-linux-arm64-gnu",
    "@revazi/career-linux-x64-musl",
    "@revazi/career-linux-arm64-musl",
    "@revazi/career-win32-x64-msvc",
    "@revazi/career-win32-arm64-msvc",
    "@revazi/career",
]
PY
run_driver bootstrap-partial-second bootstrap partial-second '{}' token
grep -Fxq $'publish\t@revazi/career' "$driver_root/bootstrap-partial-second/npm.log"
! grep -Fq '@revazi/career-darwin-arm64' <(grep '^publish' "$driver_root/bootstrap-partial-second/npm.log" || true)

for scenario in name-without-version conflict; do
  if run_driver "bootstrap-$scenario" bootstrap "$scenario" '{}' token \
    >"$driver_root/$scenario.stdout" 2>"$driver_root/$scenario.stderr"; then
    fail "bootstrap accepted conflicting preflight state: $scenario"
  fi
  ! grep -q '^publish' "$driver_root/bootstrap-$scenario/npm.log"
done

if run_driver bootstrap-no-token bootstrap absent '{}' no-token \
  >"$driver_root/bootstrap-no-token.stdout" 2>"$driver_root/bootstrap-no-token.stderr"; then
  fail "bootstrap accepted a missing token"
fi
if NODE_AUTH_TOKEN="forbidden" run_driver oidc-with-token oidc oidc-ready '{}' token \
  >"$driver_root/oidc-with-token.stdout" 2>"$driver_root/oidc-with-token.stderr"; then
  fail "OIDC mode accepted token authentication"
fi
if run_driver oidc-absent oidc absent '{}' no-token \
  >"$driver_root/oidc-absent.stdout" 2>"$driver_root/oidc-absent.stderr"; then
  fail "OIDC mode accepted missing trusted-publisher package names"
fi
run_driver oidc-ready oidc oidc-ready '{}' no-token

run_driver transient bootstrap absent '{"@revazi/career-darwin-arm64":"transient_once"}' token
python3 - "$driver_root/transient/attempts.json" <<'PY'
import json, pathlib, sys
attempts = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert attempts["@revazi/career-darwin-arm64"] == 2
for name in [
    "@revazi/career-darwin-x64",
    "@revazi/career-linux-x64-gnu",
    "@revazi/career-linux-arm64-gnu",
    "@revazi/career-linux-x64-musl",
    "@revazi/career-linux-arm64-musl",
    "@revazi/career-win32-x64-msvc",
    "@revazi/career-win32-arm64-msvc",
    "@revazi/career",
]:
    assert attempts[name] == 1
PY
if run_driver native-failure bootstrap absent '{"@revazi/career-linux-x64-gnu":"permanent"}' token \
  >"$driver_root/native-failure.stdout" 2>"$driver_root/native-failure.stderr"; then
  fail "publication driver accepted a failed native package"
fi
! grep -Fxq $'publish\t@revazi/career' "$driver_root/native-failure/npm.log"
if run_driver auth-failure bootstrap absent '{"@revazi/career-darwin-arm64":"auth"}' token \
  >"$driver_root/auth-failure.stdout" 2>"$driver_root/auth-failure.stderr"; then
  fail "publication driver silently fell back after authentication failure"
fi
! grep -Fq '@revazi/career-linux-x64-gnu' <(grep '^publish' "$driver_root/auth-failure/npm.log" || true)

python3 - \
  "$repository_root/.github/workflows/npm-publish.yml" \
  "$repository_root/.github/workflows/npm-publish-v0.1.1.yml" \
  "$publication_driver" <<'PY'
import hashlib
import pathlib
import sys
historical_path = pathlib.Path(sys.argv[1])
path = pathlib.Path(sys.argv[2])
driver_path = pathlib.Path(sys.argv[3])
historical = historical_path.read_bytes()
text = path.read_text(encoding="utf-8")
driver = driver_path.read_text(encoding="utf-8")
if hashlib.sha256(historical).hexdigest() != "4b085e8a71a527ccf800ca218dab053febe95ad8fcdb3edbbd86c231ccf55414":
    raise SystemExit("historical v0.1.0 publication workflow bytes changed")
required = [
    "workflow_dispatch:", "bootstrap:", "default: false", "npm-production",
    "node-version: '22.19.0'", "npm@11.6.2", "id-token: write",
    "ubuntu-22.04", "ubuntu-24.04-arm", "macos-14", "macos-15-intel",
    "windows-2025", "windows-11-arm", "1.97.1", "1.85.0",
    "node:22.19.0-alpine3.22@sha256:d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9",
    "scripts/publish-npm-publication-candidate.sh",
    "scripts/verify-npm-public-package.py",
    "public-acceptance-unix:", "public-acceptance-musl:",
    "public-acceptance-windows:", "dist.attestations",
    "actions/upload-artifact@bbbca2ddaa5d8feaa63e36b76fdaad77386f024f",
    "actions/download-artifact@70fc10c6e5e1ce46ad2ea6f2b72d43f7d47b13c3",
    "timeout-minutes: 110", "registry_visibility_attempts=61",
    "require_registry_package_ready",
    "10-revazi-career-darwin-arm64-0.1.1.tgz",
    "20-revazi-career-darwin-x64-0.1.1.tgz",
    "30-revazi-career-linux-x64-gnu-0.1.1.tgz",
    "40-revazi-career-linux-arm64-gnu-0.1.1.tgz",
    "50-revazi-career-linux-x64-musl-0.1.1.tgz",
    "60-revazi-career-linux-arm64-musl-0.1.1.tgz",
    "70-revazi-career-win32-x64-msvc-0.1.1.tgz",
    "80-revazi-career-win32-arm64-msvc-0.1.1.tgz",
    "90-revazi-career-0.1.1.tgz",
]
for value in required:
    if value not in text and value not in driver:
        raise SystemExit(f"publication workflow/driver is missing required policy text: {value}")
if "name: Publish npm CLI v0.1.1" not in text or "refs/tags/v0.1.1" not in text:
    raise SystemExit("protected v0.1.1 publication workflow identity is not exact")
for value in (
    "--access public", "--provenance", "--ignore-scripts", "_authToken=${NODE_AUTH_TOKEN}",
    "dist.attestations", "https://slsa.dev/provenance/v1",
):
    if value not in driver:
        raise SystemExit(f"publication driver is missing required policy text: {value}")
for stale_function in ("require_registry_integrity()", "require_registry_provenance()"):
    if stale_function in driver:
        raise SystemExit(f"publication driver has separate visibility windows: {stale_function}")
for forbidden in (
    "pull_request:", "schedule:", "release:", "cargo publish", "gh release",
    "NPM_TOKEN ||", "registry-url:", "ubuntu-latest", "rustup update stable",
    "043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
    "3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c",
):
    if forbidden in text:
        raise SystemExit(f"publication workflow contains forbidden policy text: {forbidden}")
if text.count("secrets.NPM_TOKEN") != 1:
    raise SystemExit("bootstrap token must appear in exactly one explicit workflow step")
if text.index("Configure exact release Rust for publication policy") > text.index("Run registry-free publication and adversarial tests"):
    raise SystemExit("publication tests run before exact Rust 1.97.1 setup")
for line in text.splitlines():
    stripped = line.strip()
    if stripped.startswith("uses: actions/"):
        ref = stripped.rsplit("@", 1)[-1].split()[0]
        if len(ref) != 40 or any(character not in "0123456789abcdef" for character in ref):
            raise SystemExit(f"GitHub-owned action is not pinned by immutable SHA: {stripped}")
if driver.index("10-revazi-career-darwin-arm64-0.1.1.tgz") > driver.index("90-revazi-career-0.1.1.tgz"):
    raise SystemExit("publication driver does not encode native-before-launcher ordering")
PY

[[ "$(cat "$repository_root/.gitattributes")" == "* text=auto eol=lf" ]] || {
  printf 'repository checkout text normalization policy is not exact\n' >&2
  exit 1
}

python3 - "$repository_root/.github/workflows/npm-cli-packages.yml" <<'PY'
import pathlib
import sys
text = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
ordered = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
]
positions = [text.index(value) for value in ordered]
if positions != sorted(positions):
    raise SystemExit("private native evidence targets are not in exact catalog order")
for value in (
    "macos-14",
    "macos-15-intel",
    "ubuntu-22.04",
    "ubuntu-24.04-arm",
    "container: ubuntu:22.04",
    "diffutils",
    'safe.directory "$GITHUB_WORKSPACE"',
    "node:22.19.0-alpine3.22@sha256:d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9",
    "docker run --rm --interactive",
    "musl libc ($EXPECTED_MACHINE)",
    "Version 1.2.5",
    "expected_linkage: static-pie",
    "expected_linkage: static",
    "chmod 0644 /output/evidence.json",
    "windows-2025",
    "windows-11-arm",
    "RuntimeInformation]::ProcessArchitecture",
    "scripts/prepare-npm-cli-packages-windows.py",
    "scripts/test-npm-cli-packages-windows.py",
    "career.exe",
    "scripts/inspect-npm-native-binary.py",
    'test "$(node --version)" = "v22.19.0"',
    "--evidence-kind exact_native_ci",
    "Confirm workflow remains preparation-only",
):
    if value not in text:
        raise SystemExit(f"private native evidence workflow is missing policy text: {value}")
for forbidden in (
    "pull_request:",
    "schedule:",
    "macos-latest",
    "ubuntu-latest",
    "windows-latest",
    "continue-on-error:",
    "qemu",
    "--platform",
    "actions/upload-artifact",
    "npm publish",
    "cargo publish",
):
    if forbidden in text:
        raise SystemExit(f"private native evidence workflow contains forbidden text: {forbidden}")
for line in text.splitlines():
    stripped = line.strip()
    if stripped.startswith("uses: actions/"):
        ref = stripped.rsplit("@", 1)[-1].split()[0]
        if len(ref) != 40 or any(character not in "0123456789abcdef" for character in ref):
            raise SystemExit(f"private evidence action is not SHA-pinned: {stripped}")
PY

python3 - \
  "$repository_root/scripts/inspect-npm-native-binary.py" \
  "$repository_root/scripts/prepare-npm-publication-native.sh" <<'PY'
import pathlib
import sys
inspection = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
preparation = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
for value in (
    '"aarch64-apple-darwin": "macos-14"',
    '"x86_64-apple-darwin": "macos-15-intel"',
    '"x86_64-unknown-linux-gnu": "ubuntu-22.04"',
    '"aarch64-unknown-linux-gnu": "ubuntu-24.04-arm+ubuntu:22.04"',
    '"x86_64-unknown-linux-musl"',
    '"aarch64-unknown-linux-musl"',
    '"x86_64-pc-windows-msvc": "windows-2025"',
    '"aarch64-pc-windows-msvc": "windows-11-arm"',
    "inspect_linux_musl",
    "inspect_windows",
    '"musl 1.2.5"',
    "MAX_COMMAND_OUTPUT_BYTES",
    "MAX_EVIDENCE_BYTES",
    "verify_unchanged_bytes",
    "O_NOFOLLOW",
    "O_BINARY",
    '"glibc 2.35"',
    '"career.npm_native_inspection.v1"',
):
    if value not in inspection:
        raise SystemExit(f"native inspection script is missing fail-closed policy text: {value}")
for forbidden in (
    "requests",
    "urllib",
    "http.client",
    "qemu",
    "--platform",
    "st_ino != 0",
):
    if forbidden in inspection.lower():
        raise SystemExit(f"native inspection script contains forbidden mechanism: {forbidden}")
expected_dynamic_symbol_line = (
    '  dynamic_symbols="$(LC_ALL=C readelf --dyn-syms --wide '
    '"$platform_stage/career")"'
)
if preparation.splitlines().count(expected_dynamic_symbol_line) != 1:
    raise SystemExit("musl candidate dynamic-symbol inspection line is not exact")
for value in (
    "20-revazi-career-darwin-x64-0.1.1.tgz",
    "40-revazi-career-linux-arm64-gnu-0.1.1.tgz",
    "50-revazi-career-linux-x64-musl-0.1.1.tgz",
    "60-revazi-career-linux-arm64-musl-0.1.1.tgz",
    "70-revazi-career-win32-x64-msvc-0.1.1.tgz",
    "80-revazi-career-win32-arm64-msvc-0.1.1.tgz",
    "x86_64-apple-darwin)",
    "aarch64-unknown-linux-gnu)",
    "x86_64-unknown-linux-musl)",
    "aarch64-unknown-linux-musl)",
    "x86_64-pc-windows-msvc)",
    "aarch64-pc-windows-msvc)",
    "prepare-npm-cli-packages-windows.py",
    "--evidence-kind exact_native_ci",
    'readelf --file-header --wide "$platform_stage/career"',
    "Type:[[:space:]]+DYN",
    "Type:[[:space:]]+EXEC",
    "Flags: NOW PIE",
    "x86-64 musl candidate is not one reviewed static PIE",
    "AArch64 musl candidate is not one reviewed static executable",
):
    if value not in preparation:
        raise SystemExit(f"native publication preparation is missing target policy text: {value}")
PY

python3 - \
  "$repository_root/scripts/inspect-npm-native-binary.py" \
  "$repository_root/scripts/prepare-npm-cli-packages-windows.py" \
  "$repository_root/scripts/test-npm-cli-packages-windows.py" \
  "$repository_root/scripts/npm_windows_process.py" <<'PY'
import importlib.util
import pathlib
import sys
sys.dont_write_bytecode = True
inspection_path, preparation_path, test_path, process_path = map(pathlib.Path, sys.argv[1:])
for path in (preparation_path, test_path):
    text = path.read_text(encoding="utf-8")
    for value in (
        "Windows",
        "career.exe",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
    ):
        if value not in text:
            raise SystemExit(f"Windows package script is missing policy text: {path.name}: {value}")
    for forbidden in ("requests", "urllib", "http.client", "qemu", "--platform"):
        if forbidden in text.lower():
            raise SystemExit(f"Windows package script contains forbidden mechanism: {path.name}: {forbidden}")
test_source = test_path.read_text(encoding="utf-8")
if r"node_modules\.bin\career.cmd --version" not in test_source:
    raise SystemExit("Windows package test does not use the fixed relative npm shim path")
preparation = preparation_path.read_text(encoding="utf-8")
for value in ("windows_regular_non_symlink_exe", "0644"):
    if value not in preparation:
        raise SystemExit(f"Windows preparation is missing file policy text: {value}")
process_source = process_path.read_text(encoding="utf-8")
for value in ("TemporaryFile", "Popen", "maximum_output_bytes", "process.kill()"):
    if value not in process_source:
        raise SystemExit(f"Windows bounded process helper is missing policy text: {value}")
spec = importlib.util.spec_from_file_location("career_native_inspection", inspection_path)
if spec is None or spec.loader is None:
    raise SystemExit("could not load native inspection policy")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
observed_windows_api_imports = (
    "api-ms-win-core-synch-l1-2-0.dll",
    "api-ms-win-crt-heap-l1-1-0.dll",
    "api-ms-win-crt-locale-l1-1-0.dll",
    "api-ms-win-crt-math-l1-1-0.dll",
    "api-ms-win-crt-runtime-l1-1-0.dll",
    "api-ms-win-crt-stdio-l1-1-0.dll",
    "bcryptprimitives.dll",
)
if not all(module.approved_windows_import(value) for value in observed_windows_api_imports):
    raise SystemExit("exact observed Windows system imports are not reviewed")
if module.approved_windows_import("api-ms-win-evil.dll"):
    raise SystemExit("arbitrary Windows API-set import was accepted")
binary = bytearray(1024)
binary[0:2] = b"MZ"
binary[0x3C:0x40] = (0x80).to_bytes(4, "little")
pe = 0x80
binary[pe:pe + 4] = b"PE\0\0"
binary[pe + 4:pe + 6] = (0x8664).to_bytes(2, "little")
binary[pe + 6:pe + 8] = (1).to_bytes(2, "little")
binary[pe + 20:pe + 22] = (240).to_bytes(2, "little")
binary[pe + 22:pe + 24] = (2).to_bytes(2, "little")
optional = pe + 24
binary[optional:optional + 2] = (0x020B).to_bytes(2, "little")
binary[optional + 60:optional + 64] = (0x200).to_bytes(4, "little")
binary[optional + 108:optional + 112] = (16).to_bytes(4, "little")
binary[optional + 120:optional + 124] = (0x1000).to_bytes(4, "little")
binary[optional + 124:optional + 128] = (40).to_bytes(4, "little")
section = optional + 240
binary[section + 8:section + 12] = (0x200).to_bytes(4, "little")
binary[section + 12:section + 16] = (0x1000).to_bytes(4, "little")
binary[section + 16:section + 20] = (0x200).to_bytes(4, "little")
binary[section + 20:section + 24] = (0x200).to_bytes(4, "little")
binary[0x200 + 12:0x200 + 16] = (0x1050).to_bytes(4, "little")
binary[0x250:0x250 + len(b"kernel32.dll\0")] = b"kernel32.dll\0"
target = {"binary_format": "pe32+-x86_64", "binary_architecture": "x86_64"}
module.verify_header(bytes(binary), target)
linkage = module.inspect_windows(bytes(binary))
if linkage["dynamic_imports"] != ["kernel32.dll"] or linkage["linkage"] != "dynamic":
    raise SystemExit("synthetic bounded PE import inspection did not match")
reviewed_binary = bytes(binary)
def expect_pe_rejection(candidate, label):
    try:
        module.inspect_windows(bytes(candidate))
    except module.InspectionError:
        return
    raise SystemExit(f"PE import inspection accepted {label}")
non_system = bytearray(reviewed_binary)
non_system[0x250:0x250 + len(b"api-ms-win-evil.dll\0")] = b"api-ms-win-evil.dll\0"
expect_pe_rejection(non_system, "a non-reviewed API-set DLL")
truncated_descriptor = bytearray(reviewed_binary)
truncated_descriptor[section + 16:section + 20] = (4).to_bytes(4, "little")
expect_pe_rejection(truncated_descriptor, "a cross-section import descriptor")
unterminated = bytearray(reviewed_binary)
unterminated[optional + 124:optional + 128] = (20).to_bytes(4, "little")
expect_pe_rejection(unterminated, "an unterminated import table")
crossing_name = bytearray(reviewed_binary)
crossing_name[0x200 + 12:0x200 + 16] = (0x11FC).to_bytes(4, "little")
crossing_name[0x3FC:0x400] = b"abcd"
expect_pe_rejection(crossing_name, "a cross-section import name")
overlapping = bytearray(reviewed_binary)
overlapping[pe + 6:pe + 8] = (2).to_bytes(2, "little")
second = section + 40
overlapping[second + 8:second + 12] = (0x200).to_bytes(4, "little")
overlapping[second + 12:second + 16] = (0x1100).to_bytes(4, "little")
expect_pe_rejection(overlapping, "overlapping PE sections")
process_spec = importlib.util.spec_from_file_location("career_windows_process", process_path)
if process_spec is None or process_spec.loader is None:
    raise SystemExit("could not load Windows bounded process helper")
process_module = importlib.util.module_from_spec(process_spec)
process_spec.loader.exec_module(process_module)
try:
    process_module.run_bounded(
        [sys.executable, "-c", "import sys; sys.stdout.write('x' * 4096)"],
        "synthetic over-bound child",
        maximum_output_bytes=1024,
    )
except process_module.BoundedProcessError:
    pass
else:
    raise SystemExit("Windows process helper accepted over-bound child output")
PY

for package in \
  @revazi/career-darwin-arm64 \
  @revazi/career-darwin-x64 \
  @revazi/career-linux-x64-gnu \
  @revazi/career-linux-arm64-gnu \
  @revazi/career-linux-x64-musl \
  @revazi/career-linux-arm64-musl \
  @revazi/career-win32-x64-msvc \
  @revazi/career-win32-arm64-msvc \
  @revazi/career
do
  grep -Fq "$package" "$repository_root/docs/releasing.md"
done
grep -Fq '"$trust_npm" trust github "$package"' "$repository_root/docs/releasing.md"
grep -Fq '"$trust_npm" trust list "$package"' "$repository_root/docs/releasing.md"
grep -Fq -- '--file npm-publish-v0.1.1.yml' "$repository_root/docs/releasing.md"
grep -Fq -- '--environment npm-production' "$repository_root/docs/releasing.md"
grep -Fq -- '--allow-publish' "$repository_root/docs/releasing.md"
grep -Fq 'exact npm CLI 11.15.0' "$repository_root/docs/releasing.md"
grep -Fq 'publication and public acceptance remain pinned to npm 11.6.2' "$repository_root/docs/releasing.md"
grep -Fq 'sha512-+k0tk7lRnpMUPnC7kTuU/yrV/mnFoPhJQ75VfLtZ6fwbzOVXaPsTE/Il9Pn1DHi482byMyqkHv/XsQ76mNjXLw==' "$repository_root/docs/releasing.md"
! grep -Fq '11.15.0 or newer' "$repository_root/docs/releasing.md"
grep -Fq 'No-token OIDC steady-state/public acceptance run `31287506624`' "$repository_root/.agents/current-phase.md"
! grep -Eq '@revazi/career-(darwin|linux|win32)' "$repository_root/npm/career/README.md"

printf 'npm publication source, candidate, adversarial, parity, npx-equivalent, fake-registry, and workflow dry-run tests passed.\n'
