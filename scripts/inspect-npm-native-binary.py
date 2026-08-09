#!/usr/bin/env python3
"""Inspect one exact native npm CLI binary without network or publication."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import re
import stat
import subprocess
import sys
import tempfile
from typing import Any

CATALOG_SHA256 = "9e56a3ca9b68799b0ff4bd52bbd2e71c2839d05a70398c5942062cb6e68032e2"
MAX_BINARY_BYTES = 16 * 1024 * 1024
MAX_CATALOG_BYTES = 64 * 1024
MAX_COMMAND_OUTPUT_BYTES = 8 * 1024 * 1024
MAX_EVIDENCE_BYTES = 64 * 1024
SUPPORTED_TARGETS = {
    "aarch64-apple-darwin": ("Darwin", "arm64"),
    "x86_64-apple-darwin": ("Darwin", "x86_64"),
    "x86_64-unknown-linux-gnu": ("Linux", "x86_64"),
    "aarch64-unknown-linux-gnu": ("Linux", "aarch64"),
    "x86_64-unknown-linux-musl": ("Linux", "x86_64"),
    "aarch64-unknown-linux-musl": ("Linux", "aarch64"),
    "x86_64-pc-windows-msvc": ("Windows", "x86_64"),
    "aarch64-pc-windows-msvc": ("Windows", "arm64"),
}
MUSL_IMAGE_DIGEST = "d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9"
EXACT_RUNNER_IMAGES = {
    "aarch64-apple-darwin": "macos-14",
    "x86_64-apple-darwin": "macos-15-intel",
    "x86_64-unknown-linux-gnu": "ubuntu-22.04",
    "aarch64-unknown-linux-gnu": "ubuntu-24.04-arm+ubuntu:22.04",
    "x86_64-unknown-linux-musl": (
        f"ubuntu-22.04+node:22.19.0-alpine3.22+sha256:{MUSL_IMAGE_DIGEST}"
    ),
    "aarch64-unknown-linux-musl": (
        f"ubuntu-24.04-arm+node:22.19.0-alpine3.22+sha256:{MUSL_IMAGE_DIGEST}"
    ),
    "x86_64-pc-windows-msvc": "windows-2025",
    "aarch64-pc-windows-msvc": "windows-11-arm",
}
WINDOWS_SYSTEM_IMPORTS = {
    "advapi32.dll",
    "bcrypt.dll",
    "crypt32.dll",
    "iphlpapi.dll",
    "kernel32.dll",
    "msvcrt.dll",
    "normaliz.dll",
    "ntdll.dll",
    "ole32.dll",
    "oleaut32.dll",
    "rpcrt4.dll",
    "secur32.dll",
    "shell32.dll",
    "user32.dll",
    "ucrtbase.dll",
    "userenv.dll",
    "version.dll",
    "vcruntime140.dll",
    "winmm.dll",
    "ws2_32.dll",
}


class InspectionError(ValueError):
    pass


def fail(message: str) -> None:
    raise InspectionError(message)


def same_file(left: os.stat_result, right: os.stat_result) -> bool:
    if os.name == "nt":
        return left.st_ino != 0 and (left.st_dev, left.st_ino, left.st_size) == (
            right.st_dev,
            right.st_ino,
            right.st_size,
        )
    return (left.st_dev, left.st_ino, left.st_size, left.st_mode, left.st_mtime_ns) == (
        right.st_dev,
        right.st_ino,
        right.st_size,
        right.st_mode,
        right.st_mtime_ns,
    )


def bounded_regular_bytes(path: pathlib.Path, maximum: int, label: str) -> bytes:
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or not 1 <= before.st_size <= maximum:
        fail(f"{label} must be one bounded regular non-symlink file")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags)
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode) or not same_file(before, opened):
            fail(f"{label} changed before inspection")
        chunks: list[bytes] = []
        total = 0
        while total <= maximum:
            chunk = os.read(descriptor, min(64 * 1024, maximum + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
    finally:
        os.close(descriptor)
    after = path.lstat()
    data = b"".join(chunks)
    if (
        len(data) != opened.st_size
        or len(data) > maximum
        or not same_file(opened, after)
    ):
        fail(f"{label} changed or exceeded its byte bound during inspection")
    return data


def load_catalog(repository_root: pathlib.Path) -> tuple[dict[str, Any], bytes]:
    path = repository_root / "npm/career/targets.json"
    data = bounded_regular_bytes(path, MAX_CATALOG_BYTES, "target catalog")
    if hashlib.sha256(data).hexdigest() != CATALOG_SHA256:
        fail("target catalog bytes are not the reviewed catalog")
    try:
        document = json.loads(data)
    except json.JSONDecodeError as error:
        raise InspectionError("target catalog is malformed") from error
    if (
        not isinstance(document, dict)
        or set(document) != {"schema_version", "targets"}
        or document.get("schema_version") != "career.npm_target_catalog.v1"
        or not isinstance(document.get("targets"), list)
        or len(document["targets"]) != 8
    ):
        fail("target catalog shape is invalid")
    return document, data


def selected_target(document: dict[str, Any], rust_target: str) -> dict[str, Any]:
    matches = [
        value
        for value in document["targets"]
        if value.get("rust_target") == rust_target
    ]
    if len(matches) != 1 or rust_target not in SUPPORTED_TARGETS:
        fail("inspection target is not approved for this native evidence gate")
    return matches[0]


def normalized_machine() -> str:
    machine = platform.machine().lower()
    aliases = {"amd64": "x86_64", "arm64": "arm64", "aarch64": "aarch64"}
    return aliases.get(machine, machine)


def require_native_host(rust_target: str) -> tuple[str, str]:
    expected_system, expected_machine = SUPPORTED_TARGETS[rust_target]
    actual_system = platform.system()
    actual_machine = normalized_machine()
    accepted_machines = {expected_machine}
    if expected_machine in {"arm64", "aarch64"}:
        accepted_machines = {"arm64", "aarch64"}
    if actual_system != expected_system or actual_machine not in accepted_machines:
        fail("binary inspection is not running on the exact native OS and architecture")
    return actual_system, actual_machine


def limit_command_output_file_size() -> None:
    import resource

    resource.setrlimit(
        resource.RLIMIT_FSIZE, (MAX_COMMAND_OUTPUT_BYTES, MAX_COMMAND_OUTPUT_BYTES)
    )


def run_bounded_outputs(
    command: list[str], label: str, allowed_returncodes: set[int]
) -> tuple[str, str]:
    try:
        with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
            command_options: dict[str, Any] = {
                "check": False,
                "stdout": stdout,
                "stderr": stderr,
                "timeout": 60,
            }
            if os.name != "nt":
                command_options["preexec_fn"] = limit_command_output_file_size
            result = subprocess.run(command, **command_options)
            stdout_size = os.fstat(stdout.fileno()).st_size
            stderr_size = os.fstat(stderr.fileno()).st_size
            if stdout_size + stderr_size > MAX_COMMAND_OUTPUT_BYTES:
                fail(f"{label} output exceeds its reviewed bound")
            stdout.seek(0)
            stderr.seek(0)
            stdout_bytes = stdout.read(MAX_COMMAND_OUTPUT_BYTES + 1)
            stderr_bytes = stderr.read(MAX_COMMAND_OUTPUT_BYTES + 1)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise InspectionError(f"{label} could not run") from error
    if result.returncode not in allowed_returncodes:
        fail(f"{label} failed")
    try:
        return stdout_bytes.decode("utf-8"), stderr_bytes.decode("utf-8")
    except UnicodeError as error:
        raise InspectionError(f"{label} output is not UTF-8") from error


def run_bounded(command: list[str], label: str) -> str:
    stdout, _ = run_bounded_outputs(command, label, {0})
    return stdout


def verify_header(binary: bytes, target: dict[str, Any]) -> None:
    binary_format = target["binary_format"]
    architecture = target["binary_architecture"]
    if binary_format.startswith("mach-o-64-"):
        expected_cpu = 0x0100000C if architecture == "aarch64" else 0x01000007
        valid = (
            len(binary) >= 8
            and int.from_bytes(binary[0:4], "little") == 0xFEEDFACF
            and int.from_bytes(binary[4:8], "little") == expected_cpu
        )
    elif binary_format.startswith("elf-64-"):
        expected_machine = 0xB7 if architecture == "aarch64" else 0x3E
        valid = (
            binary[:6] == bytes([0x7F, 0x45, 0x4C, 0x46, 2, 1])
            and len(binary) >= 20
            and int.from_bytes(binary[18:20], "little") == expected_machine
        )
    else:
        pe_offset = (
            int.from_bytes(binary[0x3C:0x40], "little") if len(binary) >= 64 else 0
        )
        expected_machine = 0xAA64 if architecture == "aarch64" else 0x8664
        valid = (
            binary_format.startswith("pe32+-")
            and binary[:2] == b"MZ"
            and 64 <= pe_offset
            and pe_offset + 26 <= len(binary)
            and binary[pe_offset : pe_offset + 4] == b"PE\0\0"
            and int.from_bytes(binary[pe_offset + 4 : pe_offset + 6], "little")
            == expected_machine
            and int.from_bytes(binary[pe_offset + 24 : pe_offset + 26], "little")
            == 0x020B
        )
    if not valid:
        fail("native executable header does not match the reviewed target")


def version_tuple(value: str) -> tuple[int, ...]:
    if re.fullmatch(r"(?:0|[1-9][0-9]*)(?:\.(?:0|[1-9][0-9]*)){1,3}", value) is None:
        fail("GNU libc symbol version is malformed")
    return tuple(int(part) for part in value.split("."))


def inspect_darwin(binary_path: pathlib.Path, target: dict[str, Any]) -> dict[str, Any]:
    expected_arch = "arm64" if target["binary_architecture"] == "aarch64" else "x86_64"
    architectures = run_bounded(
        ["lipo", "-archs", str(binary_path)], "lipo inspection"
    ).strip()
    if architectures != expected_arch:
        fail("Mach-O binary is not one exact thin reviewed architecture")
    output = run_bounded(["otool", "-L", str(binary_path)], "Mach-O import inspection")
    lines = output.splitlines()
    imports = sorted(
        {line.strip().split(" ", 1)[0] for line in lines[1:] if line.strip()}
    )
    if not 1 <= len(imports) <= 128:
        fail("Mach-O dynamic import count is outside its reviewed bound")
    if any(
        not value.startswith(("/usr/lib/", "/System/Library/")) for value in imports
    ):
        fail("Mach-O binary imports a non-system dynamic library")
    return {
        "linkage": "dynamic",
        "interpreter": None,
        "dynamic_imports": imports,
        "highest_glibc_symbol_version": None,
        "runtime_libc": None,
    }


def inspect_linux_gnu(
    binary_path: pathlib.Path, target: dict[str, Any]
) -> dict[str, Any]:
    program_headers = run_bounded(
        ["readelf", "--program-headers", "--wide", str(binary_path)],
        "ELF program-header inspection",
    )
    interpreter_matches = re.findall(
        r"Requesting program interpreter: ([^\]]+)", program_headers
    )
    expected_interpreter = (
        "/lib/ld-linux-aarch64.so.1"
        if target["binary_architecture"] == "aarch64"
        else "/lib64/ld-linux-x86-64.so.2"
    )
    if interpreter_matches != [expected_interpreter]:
        fail("ELF interpreter does not match the reviewed GNU target")
    dynamic = run_bounded(
        ["readelf", "--dynamic", "--wide", str(binary_path)], "ELF import inspection"
    )
    imports = sorted(set(re.findall(r"Shared library: \[([^\]]+)\]", dynamic)))
    if not 1 <= len(imports) <= 128:
        fail("ELF dynamic import count is outside its reviewed bound")
    symbols = run_bounded(
        ["readelf", "--dyn-syms", "--wide", str(binary_path)], "GNU symbol inspection"
    )
    versions = sorted(
        set(re.findall(r"GLIBC_([0-9]+(?:\.[0-9]+)+)", symbols)), key=version_tuple
    )
    if not versions:
        fail("GNU binary has no bounded imported GLIBC symbol evidence")
    highest = versions[-1]
    minimum = target["minimum_glibc_version"]
    if version_tuple(highest) > version_tuple(minimum):
        fail("GNU binary imports a GLIBC symbol newer than the proposed floor")
    runtime_libc = run_bounded(
        ["getconf", "GNU_LIBC_VERSION"], "GNU libc runtime inspection"
    ).strip()
    if re.fullmatch(r"glibc [0-9]+(?:\.[0-9]+){1,3}", runtime_libc) is None:
        fail("GNU libc runtime evidence is malformed")
    return {
        "linkage": "dynamic",
        "interpreter": expected_interpreter,
        "dynamic_imports": imports,
        "highest_glibc_symbol_version": highest,
        "runtime_libc": runtime_libc,
    }


def inspect_linux_musl(
    binary_path: pathlib.Path, target: dict[str, Any]
) -> dict[str, Any]:
    program_headers = run_bounded(
        ["readelf", "--program-headers", "--wide", str(binary_path)],
        "musl ELF program-header inspection",
    )
    if re.findall(r"Requesting program interpreter: ([^\]]+)", program_headers):
        fail("static musl ELF must not contain a program interpreter")
    dynamic = run_bounded(
        ["readelf", "--dynamic", "--wide", str(binary_path)],
        "musl ELF import inspection",
    )
    if re.findall(r"Shared library: \[([^\]]+)\]", dynamic):
        fail("musl ELF contains an unexpected dynamic import")
    symbols = run_bounded(
        ["readelf", "--dyn-syms", "--wide", str(binary_path)],
        "musl ELF symbol inspection",
    )
    if re.search(r"GLIBC_[0-9]", symbols):
        fail("musl ELF contains an unexpected imported GLIBC symbol")
    header = run_bounded(
        ["readelf", "--file-header", "--wide", str(binary_path)],
        "musl ELF type inspection",
    )
    if target["binary_architecture"] == "x86_64":
        valid_type = re.search(
            r"^\s*Type:\s+DYN \(Position-Independent Executable file\)\s*$",
            header,
            re.MULTILINE,
        )
        if valid_type is None or "Flags: NOW PIE" not in dynamic:
            fail("x86-64 musl ELF is not one reviewed static PIE")
        linkage_kind = "static-pie"
    else:
        valid_type = re.search(
            r"^\s*Type:\s+EXEC \(Executable file\)\s*$", header, re.MULTILINE
        )
        if (
            valid_type is None
            or "There is no dynamic section in this file." not in dynamic
        ):
            fail("AArch64 musl ELF is not one reviewed static executable")
        linkage_kind = "static"
    loader_arch = "aarch64" if target["binary_architecture"] == "aarch64" else "x86_64"
    loader = f"/lib/ld-musl-{loader_arch}.so.1"
    _, loader_stderr = run_bounded_outputs([loader], "musl runtime inspection", {1})
    match = re.search(
        rf"^musl libc \({loader_arch}\)\nVersion ([0-9]+(?:\.[0-9]+){{1,3}})\n",
        loader_stderr,
        re.MULTILINE,
    )
    if match is None:
        fail("musl runtime evidence is malformed or architecture-mismatched")
    return {
        "linkage": linkage_kind,
        "interpreter": None,
        "dynamic_imports": [],
        "highest_glibc_symbol_version": None,
        "runtime_libc": f"musl {match.group(1)}",
    }


def inspect_linux(binary_path: pathlib.Path, target: dict[str, Any]) -> dict[str, Any]:
    if target["libc_family"] == "glibc":
        return inspect_linux_gnu(binary_path, target)
    if target["libc_family"] == "musl":
        return inspect_linux_musl(binary_path, target)
    fail("Linux inspection target has no reviewed libc family")


def pe_integer(binary: bytes, offset: int, width: int, label: str) -> int:
    if offset < 0 or offset + width > len(binary):
        fail(f"PE {label} is outside the bounded executable")
    return int.from_bytes(binary[offset : offset + width], "little")


def overlapping_ranges(values: list[tuple[int, int]]) -> bool:
    ordered = sorted((start, end) for start, end in values if end > start)
    return any(left[1] > right[0] for left, right in zip(ordered, ordered[1:]))


def pe_layout(binary: bytes) -> tuple[int, int, list[tuple[int, int, int, int]]]:
    pe_offset = pe_integer(binary, 0x3C, 4, "header offset")
    if pe_offset < 64 or binary[pe_offset : pe_offset + 4] != b"PE\0\0":
        fail("PE signature is invalid")
    section_count = pe_integer(binary, pe_offset + 6, 2, "section count")
    optional_size = pe_integer(binary, pe_offset + 20, 2, "optional-header size")
    characteristics = pe_integer(binary, pe_offset + 22, 2, "characteristics")
    optional_offset = pe_offset + 24
    if not 1 <= section_count <= 96 or optional_size < 128:
        fail("PE section or optional-header bounds are invalid")
    if pe_integer(binary, optional_offset, 2, "optional-header magic") != 0x020B:
        fail("PE executable is not PE32+")
    if characteristics & 0x0002 == 0:
        fail("PE file is not marked as an executable image")
    section_offset = optional_offset + optional_size
    section_table_end = section_offset + section_count * 40
    if section_table_end > len(binary):
        fail("PE section table exceeds the bounded executable")
    size_of_headers = pe_integer(binary, optional_offset + 60, 4, "header size")
    if not section_table_end <= size_of_headers <= len(binary):
        fail("PE header size does not contain the section table")
    sections = []
    virtual_ranges = []
    file_ranges = []
    for index in range(section_count):
        offset = section_offset + index * 40
        virtual_address = pe_integer(binary, offset + 12, 4, "section virtual address")
        virtual_size = pe_integer(binary, offset + 8, 4, "section virtual size")
        file_offset = pe_integer(binary, offset + 20, 4, "section file offset")
        file_size = pe_integer(binary, offset + 16, 4, "section file size")
        virtual_span = max(virtual_size, file_size)
        if virtual_span > 0:
            virtual_end = virtual_address + virtual_span
            if virtual_end > 2**32:
                fail("PE section virtual range overflows")
            virtual_ranges.append((virtual_address, virtual_end))
        if file_size > 0:
            file_end = file_offset + file_size
            if file_offset < size_of_headers or file_end > len(binary):
                fail("PE section file range is outside bounded section data")
            file_ranges.append((file_offset, file_end))
        sections.append((virtual_address, virtual_size, file_offset, file_size))
    if overlapping_ranges(virtual_ranges) or overlapping_ranges(file_ranges):
        fail("PE section ranges overlap")
    return optional_offset, size_of_headers, sections


def pe_rva_span(
    binary: bytes,
    rva: int,
    length: int,
    size_of_headers: int,
    sections: list[tuple[int, int, int, int]],
) -> tuple[int, int]:
    if length < 1 or rva + length > 2**32:
        fail("PE import range length or RVA is invalid")
    if rva < size_of_headers:
        if rva + length > size_of_headers:
            fail("PE import range crosses the bounded header region")
        return rva, size_of_headers - rva
    for virtual_address, virtual_size, file_offset, file_size in sections:
        mapped_size = min(virtual_size or file_size, file_size)
        if virtual_address <= rva and rva + length <= virtual_address + mapped_size:
            delta = rva - virtual_address
            offset = file_offset + delta
            if offset + length > len(binary):
                break
            return offset, mapped_size - delta
    fail("PE import range does not map wholly within bounded file data")


def pe_ascii_name(binary: bytes, offset: int, available: int) -> str:
    maximum = min(len(binary), offset + available, offset + 261)
    end = binary.find(b"\0", offset, maximum)
    if end < 0 or end == offset:
        fail("PE import name is missing or exceeds its mapped bound")
    try:
        value = binary[offset:end].decode("ascii").lower()
    except UnicodeError as error:
        raise InspectionError("PE import name is not ASCII") from error
    if re.fullmatch(r"[a-z0-9._-]{1,260}\.dll", value) is None:
        fail("PE import name is not one bounded DLL basename")
    return value


def approved_windows_import(value: str) -> bool:
    return value in WINDOWS_SYSTEM_IMPORTS


def inspect_windows(binary: bytes) -> dict[str, Any]:
    optional_offset, size_of_headers, sections = pe_layout(binary)
    directory_count = pe_integer(
        binary, optional_offset + 108, 4, "data-directory count"
    )
    if directory_count < 2:
        fail("PE executable has no import directory")
    import_rva = pe_integer(binary, optional_offset + 120, 4, "import-directory RVA")
    import_size = pe_integer(binary, optional_offset + 124, 4, "import-directory size")
    if import_rva == 0 or not 20 <= import_size <= 1024 * 1024:
        fail("PE import directory is missing or outside its reviewed bound")
    import_offset, _ = pe_rva_span(
        binary, import_rva, import_size, size_of_headers, sections
    )
    imports = []
    terminated = False
    descriptor_limit = min(129, import_size // 20)
    for index in range(descriptor_limit):
        descriptor = import_offset + index * 20
        values = [
            pe_integer(binary, descriptor + offset, 4, "import descriptor")
            for offset in range(0, 20, 4)
        ]
        if values == [0, 0, 0, 0, 0]:
            terminated = True
            break
        name_rva = values[3]
        name_offset, name_available = pe_rva_span(
            binary, name_rva, 1, size_of_headers, sections
        )
        imports.append(pe_ascii_name(binary, name_offset, name_available))
    unique_imports = sorted(set(imports))
    if not terminated or not 1 <= len(unique_imports) <= 128:
        fail("PE dynamic import count or termination is outside its reviewed bound")
    if any(not approved_windows_import(value) for value in unique_imports):
        fail("PE executable imports a non-reviewed Windows system DLL")
    return {
        "linkage": "dynamic",
        "interpreter": None,
        "dynamic_imports": unique_imports,
        "highest_glibc_symbol_version": None,
        "runtime_libc": None,
    }


def require_source_state(
    repository_root: pathlib.Path, source_sha: str, evidence_kind: str
) -> None:
    head = run_bounded(
        ["git", "-C", str(repository_root), "rev-parse", "HEAD"],
        "source SHA inspection",
    ).strip()
    if head != source_sha:
        fail("native evidence source SHA does not match the checked-out commit")
    if evidence_kind != "exact_native_ci":
        return
    status_output = run_bounded(
        [
            "git",
            "-C",
            str(repository_root),
            "status",
            "--porcelain",
            "--untracked-files=normal",
        ],
        "source cleanliness inspection",
    )
    if status_output:
        fail("exact native CI evidence requires a clean source checkout")
    rustc_version = run_bounded(["rustc", "--version"], "rustc inspection").strip()
    cargo_version = run_bounded(["cargo", "--version"], "Cargo inspection").strip()
    if not rustc_version.startswith("rustc 1.97.1 ") or not cargo_version.startswith(
        "cargo 1.97.1 "
    ):
        fail("exact native CI evidence requires rustc and Cargo 1.97.1")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repository-root", type=pathlib.Path, required=True)
    parser.add_argument("--binary", type=pathlib.Path, required=True)
    parser.add_argument("--target", required=True, choices=sorted(SUPPORTED_TARGETS))
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--runner-image", required=True)
    parser.add_argument(
        "--evidence-kind", required=True, choices=["exact_native_ci", "local_policy"]
    )
    parser.add_argument("--output", type=pathlib.Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        repository_root = args.repository_root.resolve(strict=True)
        output = args.output.resolve(strict=False)
        if repository_root == output or repository_root in output.parents:
            fail("native inspection evidence must be written outside the checkout")
        if re.fullmatch(r"[0-9a-f]{40}", args.source_sha) is None:
            fail("source SHA must be one full lowercase commit identifier")
        if re.fullmatch(r"[A-Za-z0-9._/+:-]{1,128}", args.runner_image) is None:
            fail("runner image label is outside its reviewed bound")
        require_source_state(repository_root, args.source_sha, args.evidence_kind)
        document, catalog_bytes = load_catalog(repository_root)
        target = selected_target(document, args.target)
        system, machine = require_native_host(args.target)
        binary_argument = args.binary.expanduser()
        if stat.S_ISLNK(binary_argument.lstat().st_mode):
            fail("native executable path must not be a symbolic link")
        binary_path = binary_argument.resolve(strict=True)
        binary = bounded_regular_bytes(
            binary_path, target["maximum_binary_size_bytes"], "native executable"
        )
        if system != "Windows" and stat.S_IMODE(binary_path.lstat().st_mode) != 0o755:
            fail("native executable mode is not exactly 0755")
        if system == "Windows" and target["executable"] != "career.exe":
            fail("Windows native executable name is not exact")
        verify_header(binary, target)
        observed_version = run_bounded(
            [str(binary_path), "--version"], "native execution"
        ).strip()
        if observed_version != "career 0.1.1":
            fail("native executable version is not exact lockstep 0.1.1")
        if system == "Darwin":
            linkage = inspect_darwin(binary_path, target)
        elif system == "Linux":
            linkage = inspect_linux(binary_path, target)
        else:
            linkage = inspect_windows(binary)
        if args.evidence_kind == "exact_native_ci":
            if args.runner_image != EXACT_RUNNER_IMAGES[args.target]:
                fail(
                    "exact native CI evidence runner image does not match the reviewed target"
                )
            if (
                target["libc_family"] == "glibc"
                and linkage["runtime_libc"] != "glibc 2.35"
            ):
                fail("exact GNU CI evidence requires Ubuntu 22.04 glibc 2.35 userland")
            if (
                target["libc_family"] == "musl"
                and linkage["runtime_libc"] != "musl 1.2.5"
            ):
                fail("exact musl CI evidence requires Alpine 3.22 musl 1.2.5 userland")
        evidence = {
            "schema_version": "career.npm_native_inspection.v1",
            "evidence_kind": args.evidence_kind,
            "source_sha": args.source_sha,
            "target_catalog_sha256": hashlib.sha256(catalog_bytes).hexdigest(),
            "runner": {
                "image": args.runner_image,
                "system": system,
                "machine": machine,
            },
            "target": {
                "platform_key": target["platform_key"],
                "rust_target": target["rust_target"],
                "node_platform": target["node_platform"],
                "node_arch": target["node_arch"],
                "libc_family": target["libc_family"],
            },
            "binary": {
                "file_name": target["executable"],
                "binary_format": target["binary_format"],
                "binary_architecture": target["binary_architecture"],
                "size_bytes": len(binary),
                "sha256": hashlib.sha256(binary).hexdigest(),
                "mode": target["executable_mode"],
                "observed_version": observed_version,
            },
            "linkage": linkage,
        }
        encoded = (json.dumps(evidence, indent=2, sort_keys=True) + "\n").encode(
            "utf-8"
        )
        if len(encoded) > MAX_EVIDENCE_BYTES:
            fail("native inspection evidence exceeds its reviewed bound")
        output.parent.mkdir(parents=True, exist_ok=True)
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
        descriptor = os.open(output, flags, 0o600)
        try:
            written = 0
            while written < len(encoded):
                count = os.write(descriptor, encoded[written:])
                if count <= 0:
                    fail("native inspection evidence could not be written completely")
                written += count
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
        print(encoded.decode("utf-8"), end="")
        return 0
    except (InspectionError, OSError) as error:
        print(f"native npm binary inspection failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
