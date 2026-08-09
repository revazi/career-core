#!/usr/bin/env python3
"""Build and stage one exact native Windows npm CLI package without publication."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
from typing import Any

CATALOG_SHA256 = "9e56a3ca9b68799b0ff4bd52bbd2e71c2839d05a70398c5942062cb6e68032e2"
MAX_BINARY_BYTES = 16 * 1024 * 1024
MAX_JSON_BYTES = 64 * 1024
MAX_OUTPUT_BYTES = 32 * 1024 * 1024
MAX_TARBALL_BYTES = 32 * 1024 * 1024
VERSION = "0.1.1"
TARGETS = {
    "x86_64-pc-windows-msvc": {
        "machine": "x86_64",
        "node_arch": "x64",
        "platform_key": "win32-x64-msvc",
        "package": "@revazi/career-win32-x64-msvc",
        "runner_arch": "X64",
    },
    "aarch64-pc-windows-msvc": {
        "machine": "arm64",
        "node_arch": "arm64",
        "platform_key": "win32-arm64-msvc",
        "package": "@revazi/career-win32-arm64-msvc",
        "runner_arch": "ARM64",
    },
}
LICENSE_FILES = ("LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md")


class PreparationError(ValueError):
    pass


def fail(message: str) -> None:
    raise PreparationError(message)


def command_path(name: str) -> str:
    candidates = [name]
    if os.name == "nt":
        candidates = [f"{name}.exe", f"{name}.cmd", name]
    for candidate in candidates:
        value = shutil.which(candidate)
        if value is not None:
            return value
    fail(f"required command is unavailable: {name}")


def run(
    command: list[str],
    label: str,
    *,
    cwd: pathlib.Path | None = None,
    env: dict[str, str] | None = None,
    timeout: int = 900,
) -> tuple[bytes, bytes]:
    try:
        result = subprocess.run(
            command,
            cwd=cwd,
            env=env,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise PreparationError(f"{label} could not run") from error
    if len(result.stdout) + len(result.stderr) > MAX_OUTPUT_BYTES:
        fail(f"{label} output exceeds its reviewed bound")
    if result.returncode != 0:
        fail(f"{label} failed")
    return result.stdout, result.stderr


def text(command: list[str], label: str, *, cwd: pathlib.Path | None = None) -> str:
    stdout, _ = run(command, label, cwd=cwd)
    try:
        return stdout.decode("utf-8").strip()
    except UnicodeError as error:
        raise PreparationError(f"{label} output is not UTF-8") from error


def bounded_json(path: pathlib.Path, maximum: int = MAX_JSON_BYTES) -> dict[str, Any]:
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or not 1 <= before.st_size <= maximum:
        fail(f"JSON input is not one bounded regular file: {path.name}")
    data = path.read_bytes()
    after = path.lstat()
    if len(data) != before.st_size or (before.st_dev, before.st_ino) != (
        after.st_dev,
        after.st_ino,
    ):
        fail(f"JSON input changed during inspection: {path.name}")
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise PreparationError(f"JSON input is malformed: {path.name}") from error
    if not isinstance(value, dict):
        fail(f"JSON input is not an object: {path.name}")
    return value


def normalized_machine() -> str:
    machine = platform.machine().lower()
    aliases = {"amd64": "x86_64", "arm64": "arm64", "aarch64": "arm64"}
    return aliases.get(machine, machine)


def selected_host(expected_target: str) -> dict[str, str]:
    if platform.system() != "Windows":
        fail("Windows package preparation requires exact native Windows")
    target = TARGETS.get(expected_target)
    if target is None or normalized_machine() != target["machine"]:
        fail("native Windows host architecture does not match the expected target")
    return target


def outside_checkout(
    repository_root: pathlib.Path, output: pathlib.Path
) -> pathlib.Path:
    resolved = output.expanduser().resolve(strict=False)
    try:
        common = pathlib.Path(os.path.commonpath([repository_root, resolved]))
    except ValueError:
        common = None
    if common is not None and os.path.normcase(str(common)) == os.path.normcase(
        str(repository_root)
    ):
        fail("output directory must be outside the source checkout")
    if resolved.exists() and any(resolved.iterdir()):
        fail("output directory must be empty")
    resolved.mkdir(parents=True, exist_ok=True)
    return resolved


def load_catalog(repository_root: pathlib.Path) -> tuple[dict[str, Any], bytes]:
    path = repository_root / "npm/career/targets.json"
    data = path.read_bytes()
    if (
        not 1 <= len(data) <= MAX_JSON_BYTES
        or hashlib.sha256(data).hexdigest() != CATALOG_SHA256
    ):
        fail("target catalog bytes are not exact reviewed bytes")
    try:
        catalog = json.loads(data)
    except json.JSONDecodeError as error:
        raise PreparationError("target catalog is malformed") from error
    if (
        not isinstance(catalog, dict)
        or set(catalog) != {"schema_version", "targets"}
        or catalog.get("schema_version") != "career.npm_target_catalog.v1"
        or not isinstance(catalog.get("targets"), list)
        or len(catalog["targets"]) != 8
    ):
        fail("target catalog shape is invalid")
    return catalog, data


def validate_templates(
    repository_root: pathlib.Path,
    catalog: dict[str, Any],
    selected: dict[str, Any],
) -> None:
    launcher = bounded_json(repository_root / "npm/career/package.json")
    names = [value.get("native_package") for value in catalog["targets"]]
    expected_optional = {name: VERSION for name in names}
    if (
        launcher.get("name") != "@revazi/career"
        or launcher.get("version") != VERSION
        or launcher.get("private") is not True
        or launcher.get("optionalDependencies") != expected_optional
        or launcher.get("career_launcher", {}).get("platform_packages") != names
        or "dependencies" in launcher
        or "scripts" in launcher
    ):
        fail("launcher template identity, order, lockstep, or private guard is invalid")
    for value in catalog["targets"]:
        manifest = bounded_json(
            repository_root / "npm/platforms" / value["platform_key"] / "package.json"
        )
        if (
            manifest.get("name") != value["native_package"]
            or manifest.get("version") != VERSION
            or manifest.get("private") is not True
            or any(
                key in manifest
                for key in ("dependencies", "optionalDependencies", "scripts")
            )
        ):
            fail(
                "native template identity, lockstep, dependency, or private guard is invalid"
            )
    selected_manifest = bounded_json(
        repository_root / "npm/platforms" / selected["platform_key"] / "package.json"
    )
    native = selected_manifest.get("career_native")
    expected_native = {
        "schema_version": "career.npm_native_package.v2",
        "platform_key": selected["platform_key"],
        "node_platform": selected["node_platform"],
        "node_arch": selected["node_arch"],
        "libc_family": None,
        "rust_target": selected["rust_target"],
        "binary_file": "career.exe",
        "provenance_file": "provenance.json",
        "binary_format": selected["binary_format"],
        "binary_architecture": selected["binary_architecture"],
        "file_invariant": "windows_regular_non_symlink_exe",
        "archive_mode": "0644",
        "executable_mode": None,
        "maximum_binary_size_bytes": MAX_BINARY_BYTES,
        "minimum_glibc_version": None,
    }
    if native != expected_native:
        fail("selected Windows native template metadata is invalid")


def run_native(binary: pathlib.Path, arguments: list[str], label: str) -> bytes:
    stdout, stderr = run([str(binary), *arguments], label, timeout=120)
    if stderr:
        fail(f"{label} wrote unexpected stderr")
    if not stdout or len(stdout) > MAX_OUTPUT_BYTES:
        fail(f"{label} output is empty or exceeds its reviewed bound")
    return stdout


def verify_native_operations(
    repository_root: pathlib.Path, binary: pathlib.Path
) -> None:
    checks = [
        ("capabilities", ["capabilities", "--format", "json-compact"], None),
        ("operations", ["operations", "--format", "json-compact"], None),
        (
            "schema bundle",
            [
                "schema",
                "bundle",
                "--id",
                "career.job_match_input.v1",
                "--format",
                "json-compact",
            ],
            None,
        ),
        (
            "resume analysis",
            [
                "resume",
                "analyze",
                "--input",
                str(
                    repository_root
                    / "fixtures/resume/phase3/complete-analysis.input.json"
                ),
            ],
            repository_root / "fixtures/resume/phase3/complete-analysis.expected.json",
        ),
        (
            "job match",
            [
                "job",
                "match",
                "--input",
                str(repository_root / "fixtures/job/phase4b/complete-match.input.json"),
            ],
            repository_root / "fixtures/job/phase4b/complete-match.expected.json",
        ),
    ]
    for label, arguments, golden in checks:
        output = run_native(binary, arguments, label)
        try:
            json.loads(output)
        except json.JSONDecodeError as error:
            raise PreparationError(f"{label} output is not JSON") from error
        if golden is not None and output != golden.read_bytes():
            fail(f"{label} output differs from its reviewed golden")


def copy_regular(source: pathlib.Path, destination: pathlib.Path) -> None:
    if source.is_symlink() or not source.is_file():
        fail(f"staged source is not one regular non-symlink file: {source.name}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)


def write_provenance(
    stage: pathlib.Path,
    target: dict[str, Any],
    git_sha: str,
    git_dirty: bool,
    rustc_version: str,
    cargo_version: str,
) -> None:
    binary = stage / "career.exe"
    binary_bytes = binary.read_bytes()
    if not 1 <= len(binary_bytes) <= MAX_BINARY_BYTES:
        fail("native executable size is outside the catalog package bound")
    provenance = {
        "schema_version": "career.npm_native_provenance.v2",
        "package": {
            "name": target["native_package"],
            "version": VERSION,
            "platform_key": target["platform_key"],
            "node_platform": "win32",
            "node_arch": target["node_arch"],
            "libc_family": None,
            "rust_target": target["rust_target"],
            "minimum_glibc_version": None,
        },
        "source": {
            "repository": "https://github.com/revazi/career-core",
            "git_sha": git_sha,
            "git_ref": None,
            "git_tag": None,
            "git_dirty": git_dirty,
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
                target["rust_target"],
            ],
            "profile": "release",
            "locked": True,
            "rustc_version": rustc_version,
            "cargo_version": cargo_version,
            "runner": {
                "os": "Windows",
                "arch": target["runner_arch"],
                "image": "local",
                "libc": None,
            },
        },
        "executable": {
            "file_name": "career.exe",
            "binary_format": target["binary_format"],
            "binary_architecture": target["binary_architecture"],
            "file_invariant": "windows_regular_non_symlink_exe",
            "archive_mode": "0644",
            "mode": None,
            "size_bytes": len(binary_bytes),
            "sha256": hashlib.sha256(binary_bytes).hexdigest(),
        },
        "integrity": {
            "npm_registry_integrity": "external_to_launcher_runtime",
            "package_contained_sha256": "consistency_only",
            "independent_signature": "absent",
        },
    }
    encoded = (json.dumps(provenance, indent=2, sort_keys=True) + "\n").encode("utf-8")
    if len(encoded) > MAX_JSON_BYTES:
        fail("native provenance exceeds its reviewed bound")
    (stage / "provenance.json").write_bytes(encoded)


def stage_packages(
    repository_root: pathlib.Path,
    output: pathlib.Path,
    target: dict[str, Any],
    binary: pathlib.Path,
    git_sha: str,
    git_dirty: bool,
    rustc_version: str,
    cargo_version: str,
) -> tuple[pathlib.Path, pathlib.Path, pathlib.Path]:
    launcher = output / "stage/launcher"
    platform_stage = output / "stage" / target["platform_key"]
    tarballs = output / "tarballs"
    launcher.mkdir(parents=True)
    platform_stage.mkdir(parents=True)
    tarballs.mkdir(parents=True)
    for relative in ("package.json", "targets.json", "README.md", "bin/career.js"):
        copy_regular(repository_root / "npm/career" / relative, launcher / relative)
    for name in LICENSE_FILES:
        copy_regular(repository_root / name, launcher / name)
        copy_regular(repository_root / name, platform_stage / name)
    copy_regular(
        repository_root / "npm/platforms" / target["platform_key"] / "package.json",
        platform_stage / "package.json",
    )
    copy_regular(binary, platform_stage / "career.exe")
    write_provenance(
        platform_stage, target, git_sha, git_dirty, rustc_version, cargo_version
    )
    expected_launcher = {
        "package.json",
        "targets.json",
        "README.md",
        "bin/career.js",
        *LICENSE_FILES,
    }
    expected_platform = {
        "package.json",
        "career.exe",
        "provenance.json",
        *LICENSE_FILES,
    }
    for root, expected in (
        (launcher, expected_launcher),
        (platform_stage, expected_platform),
    ):
        actual = set()
        for path in root.rglob("*"):
            if path.is_symlink():
                fail("staged package contains a symlink")
            if path.is_file():
                actual.add(path.relative_to(root).as_posix())
        if actual != expected:
            fail("staged package file allowlist mismatch")
    return launcher, platform_stage, tarballs


def npm_environment(cache: pathlib.Path) -> dict[str, str]:
    value = dict(os.environ)
    value.update(
        {
            "npm_config_audit": "false",
            "npm_config_fund": "false",
            "npm_config_ignore_scripts": "true",
            "npm_config_offline": "true",
            "npm_config_update_notifier": "false",
            "npm_config_cache": str(cache),
        }
    )
    return value


def pack_and_verify(
    launcher: pathlib.Path,
    platform_stage: pathlib.Path,
    tarball_dir: pathlib.Path,
    target: dict[str, Any],
    npm: str,
    cache: pathlib.Path,
) -> None:
    env = npm_environment(cache)
    for label, stage in (
        ("native npm pack", platform_stage),
        ("launcher npm pack", launcher),
    ):
        stdout, _ = run(
            [
                npm,
                "pack",
                "--ignore-scripts",
                "--json",
                "--pack-destination",
                str(tarball_dir),
                str(stage),
            ],
            label,
            env=env,
        )
        try:
            value = json.loads(stdout)
        except json.JSONDecodeError as error:
            raise PreparationError(f"{label} output is malformed") from error
        if not isinstance(value, list) or len(value) != 1:
            fail(f"{label} did not return exactly one tarball")
    tarballs = sorted(tarball_dir.glob("*.tgz"))
    if len(tarballs) != 2:
        fail("private package preparation did not produce exactly two tarballs")
    expected = {
        "@revazi/career": {
            "package/package.json",
            "package/bin/career.js",
            "package/targets.json",
            "package/README.md",
            *(f"package/{name}" for name in LICENSE_FILES),
        },
        target["native_package"]: {
            "package/package.json",
            "package/career.exe",
            "package/provenance.json",
            *(f"package/{name}" for name in LICENSE_FILES),
        },
    }
    seen = set()
    for tarball in tarballs:
        if not 1 <= tarball.stat().st_size <= MAX_TARBALL_BYTES:
            fail("npm tarball is outside its reviewed bound")
        with tarfile.open(tarball, "r:gz") as archive:
            members = archive.getmembers()
            if any(not member.isfile() for member in members):
                fail("npm tarball contains a non-regular entry")
            names = {member.name for member in members}
            manifest_file = archive.extractfile("package/package.json")
            if manifest_file is None:
                fail("npm tarball manifest is missing")
            manifest = json.load(manifest_file)
            name = manifest.get("name")
            if name not in expected or name in seen or names != expected[name]:
                fail("npm tarball package identity or file allowlist is invalid")
            if manifest.get("private") is not True or "scripts" in manifest:
                fail("npm tarball private guard or lifecycle policy is invalid")
            if name == target["native_package"]:
                executable = archive.getmember("package/career.exe")
                if stat.S_IMODE(executable.mode) != 0o644:
                    fail("Windows npm executable archive mode is not exactly 0644")
            seen.add(name)
    if seen != set(expected):
        fail("npm tarball package set is incomplete")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=pathlib.Path, required=True)
    parser.add_argument("--expected-target", choices=sorted(TARGETS), required=True)
    parser.add_argument("--allow-dirty", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        host = selected_host(args.expected_target)
        repository_root = pathlib.Path(__file__).resolve().parent.parent
        output = outside_checkout(repository_root, args.output_dir)
        cargo = command_path("cargo")
        git = command_path("git")
        node = command_path("node")
        npm = command_path("npm")
        rustc = command_path("rustc")
        catalog, _ = load_catalog(repository_root)
        selected_values = [
            value
            for value in catalog["targets"]
            if value.get("rust_target") == args.expected_target
        ]
        if len(selected_values) != 1:
            fail("expected Windows target is not unique in the reviewed catalog")
        target = selected_values[0]
        target.update(host)
        validate_templates(repository_root, catalog, target)
        if text([node, "--version"], "Node version inspection") != "v22.19.0":
            fail("Windows package preparation requires exact Node v22.19.0")
        node_platform = text(
            [node, "-p", "process.platform"], "Node platform inspection"
        )
        node_arch = text([node, "-p", "process.arch"], "Node architecture inspection")
        if (node_platform, node_arch) != ("win32", target["node_arch"]):
            fail("Node runtime does not match the exact native Windows target")
        rust_host_output = text([rustc, "-vV"], "rustc host inspection")
        match = re.search(r"^host: (\S+)$", rust_host_output, re.MULTILINE)
        if match is None or match.group(1) != args.expected_target:
            fail("rustc host does not match the exact native Windows target")
        git_sha = text(
            [git, "rev-parse", "HEAD"], "Git SHA inspection", cwd=repository_root
        )
        if re.fullmatch(r"[0-9a-f]{40}", git_sha) is None:
            fail("Git SHA is not one full lowercase commit identifier")
        git_status = text(
            [git, "status", "--porcelain", "--untracked-files=normal"],
            "Git status inspection",
            cwd=repository_root,
        )
        git_dirty = bool(git_status)
        if git_dirty and not args.allow_dirty:
            fail(
                "source worktree is dirty; package preparation requires a reviewed clean commit"
            )
        run(
            [
                cargo,
                "build",
                "--release",
                "--locked",
                "-p",
                "career-cli",
                "--target",
                args.expected_target,
            ],
            "native Windows release build",
            cwd=repository_root,
        )
        target_root = pathlib.Path(
            os.environ.get("CARGO_TARGET_DIR", repository_root / "target")
        )
        if not target_root.is_absolute():
            target_root = repository_root / target_root
        binary = target_root / args.expected_target / "release/career.exe"
        if (
            binary.is_symlink()
            or not binary.is_file()
            or not 1 <= binary.stat().st_size <= MAX_BINARY_BYTES
        ):
            fail(
                "native Windows release executable was not produced as one bounded regular file"
            )
        if (
            text([str(binary), "--version"], "native Windows version inspection")
            != "career 0.1.1"
        ):
            fail("native CLI and npm versions are not exact lockstep 0.1.1")
        verify_native_operations(repository_root, binary)
        rustc_version = text([rustc, "--version"], "rustc version inspection")
        cargo_version = text([cargo, "--version"], "Cargo version inspection")
        launcher, platform_stage, tarballs = stage_packages(
            repository_root,
            output,
            target,
            binary,
            git_sha,
            git_dirty,
            rustc_version,
            cargo_version,
        )
        with tempfile.TemporaryDirectory(prefix="career-windows-npm-") as temporary:
            temporary_path = pathlib.Path(temporary)
            inspection = temporary_path / "inspection.json"
            inspector = repository_root / "scripts/inspect-npm-native-binary.py"
            run(
                [
                    sys.executable,
                    str(inspector),
                    "--repository-root",
                    str(repository_root),
                    "--binary",
                    str(platform_stage / "career.exe"),
                    "--target",
                    args.expected_target,
                    "--source-sha",
                    git_sha,
                    "--runner-image",
                    "local-windows-package-test",
                    "--evidence-kind",
                    "local_policy",
                    "--output",
                    str(inspection),
                ],
                "native Windows PE inspection",
                cwd=repository_root,
            )
            evidence = bounded_json(inspection)
            if evidence.get("target", {}).get("rust_target") != args.expected_target:
                fail("native Windows PE evidence target is invalid")
            pack_and_verify(
                launcher,
                platform_stage,
                tarballs,
                target,
                npm,
                temporary_path / "npm-cache",
            )
        print(
            f"Prepared private npm launcher and native package for {args.expected_target}."
        )
        print(
            f"Source commit: {git_sha} (dirty={str(git_dirty).lower()}, publication_candidate=false)"
        )
        print(f"Output: {output}")
        return 0
    except (PreparationError, OSError) as error:
        print(f"Windows npm CLI package preparation failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
