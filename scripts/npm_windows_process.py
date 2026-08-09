"""Bounded subprocess execution shared by native Windows npm policy scripts."""

from __future__ import annotations

import os
import pathlib
import subprocess
import tempfile
import time


class BoundedProcessError(ValueError):
    pass


def run_bounded(
    command: list[str],
    label: str,
    *,
    maximum_output_bytes: int,
    cwd: pathlib.Path | None = None,
    env: dict[str, str] | None = None,
    expected_code: int = 0,
    timeout: int = 180,
) -> tuple[bytes, bytes]:
    process: subprocess.Popen[bytes] | None = None
    try:
        with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                command, cwd=cwd, env=env, stdout=stdout, stderr=stderr
            )
            deadline = time.monotonic() + timeout
            while process.poll() is None:
                stdout_size = os.fstat(stdout.fileno()).st_size
                stderr_size = os.fstat(stderr.fileno()).st_size
                if stdout_size + stderr_size > maximum_output_bytes:
                    process.kill()
                    process.wait(timeout=10)
                    raise BoundedProcessError(
                        f"{label} output exceeds its reviewed bound"
                    )
                if time.monotonic() >= deadline:
                    process.kill()
                    process.wait(timeout=10)
                    raise BoundedProcessError(f"{label} timed out")
                time.sleep(0.01)
            stdout_size = os.fstat(stdout.fileno()).st_size
            stderr_size = os.fstat(stderr.fileno()).st_size
            if stdout_size + stderr_size > maximum_output_bytes:
                raise BoundedProcessError(f"{label} output exceeds its reviewed bound")
            stdout.seek(0)
            stderr.seek(0)
            stdout_bytes = stdout.read(maximum_output_bytes + 1)
            stderr_bytes = stderr.read(maximum_output_bytes + 1)
    except BoundedProcessError:
        raise
    except (OSError, subprocess.SubprocessError) as error:
        if process is not None and process.poll() is None:
            process.kill()
        raise BoundedProcessError(f"{label} could not run") from error
    if process.returncode != expected_code:
        diagnostic = stderr_bytes[:512].decode("utf-8", errors="replace").strip()
        raise BoundedProcessError(
            f"{label} returned an unexpected exit code: {diagnostic}"
        )
    return stdout_bytes, stderr_bytes
