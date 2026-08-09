#!/usr/bin/env python3
"""Exercise one exact native Windows private npm package and launcher offline."""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import platform
import shutil
import sys
import tarfile
import tempfile

sys.dont_write_bytecode = True

from npm_windows_process import BoundedProcessError, run_bounded  # noqa: E402

MAX_OUTPUT_BYTES = 32 * 1024 * 1024
TARGETS = {
    "x86_64-pc-windows-msvc": (
        "x86_64",
        "win32-x64-msvc",
        "@revazi/career-win32-x64-msvc",
    ),
    "aarch64-pc-windows-msvc": (
        "arm64",
        "win32-arm64-msvc",
        "@revazi/career-win32-arm64-msvc",
    ),
}


class TestError(ValueError):
    pass


def fail(message: str) -> None:
    raise TestError(message)


def command_path(name: str) -> str:
    for candidate in (f"{name}.exe", f"{name}.cmd", name):
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
    expected_code: int = 0,
    timeout: int = 180,
) -> tuple[bytes, bytes]:
    try:
        return run_bounded(
            command,
            label,
            maximum_output_bytes=MAX_OUTPUT_BYTES,
            cwd=cwd,
            env=env,
            expected_code=expected_code,
            timeout=timeout,
        )
    except BoundedProcessError as error:
        raise TestError(str(error)) from error


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


def tarball_name(path: pathlib.Path) -> str:
    with tarfile.open(path, "r:gz") as archive:
        manifest_file = archive.extractfile("package/package.json")
        if manifest_file is None:
            fail("packed manifest is missing")
        value = json.load(manifest_file)
    name = value.get("name")
    if not isinstance(name, str):
        fail("packed package name is invalid")
    return name


def expected_package_files(root: pathlib.Path) -> set[str]:
    values = set()
    for path in root.rglob("*"):
        if path.is_symlink():
            fail("installed npm package contains a symlink")
        if path.is_file():
            values.add(path.relative_to(root).as_posix())
    return values


def compare_command(
    name: str,
    arguments: list[str],
    native: pathlib.Path,
    node: str,
    launcher_js: pathlib.Path,
    golden: pathlib.Path | None,
) -> None:
    native_stdout, native_stderr = run([str(native), *arguments], f"native {name}")
    launcher_stdout, launcher_stderr = run(
        [node, str(launcher_js), *arguments], f"launcher {name}"
    )
    if native_stderr or launcher_stderr:
        fail(f"{name} wrote unexpected stderr")
    if native_stdout != launcher_stdout or not native_stdout:
        fail(f"{name} native and launcher output differ")
    if len(native_stdout) > MAX_OUTPUT_BYTES:
        fail(f"{name} output exceeds the public successful-output bound")
    if golden is not None and native_stdout != golden.read_bytes():
        fail(f"{name} differs from its reviewed golden")
    if name == "version":
        if native_stdout.strip() != b"career 0.1.1":
            fail("packaged Windows version is not exact 0.1.1")
    else:
        try:
            json.loads(native_stdout)
        except json.JSONDecodeError as error:
            raise TestError(f"{name} output is not JSON") from error


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--expected-target", choices=sorted(TARGETS), required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        machine, platform_key, package_name = TARGETS[args.expected_target]
        actual_machine = platform.machine().lower()
        actual_machine = {"amd64": "x86_64", "aarch64": "arm64"}.get(
            actual_machine, actual_machine
        )
        if platform.system() != "Windows" or actual_machine != machine:
            fail("Windows npm package test requires the exact native host architecture")
        repository_root = pathlib.Path(__file__).resolve().parent.parent
        node = command_path("node")
        npm = command_path("npm")
        prepare = repository_root / "scripts/prepare-npm-cli-packages-windows.py"
        with tempfile.TemporaryDirectory(
            prefix="career-windows-package-test-"
        ) as temporary:
            temporary_root = pathlib.Path(temporary)
            inside_output = repository_root / f".npm-windows-inside-{os.getpid()}"
            _, inside_stderr = run(
                [
                    sys.executable,
                    str(prepare),
                    "--output-dir",
                    str(inside_output),
                    "--expected-target",
                    args.expected_target,
                ],
                "in-checkout output rejection",
                expected_code=1,
            )
            if (
                b"output directory must be outside the source checkout"
                not in inside_stderr
            ):
                fail("Windows preparation returned an unexpected in-checkout rejection")
            if inside_output.exists():
                fail("Windows preparation created an in-checkout output")
            wrong_target = next(
                value for value in TARGETS if value != args.expected_target
            )
            _, wrong_stderr = run(
                [
                    sys.executable,
                    str(prepare),
                    "--output-dir",
                    str(temporary_root / "wrong-target"),
                    "--expected-target",
                    wrong_target,
                ],
                "wrong-target rejection",
                expected_code=1,
            )
            if b"host architecture does not match" not in wrong_stderr:
                fail(
                    "Windows preparation returned an unexpected target-mismatch rejection"
                )
            output = temporary_root / "prepared"
            run(
                [
                    sys.executable,
                    str(prepare),
                    "--output-dir",
                    str(output),
                    "--expected-target",
                    args.expected_target,
                ],
                "Windows private package preparation",
                timeout=1200,
            )
            tarballs = sorted((output / "tarballs").glob("*.tgz"))
            if len(tarballs) != 2:
                fail("Windows preparation did not produce exactly two tarballs")
            by_name = {tarball_name(path): path for path in tarballs}
            if set(by_name) != {"@revazi/career", package_name}:
                fail("Windows prepared tarball package set is invalid")
            consumer = temporary_root / "consumer"
            consumer.mkdir()
            manifest = {
                "name": "career-windows-offline-test",
                "version": "0.0.0",
                "private": True,
                "dependencies": {
                    "@revazi/career": by_name["@revazi/career"].resolve().as_uri(),
                    package_name: by_name[package_name].resolve().as_uri(),
                },
            }
            (consumer / "package.json").write_text(
                json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
            run(
                [
                    npm,
                    "install",
                    "--offline",
                    "--ignore-scripts",
                    "--no-audit",
                    "--no-fund",
                    "--no-package-lock",
                ],
                "offline Windows package install",
                cwd=consumer,
                env=npm_environment(temporary_root / "npm-cache"),
                timeout=300,
            )
            modules = consumer / "node_modules"
            launcher_root = modules / "@revazi/career"
            platform_root = modules.joinpath(*package_name.split("/"))
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
                "career.exe",
                "provenance.json",
                "LICENSE-MIT",
                "LICENSE-APACHE",
                "THIRD_PARTY_NOTICES.md",
            }
            if expected_package_files(launcher_root) != expected_launcher:
                fail("installed Windows launcher file allowlist is invalid")
            if expected_package_files(platform_root) != expected_platform:
                fail("installed Windows native package file allowlist is invalid")
            native = platform_root / "career.exe"
            launcher_js = launcher_root / "bin/career.js"
            if native.is_symlink() or not native.is_file() or launcher_js.is_symlink():
                fail("installed Windows executable or launcher is not one regular file")
            commands: list[tuple[str, list[str], pathlib.Path | None]] = [
                ("version", ["--version"], None),
                (
                    "capabilities",
                    ["capabilities"],
                    repository_root
                    / "fixtures/managed-adapter/phase8/capabilities.pre-phase8.expected.json",
                ),
                (
                    "operations",
                    ["operations"],
                    repository_root
                    / "fixtures/managed-adapter/phase8/operation-catalog.expected.json",
                ),
                ("schema-list", ["schema", "list"], None),
                (
                    "schema-export",
                    [
                        "schema",
                        "export",
                        "--id",
                        "career.job_match.v1",
                        "--format",
                        "json-compact",
                    ],
                    None,
                ),
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
                    None,
                ),
                (
                    "resume-evaluate",
                    [
                        "resume",
                        "evaluate",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase1/complete-sections.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase1/complete-sections.expected.json",
                ),
                (
                    "resume-analyze",
                    [
                        "resume",
                        "analyze",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase3/complete-analysis.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase3/complete-analysis.expected.json",
                ),
                (
                    "analysis-suggestions",
                    [
                        "resume",
                        "analysis-suggestions-review",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase7/complete-analysis-suggestion-review.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase7/complete-analysis-suggestion-review.expected.json",
                ),
                (
                    "analysis-replacements",
                    [
                        "resume",
                        "analysis-replacements-review",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase7/complete-analysis-replacement-review.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase7/complete-analysis-replacement-review.expected.json",
                ),
                (
                    "resume-normalize",
                    [
                        "resume",
                        "normalize",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase2/complete-normalization.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase2/complete-normalization.expected.json",
                ),
                (
                    "resume-enrich",
                    [
                        "resume",
                        "enrich",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase2/messy-unlabeled.enrichment-input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase2/messy-unlabeled.enrichment-expected.json",
                ),
                (
                    "variant-review",
                    [
                        "resume",
                        "variant-review",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase7/complete-variant-review.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase7/complete-variant-review.expected.json",
                ),
                (
                    "variant-materialize",
                    [
                        "resume",
                        "variant-materialize",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/resume/phase7/selected-variant-materialization.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/resume/phase7/selected-variant-materialization.expected.json",
                ),
                (
                    "job-normalize",
                    [
                        "job",
                        "normalize",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/job/phase4a/complete-normalization.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/job/phase4a/complete-normalization.expected.json",
                ),
                (
                    "job-match",
                    [
                        "job",
                        "match",
                        "--input",
                        str(
                            repository_root
                            / "fixtures/job/phase4b/complete-match.input.json"
                        ),
                    ],
                    repository_root
                    / "fixtures/job/phase4b/complete-match.expected.json",
                ),
            ]
            for name, arguments, golden in commands:
                compare_command(name, arguments, native, node, launcher_js, golden)
            shim = modules / ".bin/career.cmd"
            comspec = os.environ.get("COMSPEC")
            if not shim.is_file() or comspec is None:
                fail("installed Windows npm command shim is missing")
            shim_stdout, shim_stderr = run(
                [comspec, "/d", "/s", "/c", f'call "{shim}" --version'],
                "installed Windows npm command shim",
                cwd=consumer,
            )
            if shim_stderr or shim_stdout.strip() != b"career 0.1.1":
                fail("installed Windows npm command shim did not execute exact 0.1.1")
            print(
                f"Private Windows npm tarball, offline install, native parity, and allowlist tests passed for {platform_key}."
            )
        return 0
    except (TestError, OSError) as error:
        print(f"Windows npm CLI package test failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
