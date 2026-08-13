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
import signal
import subprocess
import sys

sys.dont_write_bytecode = True

import tempfile
import time

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
}
LICENSE_FILES = {"LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"}


class AcceptanceError(ValueError):
    pass


class BoundedProcessError(ValueError):
    pass


def process_group_exists(process_group: int) -> bool:
    try:
        os.killpg(process_group, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def wait_for_process_group(
    process: subprocess.Popen[bytes], settlement_timeout: float
) -> bool:
    deadline = time.monotonic() + settlement_timeout
    while True:
        process_exited = process.poll() is not None
        if process_exited and not process_group_exists(process.pid):
            return True
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return False
        time.sleep(min(0.01, remaining))


def signal_process_group(process_group: int, requested_signal: signal.Signals) -> None:
    try:
        os.killpg(process_group, requested_signal)
    except ProcessLookupError:
        pass
    except PermissionError as error:
        raise BoundedProcessError("subprocess group could not be signaled") from error


def stop_process_group(
    process: subprocess.Popen[bytes], settlement_timeout: float = 10
) -> None:
    signal_process_group(process.pid, signal.SIGTERM)
    if wait_for_process_group(process, settlement_timeout):
        return
    signal_process_group(process.pid, signal.SIGKILL)
    if not wait_for_process_group(process, settlement_timeout):
        raise BoundedProcessError(
            "subprocess group did not settle within its reviewed bound"
        )


def run_bounded(
    command: list[str],
    label: str,
    *,
    maximum_output_bytes: int,
    cwd: pathlib.Path | None = None,
    timeout: float = 180,
    settlement_timeout: float = 10,
) -> tuple[bytes, bytes]:
    process: subprocess.Popen[bytes] | None = None
    try:
        with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                command,
                cwd=cwd,
                stdout=stdout,
                stderr=stderr,
                start_new_session=True,
            )
            deadline = time.monotonic() + timeout
            while process.poll() is None:
                output_size = (
                    os.fstat(stdout.fileno()).st_size
                    + os.fstat(stderr.fileno()).st_size
                )
                if output_size > maximum_output_bytes:
                    stop_process_group(process, settlement_timeout)
                    raise BoundedProcessError(
                        f"{label} output exceeds its reviewed bound"
                    )
                if time.monotonic() >= deadline:
                    stop_process_group(process, settlement_timeout)
                    raise BoundedProcessError(f"{label} timed out")
                time.sleep(0.01)
            output_size = (
                os.fstat(stdout.fileno()).st_size
                + os.fstat(stderr.fileno()).st_size
            )
            if output_size > maximum_output_bytes:
                raise BoundedProcessError(f"{label} output exceeds its reviewed bound")
            stdout.seek(0)
            stderr.seek(0)
            stdout_bytes = stdout.read(maximum_output_bytes + 1)
            stderr_bytes = stderr.read(maximum_output_bytes + 1)
    except BoundedProcessError:
        raise
    except (OSError, subprocess.SubprocessError) as error:
        if process is not None:
            try:
                stop_process_group(process, settlement_timeout)
            except BoundedProcessError as settlement_error:
                raise settlement_error from error
        raise BoundedProcessError(f"{label} could not run") from error
    if process.returncode != 0:
        diagnostic = stderr_bytes[:512].decode("utf-8", errors="replace").strip()
        raise BoundedProcessError(
            f"{label} returned an unexpected exit code: {diagnostic}"
        )
    return stdout_bytes, stderr_bytes


def fail(message: str) -> None:
    raise AcceptanceError(message)


def command_path(name: str) -> str:
    value = shutil.which(name)
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
