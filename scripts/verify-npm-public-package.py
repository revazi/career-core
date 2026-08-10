#!/usr/bin/env python3
"""Verify one exact public @revazi/career install without registry access."""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import platform
import re
import shutil
import sys

sys.dont_write_bytecode = True

from npm_windows_process import BoundedProcessError, run_bounded  # noqa: E402

MAX_OUTPUT_BYTES = 32 * 1024 * 1024
TARGETS = {
    "aarch64-apple-darwin": (
        "Darwin",
        "arm64",
        "@revazi/career-darwin-arm64",
        "career",
    ),
    "x86_64-apple-darwin": ("Darwin", "x86_64", "@revazi/career-darwin-x64", "career"),
    "x86_64-unknown-linux-gnu": (
        "Linux",
        "x86_64",
        "@revazi/career-linux-x64-gnu",
        "career",
    ),
    "aarch64-unknown-linux-gnu": (
        "Linux",
        "arm64",
        "@revazi/career-linux-arm64-gnu",
        "career",
    ),
    "x86_64-unknown-linux-musl": (
        "Linux",
        "x86_64",
        "@revazi/career-linux-x64-musl",
        "career",
    ),
    "aarch64-unknown-linux-musl": (
        "Linux",
        "arm64",
        "@revazi/career-linux-arm64-musl",
        "career",
    ),
    "x86_64-pc-windows-msvc": (
        "Windows",
        "x86_64",
        "@revazi/career-win32-x64-msvc",
        "career.exe",
    ),
    "aarch64-pc-windows-msvc": (
        "Windows",
        "arm64",
        "@revazi/career-win32-arm64-msvc",
        "career.exe",
    ),
}
LICENSE_FILES = {"LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"}


class AcceptanceError(ValueError):
    pass


def fail(message: str) -> None:
    raise AcceptanceError(message)


def command_path(name: str) -> str:
    candidates = (f"{name}.exe", f"{name}.cmd", name) if os.name == "nt" else (name,)
    for candidate in candidates:
        value = shutil.which(candidate)
        if value is not None:
            return value
    fail(f"required acceptance command is unavailable: {name}")


def run(command: list[str], label: str, cwd: pathlib.Path | None = None) -> bytes:
    try:
        stdout, stderr = run_bounded(
            command,
            label,
            maximum_output_bytes=MAX_OUTPUT_BYTES,
            cwd=cwd,
            timeout=180,
        )
    except BoundedProcessError as error:
        raise AcceptanceError(str(error)) from error
    if stderr or not stdout:
        fail(f"{label} produced empty output or unexpected stderr")
    return stdout


def package_entries(root: pathlib.Path) -> set[str]:
    values = set()
    for path in root.rglob("*"):
        if path.is_symlink():
            fail("installed package contains a symbolic link")
        if path.is_file():
            values.add(path.relative_to(root).as_posix())
    return values


def normalized_machine() -> str:
    value = platform.machine().lower()
    return {"amd64": "x86_64", "aarch64": "arm64"}.get(value, value)


def verify_host(target: str, expected_system: str, expected_machine: str) -> None:
    if platform.system() != expected_system or normalized_machine() != expected_machine:
        fail("public acceptance host does not match the exact native target")
    if target.endswith("linux-gnu"):
        family, version = platform.libc_ver()
        if family != "glibc" or not version:
            fail("GNU public acceptance requires positive glibc evidence")
    if target.endswith("linux-musl"):
        loader_machine = "aarch64" if expected_machine == "arm64" else "x86_64"
        if not pathlib.Path(f"/lib/ld-musl-{loader_machine}.so.1").exists():
            fail("musl public acceptance requires its architecture-bound loader")


def compare(
    name: str,
    arguments: list[str],
    native: pathlib.Path,
    node: str,
    launcher: pathlib.Path,
    golden: pathlib.Path | None = None,
) -> bytes:
    native_output = run([str(native), *arguments], f"native {name}")
    launcher_output = run([node, str(launcher), *arguments], f"launcher {name}")
    if native_output != launcher_output:
        fail(f"public launcher and native outputs differ: {name}")
    if golden is not None and launcher_output != golden.read_bytes():
        fail(f"public package output differs from reviewed golden: {name}")
    return launcher_output


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--consumer-dir", required=True, type=pathlib.Path)
    parser.add_argument("--repository-root", required=True, type=pathlib.Path)
    parser.add_argument("--expected-target", required=True, choices=sorted(TARGETS))
    parser.add_argument("--expected-version", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if re.fullmatch(
            r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)",
            args.expected_version,
        ) is None:
            fail("expected version is not an exact stable SemVer")
        release_version = args.expected_version
        expected_system, expected_machine, package_name, executable = TARGETS[
            args.expected_target
        ]
        verify_host(args.expected_target, expected_system, expected_machine)
        consumer = args.consumer_dir.resolve(strict=True)
        repository_root = args.repository_root.resolve(strict=True)
        modules = consumer / "node_modules"
        scope = modules / "@revazi"
        launcher_root = scope / "career"
        native_root = modules.joinpath(*package_name.split("/"))
        expected_scope = {"career", package_name.split("/", 1)[1]}
        actual_scope = {path.name for path in scope.iterdir() if path.is_dir()}
        if actual_scope != expected_scope:
            fail("public install selected an unexpected native package set")
        expected_launcher = {
            "package.json",
            "bin/career.js",
            "targets.json",
            "README.md",
            *LICENSE_FILES,
        }
        expected_native = {
            "package.json",
            executable,
            "provenance.json",
            *LICENSE_FILES,
        }
        if package_entries(launcher_root) != expected_launcher:
            fail("public launcher file allowlist mismatch")
        if package_entries(native_root) != expected_native:
            fail("public native package file allowlist mismatch")
        launcher_manifest = json.loads((launcher_root / "package.json").read_bytes())
        native_manifest = json.loads((native_root / "package.json").read_bytes())
        provenance = json.loads((native_root / "provenance.json").read_bytes())
        if (
            launcher_manifest.get("name") != "@revazi/career"
            or launcher_manifest.get("version") != release_version
        ):
            fail("public launcher identity/version mismatch")
        if (
            native_manifest.get("name") != package_name
            or native_manifest.get("version") != release_version
        ):
            fail("public native identity/version mismatch")
        if (
            provenance.get("source", {}).get("git_ref")
            != f"refs/tags/v{release_version}"
            or provenance.get("source", {}).get("publication_candidate") is not True
        ):
            fail("public native provenance is not bound to the exact release tag")
        node = command_path("node")
        launcher = launcher_root / "bin/career.js"
        native = native_root / executable
        version = compare("version", ["--version"], native, node, launcher)
        if version.strip() != f"career {release_version}".encode("ascii"):
            fail("public CLI version does not match the exact release")
        compare(
            "capabilities",
            ["capabilities"],
            native,
            node,
            launcher,
            repository_root
            / "fixtures/managed-adapter/phase8/capabilities.pre-phase8.expected.json",
        )
        compare(
            "operations",
            ["operations"],
            native,
            node,
            launcher,
            repository_root
            / "fixtures/managed-adapter/phase8/operation-catalog.expected.json",
        )
        for name, arguments in (
            ("schema-list", ["schema", "list", "--format", "json-compact"]),
            (
                "schema-bundle",
                [
                    "schema",
                    "bundle",
                    "--id",
                    "career.job_match_input.v1",
                    "--format",
                    "json-compact",
                ],
            ),
        ):
            json.loads(compare(name, arguments, native, node, launcher))
        compare(
            "resume-analysis",
            [
                "resume",
                "analyze",
                "--input",
                str(
                    repository_root
                    / "fixtures/resume/phase3/complete-analysis.input.json"
                ),
            ],
            native,
            node,
            launcher,
            repository_root / "fixtures/resume/phase3/complete-analysis.expected.json",
        )
        compare(
            "job-match",
            [
                "job",
                "match",
                "--input",
                str(repository_root / "fixtures/job/phase4b/complete-match.input.json"),
            ],
            native,
            node,
            launcher,
            repository_root / "fixtures/job/phase4b/complete-match.expected.json",
        )
        if os.name == "nt":
            comspec = os.environ.get("COMSPEC")
            if comspec is None:
                fail("Windows command processor is unavailable")
            shim = run(
                [comspec, "/d", "/c", r"node_modules\.bin\career.cmd --version"],
                "public Windows npm shim",
                cwd=consumer,
            )
        else:
            shim = run(
                [str(modules / ".bin/career"), "--version"],
                "public npm bin shim",
                cwd=consumer,
            )
        if shim.strip() != f"career {release_version}".encode("ascii"):
            fail("public npm bin shim version mismatch")
        print(f"Exact public npm acceptance passed for {args.expected_target}.")
        return 0
    except (AcceptanceError, OSError, json.JSONDecodeError) as error:
        print(f"public npm acceptance failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
