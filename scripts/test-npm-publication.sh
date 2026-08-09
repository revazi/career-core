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
  mkdir -p "$root/npm/career" "$root/npm/platforms/darwin-arm64" "$root/npm/platforms/linux-x64-gnu"
  cp "$repository_root/Cargo.toml" "$root/Cargo.toml"
  cp "$repository_root/npm/career/package.json" "$root/npm/career/package.json"
  cp "$repository_root/npm/platforms/darwin-arm64/package.json" "$root/npm/platforms/darwin-arm64/package.json"
  cp "$repository_root/npm/platforms/linux-x64-gnu/package.json" "$root/npm/platforms/linux-x64-gnu/package.json"
  git -C "$root" init -q
  git -C "$root" config user.name "Publication Fixture"
  git -C "$root" config user.email "publication-fixture@example.invalid"
  git -C "$root" add .
  git -C "$root" commit -q -m "fixture source"
  git -C "$root" remote add origin https://github.com/revazi/career-core.git
  git -C "$root" update-ref refs/remotes/origin/main "$(git -C "$root" rev-parse HEAD)"
  git -C "$root" tag -a v0.1.0 -m "fixture v0.1.0"
}

verify_fixture() {
  local root="$1"
  "$source_gate" \
    --repository-root "$root" \
    --expected-ref refs/tags/v0.1.0 \
    --reviewed-sha "$(git -C "$root" rev-parse HEAD)"
}

expect_source_failure() {
  local label="$1"
  local expected="$2"
  local root="$3"
  shift 3
  if "$source_gate" \
    --repository-root "$root" \
    --expected-ref refs/tags/v0.1.0 \
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
git -C "$fixture" tag -d v0.1.0 >/dev/null
expect_source_failure missing-tag "exact v0.1.0 tag is missing" "$fixture"

fixture="$temporary_root/source-lightweight-tag"
make_minimal_fixture "$fixture"
git -C "$fixture" tag -d v0.1.0 >/dev/null
git -C "$fixture" tag v0.1.0
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
git -C "$fixture" tag -f -a v0.1.0 -m "non-main v0.1.0" >/dev/null
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
value["version"] = "0.1.1"
path.write_text(json.dumps(value, indent=2) + "\n")
PY
git -C "$fixture" add .
git -C "$fixture" commit -q -m "wrong version"
git -C "$fixture" update-ref refs/remotes/origin/main "$(git -C "$fixture" rev-parse HEAD)"
git -C "$fixture" tag -f -a v0.1.0 -m "wrong version tag" >/dev/null
expect_source_failure version "launcher name/version is not the approved initial release" "$fixture"

if "$source_gate" \
  --repository-root "$baseline" \
  --expected-ref refs/tags/v0.1.1 \
  --reviewed-sha "$(git -C "$baseline" rev-parse HEAD)" \
  >"$temporary_root/wrong-ref.stdout" 2>"$temporary_root/wrong-ref.stderr"; then
  fail "source gate accepted the wrong tag/ref"
fi
grep -Fq 'expected ref must be exactly refs/tags/v0.1.0' "$temporary_root/wrong-ref.stderr"

if "$source_gate" \
  --repository-root "$baseline" \
  --expected-ref refs/tags/v0.1.0 \
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
git -C "$fixture_repository" tag -a v0.1.0 -m "Career Core v0.1.0 fixture"

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
    runner_image="macos-14-fixture"
    current_file="10-revazi-career-darwin-arm64-0.1.0.tgz"
    opposite_key="linux-x64-gnu"
    opposite_file="20-revazi-career-linux-x64-gnu-0.1.0.tgz"
    ;;
  Linux:x86_64|Linux:amd64)
    current_key="linux-x64-gnu"
    current_target="x86_64-unknown-linux-gnu"
    runner_os="Linux"
    runner_arch="X64"
    runner_image="ubuntu-fixture"
    current_file="20-revazi-career-linux-x64-gnu-0.1.0.tgz"
    opposite_key="darwin-arm64"
    opposite_file="10-revazi-career-darwin-arm64-0.1.0.tgz"
    ;;
  *) fail "candidate test requires an approved native host" ;;
esac

native_output="$temporary_root/current-native"
CARGO_TARGET_DIR="$repository_root/target" \
  "$fixture_repository/scripts/prepare-npm-publication-native.sh" \
  --output-dir "$native_output" \
  --expected-target "$current_target" \
  --expected-ref refs/tags/v0.1.0 \
  --reviewed-sha "$fixture_sha" \
  --runner-os "$runner_os" \
  --runner-arch "$runner_arch" \
  --runner-image "$runner_image"
current_tarball="$native_output/tarballs/$current_file"
[[ -f "$current_tarball" ]]

# The opposite-host tarball is a format-correct non-executed fixture. Real
# publication builds and executes that package on its approved native runner.
opposite_stage="$temporary_root/opposite-stage"
mkdir -p "$opposite_stage"
python3 - \
  "$fixture_repository" "$opposite_stage" "$opposite_key" "$fixture_sha" <<'PY'
import hashlib
import json
import os
import pathlib
import sys
root = pathlib.Path(sys.argv[1])
stage = pathlib.Path(sys.argv[2])
key = sys.argv[3]
sha = sys.argv[4]
targets = {
    "darwin-arm64": {
        "name": "@revazi/career-darwin-arm64", "platform": "darwin", "arch": "arm64",
        "rust": "aarch64-apple-darwin", "format": "mach-o-64-aarch64",
        "runner_os": "macOS", "runner_arch": "ARM64", "image": "macos-14-opposite-fixture",
        "libc": None, "header": bytes.fromhex("cffaedfe0c000001") + bytes(56),
    },
    "linux-x64-gnu": {
        "name": "@revazi/career-linux-x64-gnu", "platform": "linux", "arch": "x64",
        "rust": "x86_64-unknown-linux-gnu", "format": "elf-64-x86_64",
        "runner_os": "Linux", "runner_arch": "X64", "image": "ubuntu-opposite-fixture",
        "libc": "glibc 2.35", "header": bytes([0x7f, 0x45, 0x4c, 0x46, 2, 1]) + bytes(12) + bytes([0x3e, 0]) + bytes(44),
    },
}
target = targets[key]
source_manifest = json.loads((root / "npm/platforms" / key / "package.json").read_text())
del source_manifest["private"]
source_manifest["publishConfig"] = {"access": "public", "provenance": True}
(stage / "package.json").write_text(json.dumps(source_manifest, indent=2, sort_keys=True) + "\n")
binary = target["header"]
(stage / "career").write_bytes(binary)
os.chmod(stage / "career", 0o755)
for name in ("LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"):
    (stage / name).write_bytes((root / name).read_bytes())
provenance = {
    "schema_version": "career.npm_native_provenance.v1",
    "package": {
        "name": target["name"], "version": "0.1.0", "platform_key": key,
        "node_platform": target["platform"], "node_arch": target["arch"], "rust_target": target["rust"],
        "minimum_glibc_version": "2.35" if key == "linux-x64-gnu" else None,
    },
    "source": {
        "repository": "https://github.com/revazi/career-core", "git_sha": sha,
        "git_ref": "refs/tags/v0.1.0", "git_tag": "v0.1.0",
        "git_dirty": False, "publication_candidate": True,
    },
    "build": {
        "command": ["cargo", "build", "--release", "--locked", "-p", "career-cli", "--target", target["rust"]],
        "profile": "release", "locked": True, "rustc_version": "rustc 1.97.1 (synthetic fixture)",
        "cargo_version": "cargo 1.97.1 (synthetic fixture)",
        "runner": {"os": target["runner_os"], "arch": target["runner_arch"], "image": target["image"], "libc": target["libc"]},
    },
    "executable": {
        "file_name": "career", "binary_format": target["format"], "mode": "0755",
        "size_bytes": len(binary), "sha256": hashlib.sha256(binary).hexdigest(),
    },
    "integrity": {
        "npm_registry_integrity": "external_to_launcher_runtime",
        "package_contained_sha256": "consistency_only", "independent_signature": "absent",
    },
}
(stage / "provenance.json").write_text(json.dumps(provenance, indent=2, sort_keys=True) + "\n")
PY
opposite_pack="$temporary_root/opposite-pack"
mkdir -p "$opposite_pack"
npm pack --offline --ignore-scripts --json --pack-destination "$opposite_pack" "$opposite_stage" \
  >"$temporary_root/opposite-pack.json"
opposite_packed="$(python3 - "$temporary_root/opposite-pack.json" <<'PY'
import json, pathlib, sys
print(json.loads(pathlib.Path(sys.argv[1]).read_text())[0]["filename"])
PY
)"
mv "$opposite_pack/$opposite_packed" "$opposite_pack/$opposite_file"
opposite_tarball="$opposite_pack/$opposite_file"

if [[ "$current_key" == "darwin-arm64" ]]; then
  darwin_tarball="$current_tarball"
  linux_tarball="$opposite_tarball"
else
  darwin_tarball="$opposite_tarball"
  linux_tarball="$current_tarball"
fi
candidate_dir="$temporary_root/candidate"
CARGO_TARGET_DIR="$repository_root/target" \
  "$fixture_repository/scripts/assemble-npm-publication-candidate.sh" \
  --output-dir "$candidate_dir" \
  --darwin-tarball "$darwin_tarball" \
  --linux-tarball "$linux_tarball" \
  --expected-ref refs/tags/v0.1.0 \
  --reviewed-sha "$fixture_sha"
"$fixture_repository/scripts/verify-npm-publication-candidate.sh" \
  --candidate-dir "$candidate_dir" \
  --reviewed-sha "$fixture_sha" \
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

def rewrite(tarball, member_mutator=None, add_extra=False, unsafe_mode_member=None):
    source = tarball.with_suffix(".source")
    tarball.rename(source)
    with tarfile.open(source, "r:gz") as archive:
        entries = [(copy.copy(member), archive.extractfile(member).read()) for member in archive.getmembers()]
    with tarfile.open(tarball, "w:gz") as archive:
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

if kind == "wrong-sha":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["source"].update(git_sha="f" * 40)))
elif kind == "wrong-target":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["package"].update(rust_target="x86_64-pc-windows-msvc")))
elif kind == "provenance-mismatch":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["source"].update(publication_candidate=False)))
elif kind == "wrong-toolchain":
    rewrite(root / current_file, mutate_json_member("package/provenance.json", None, lambda v: v["build"].update(rustc_version="rustc 1.96.0 (wrong)")))
elif kind == "glibc-floor":
    linux = root / "20-revazi-career-linux-x64-gnu-0.1.0.tgz"
    rewrite(linux, mutate_json_member("package/package.json", None, lambda v: v["career_native"].update(minimum_glibc_version="2.34")))
elif kind == "wrong-version":
    rewrite(root / "30-revazi-career-0.1.0.tgz", mutate_json_member("package/package.json", None, lambda v: v.update(version="0.1.1")))
elif kind == "lifecycle":
    rewrite(root / "30-revazi-career-0.1.0.tgz", mutate_json_member("package/package.json", None, lambda v: v.update(scripts={"postinstall": "node download.js"})))
elif kind == "dependency":
    rewrite(root / current_file, mutate_json_member("package/package.json", None, lambda v: v.update(dependencies={"download": "1.0.0"})))
elif kind == "allowlist":
    rewrite(root / current_file, add_extra=True)
elif kind == "extra-property":
    rewrite(root / "30-revazi-career-0.1.0.tgz", mutate_json_member("package/package.json", None, lambda v: v.update(unreviewed=True)))
elif kind == "unsafe-mode":
    rewrite(root / "30-revazi-career-0.1.0.tgz", unsafe_mode_member="package/LICENSE-MIT")
elif kind == "launcher-bytes":
    rewrite(root / "30-revazi-career-0.1.0.tgz", lambda name, data: b"tampered\n" if name == "package/bin/career.js" else data)
elif kind == "readme-bytes":
    rewrite(root / "30-revazi-career-0.1.0.tgz", lambda name, data: b"tampered\n" if name == "package/README.md" else data)
elif kind == "launcher-order":
    path = root / "publication-manifest.json"
    value = json.loads(path.read_text())
    value["packages"] = [value["packages"][2], value["packages"][0], value["packages"][1]]
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
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
  extra-property unsafe-mode launcher-bytes readme-bytes launcher-order extra-directory extra-fifo
do
  mutated="$temporary_root/mutated-$mutation"
  mutate_candidate "$mutation" "$mutated"
  if "$fixture_repository/scripts/npm-publication-candidate.py" verify \
    "$mutated" --source-sha "$fixture_sha" --repository-root "$fixture_repository" \
    >"$temporary_root/$mutation.stdout" 2>"$temporary_root/$mutation.stderr"; then
    fail "candidate verification accepted mutation: $mutation"
  fi
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
    packages = {manifest["packages"][0]["name"]: {"0.1.0": {"integrity": "sha512-conflict", "attestations": {}}}}
elif scenario == "oidc-ready": packages = {row["name"]: {} for row in manifest["packages"]}
elif scenario == "missing-attestation":
    packages = exact
    packages[manifest["packages"][0]["name"]]["0.1.0"]["attestations"] = None
elif scenario == "malformed-attestation":
    packages = exact
    packages[manifest["packages"][0]["name"]]["0.1.0"]["attestations"] = {
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
assert published == ["@revazi/career-darwin-arm64", "@revazi/career-linux-x64-gnu", "@revazi/career"]
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
assert published == ["@revazi/career-linux-x64-gnu", "@revazi/career"]
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
assert attempts["@revazi/career-linux-x64-gnu"] == 1
assert attempts["@revazi/career"] == 1
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

python3 - "$repository_root/.github/workflows/npm-publish.yml" "$publication_driver" <<'PY'
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
driver_path = pathlib.Path(sys.argv[2])
text = path.read_text(encoding="utf-8")
driver = driver_path.read_text(encoding="utf-8")
required = [
    "workflow_dispatch:", "bootstrap:", "default: false", "npm-production",
    "node-version: '22.19.0'", "npm@11.6.2", "id-token: write",
    "ubuntu-22.04", "macos-14", "1.97.1", "1.85.0",
    "scripts/publish-npm-publication-candidate.sh", "public-acceptance:",
    "npx --yes --package=@revazi/career@0.1.0 career --version", "dist.attestations",
    "actions/upload-artifact@bbbca2ddaa5d8feaa63e36b76fdaad77386f024f",
    "actions/download-artifact@70fc10c6e5e1ce46ad2ea6f2b72d43f7d47b13c3",
    "timeout-minutes: 40", "registry_visibility_attempts=61",
    "require_registry_package_ready",
    "10-revazi-career-darwin-arm64-0.1.0.tgz",
    "20-revazi-career-linux-x64-gnu-0.1.0.tgz",
    "30-revazi-career-0.1.0.tgz",
]
for value in required:
    if value not in text and value not in driver:
        raise SystemExit(f"publication workflow/driver is missing required policy text: {value}")
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
if text.index("Configure exact release Rust for publication fixtures") > text.index("Run registry-free publication fixtures and policy tests"):
    raise SystemExit("publication fixtures run before exact Rust 1.97.1 setup")
for line in text.splitlines():
    stripped = line.strip()
    if stripped.startswith("uses: actions/"):
        ref = stripped.rsplit("@", 1)[-1].split()[0]
        if len(ref) != 40 or any(character not in "0123456789abcdef" for character in ref):
            raise SystemExit(f"GitHub-owned action is not pinned by immutable SHA: {stripped}")
if driver.index("10-revazi-career-darwin-arm64-0.1.0.tgz") > driver.index("30-revazi-career-0.1.0.tgz"):
    raise SystemExit("publication driver does not encode native-before-launcher ordering")
PY

for package in \
  @revazi/career-darwin-arm64 \
  @revazi/career-linux-x64-gnu \
  @revazi/career
do
  grep -Fq "\"\$trust_npm\" trust github $package" "$repository_root/docs/releasing.md"
  grep -Fq "\"\$trust_npm\" trust list $package" "$repository_root/docs/releasing.md"
done
grep -Fq -- '--file npm-publish.yml' "$repository_root/docs/releasing.md"
grep -Fq -- '--environment npm-production' "$repository_root/docs/releasing.md"
grep -Fq -- '--allow-publish' "$repository_root/docs/releasing.md"
grep -Fq 'exact npm CLI 11.15.0' "$repository_root/docs/releasing.md"
grep -Fq 'publication and public acceptance remain pinned to npm 11.6.2' "$repository_root/docs/releasing.md"
grep -Fq 'sha512-+k0tk7lRnpMUPnC7kTuU/yrV/mnFoPhJQ75VfLtZ6fwbzOVXaPsTE/Il9Pn1DHi482byMyqkHv/XsQ76mNjXLw==' "$repository_root/docs/releasing.md"
! grep -Fq '11.15.0 or newer' "$repository_root/docs/releasing.md"
grep -Fq 'No-token OIDC steady-state/public acceptance run `31287506624`' "$repository_root/.agents/current-phase.md"
! grep -Eq '@revazi/career-(darwin|linux)' "$repository_root/npm/career/README.md"

printf 'npm publication source, candidate, adversarial, parity, npx-equivalent, fake-registry, and workflow dry-run tests passed.\n'
