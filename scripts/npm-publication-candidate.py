#!/usr/bin/env python3
"""Build-time validation helpers for the exact Career Core npm candidate.

This module uses only the Python standard library. It never contacts npm,
authenticates, or publishes.
"""

from __future__ import annotations

import argparse
import base64
import gzip
import hashlib
import json
import os
import pathlib
import re
import stat
import sys
import tarfile
from typing import Any, Optional

VERSION = "0.1.1"
TAG = "v0.1.1"
REF = "refs/tags/v0.1.1"
REPOSITORY = "https://github.com/revazi/career-core"
NODE_VERSION = "v22.19.0"
NPM_VERSION = "11.6.2"
MAX_TARBALL_BYTES = 32 * 1024 * 1024
MAX_UNCOMPRESSED_TARBALL_BYTES = 20 * 1024 * 1024
MAX_BINARY_BYTES = 16 * 1024 * 1024
MAX_MANIFEST_BYTES = 32 * 1024
MAX_METADATA_BYTES = 64 * 1024
MAX_SOURCE_BYTES = 256 * 1024
MAX_README_BYTES = 16 * 1024
LIFECYCLE_NAMES = {
    "preinstall",
    "install",
    "postinstall",
    "prepack",
    "prepare",
    "postpack",
    "prepublish",
    "prepublishOnly",
    "publish",
    "postpublish",
}
LICENSE_FILES = {"LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY_NOTICES.md"}
LAUNCHER_NAME = "@revazi/career"
TARGET_CATALOG_SHA256 = (
    "9e56a3ca9b68799b0ff4bd52bbd2e71c2839d05a70398c5942062cb6e68032e2"
)
TARGET_CATALOG_PATH = (
    pathlib.Path(__file__).resolve().parent.parent / "npm/career/targets.json"
)


class CandidateError(ValueError):
    pass


def fail(message: str) -> None:
    raise CandidateError(message)


def same_file_identity(left: os.stat_result, right: os.stat_result) -> bool:
    if os.name == "nt":
        return left.st_size == right.st_size
    return (left.st_dev, left.st_ino, left.st_size, left.st_mode, left.st_mtime_ns) == (
        right.st_dev,
        right.st_ino,
        right.st_size,
        right.st_mode,
        right.st_mtime_ns,
    )


def bounded_file_bytes(path: pathlib.Path, maximum: int, label: str) -> bytes:
    try:
        before = path.lstat()
        if not stat.S_ISREG(before.st_mode):
            fail(f"{label} must be a regular non-symlink file")
        if not 1 <= before.st_size <= maximum:
            fail(f"{label} is outside its reviewed byte bound")
        with path.open("rb") as handle:
            opened = os.fstat(handle.fileno())
            if not stat.S_ISREG(opened.st_mode) or not same_file_identity(
                before, opened
            ):
                fail(f"{label} changed before it could be read")
            data = handle.read(maximum + 1)
        after = path.lstat()
    except OSError as error:
        raise CandidateError(f"{label} could not be read") from error
    if (
        len(data) != opened.st_size
        or len(data) > maximum
        or not same_file_identity(opened, after)
    ):
        fail(f"{label} changed or exceeded its reviewed byte bound while reading")
    return data


def load_object(
    path: pathlib.Path, maximum: int = MAX_METADATA_BYTES
) -> dict[str, Any]:
    try:
        value = json.loads(bounded_file_bytes(path, maximum, path.name).decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise CandidateError(f"invalid JSON object: {path.name}") from error
    if not isinstance(value, dict):
        fail(f"invalid JSON object: {path.name}")
    return value


def load_target_catalog() -> dict[str, dict[str, Any]]:
    data = bounded_file_bytes(TARGET_CATALOG_PATH, MAX_METADATA_BYTES, "target catalog")
    if hashlib.sha256(data).hexdigest() != TARGET_CATALOG_SHA256:
        raise RuntimeError("reviewed npm target catalog digest mismatch")
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise RuntimeError("reviewed npm target catalog is malformed") from error
    if (
        not isinstance(value, dict)
        or set(value) != {"schema_version", "targets"}
        or value.get("schema_version") != "career.npm_target_catalog.v1"
        or not isinstance(value.get("targets"), list)
        or len(value["targets"]) != 8
    ):
        raise RuntimeError("reviewed npm target catalog shape mismatch")
    targets: dict[str, dict[str, Any]] = {}
    for index, source in enumerate(value["targets"], start=1):
        target = dict(source)
        key = target["platform_key"]
        target.update(
            name=target["native_package"],
            libc=target["libc_family"],
            order=index * 10,
            file=f"{index * 10:02d}-revazi-career-{key}-{VERSION}.tgz",
        )
        if key in targets:
            raise RuntimeError("duplicate reviewed npm platform key")
        targets[key] = target
    return targets


TARGETS = load_target_catalog()
LAUNCHER_FILE = f"90-revazi-career-{VERSION}.tgz"


def write_object(path: pathlib.Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def exact_keys(value: Any, keys: set[str]) -> bool:
    return isinstance(value, dict) and set(value) == keys


def no_code_fields(manifest: dict[str, Any], allow_optional: bool) -> bool:
    forbidden = {"scripts", "dependencies", "devDependencies", "peerDependencies"}
    if not allow_optional:
        forbidden.add("optionalDependencies")
    return not forbidden.intersection(manifest) and not LIFECYCLE_NAMES.intersection(
        manifest
    )


def manifest_property_set(name: str, publication_candidate: bool) -> set[str]:
    keys = {"name", "version", "description", "license", "repository", "files"}
    keys.add("publishConfig" if publication_candidate else "private")
    if name == LAUNCHER_NAME:
        return keys | {
            "author",
            "homepage",
            "bugs",
            "keywords",
            "engines",
            "bin",
            "optionalDependencies",
            "career_launcher",
        }
    keys |= {"os", "cpu", "exports", "career_native"}
    target = next((value for value in TARGETS.values() if value["name"] == name), None)
    if target is not None and target["libc"] is not None:
        keys.add("libc")
    return keys


def validate_shared_manifest_metadata(manifest: dict[str, Any]) -> None:
    if manifest.get("license") != "MIT OR Apache-2.0":
        fail("package license metadata mismatch")
    if manifest.get("repository") != {
        "type": "git",
        "url": "git+https://github.com/revazi/career-core.git",
    }:
        fail("package repository metadata mismatch")
    if not bounded_text(manifest.get("description"), 256):
        fail("package description metadata is invalid")


def validate_source_template(manifest: dict[str, Any]) -> None:
    name = manifest.get("name")
    approved = {LAUNCHER_NAME, *(target["name"] for target in TARGETS.values())}
    if name not in approved or manifest.get("version") != VERSION:
        fail("source package identity/version is not approved")
    if set(manifest) != manifest_property_set(name, publication_candidate=False):
        fail("source package property set is not exact")
    if manifest.get("private") is not True:
        fail("source package template must retain private: true")
    validate_shared_manifest_metadata(manifest)
    allow_optional = name == LAUNCHER_NAME
    if not no_code_fields(manifest, allow_optional=allow_optional):
        fail("source package contains a lifecycle script or forbidden dependency")
    if name == LAUNCHER_NAME:
        expected = {target["name"]: VERSION for target in TARGETS.values()}
        optional = manifest.get("optionalDependencies")
        if optional != expected or list(optional) != list(expected):
            fail("launcher optional dependencies are not exact, ordered, and lockstep")


def make_public_manifest(source: pathlib.Path, destination: pathlib.Path) -> None:
    manifest = load_object(source, MAX_MANIFEST_BYTES)
    validate_source_template(manifest)
    del manifest["private"]
    manifest["publishConfig"] = {"access": "public", "provenance": True}
    write_object(destination, manifest)


def bounded_text(value: Any, maximum: int = 256) -> bool:
    return (
        isinstance(value, str)
        and 0 < len(value) <= maximum
        and re.search(r"[\x00-\x1f\x7f]", value) is None
    )


def promote_provenance(
    path: pathlib.Path,
    source_sha: str,
    runner_os: str,
    runner_arch: str,
    runner_image: str,
) -> None:
    provenance = load_object(path)
    source = provenance.get("source")
    build = provenance.get("build")
    if not isinstance(source, dict) or not isinstance(build, dict):
        fail("private native provenance is incomplete")
    if source.get("git_sha") != source_sha or source.get("git_dirty") is not False:
        fail("private native provenance does not bind the clean reviewed SHA")
    if source.get("publication_candidate") is not False:
        fail("private native provenance unexpectedly claims candidate status")
    source["git_ref"] = REF
    source["git_tag"] = TAG
    source["publication_candidate"] = True
    runner = build.get("runner")
    if not isinstance(runner, dict):
        fail("private native provenance has no runner record")
    runner["os"] = runner_os
    runner["arch"] = runner_arch
    runner["image"] = runner_image
    write_object(path, provenance)


class BoundedDecompressedReader:
    def __init__(self, source: gzip.GzipFile, maximum: int) -> None:
        self.source = source
        self.maximum = maximum
        self.total = 0

    def read(self, size: int = -1) -> bytes:
        remaining = self.maximum - self.total
        wanted = remaining + 1 if size < 0 else min(size, remaining + 1)
        data = self.source.read(wanted)
        self.total += len(data)
        if self.total > self.maximum:
            fail("candidate tarball exceeds its uncompressed byte bound")
        return data


def native_member_limits(target: dict[str, Any]) -> dict[str, int]:
    return {
        "package/package.json": MAX_MANIFEST_BYTES,
        f"package/{target['executable']}": min(
            target["maximum_binary_size_bytes"], MAX_BINARY_BYTES
        ),
        "package/provenance.json": MAX_METADATA_BYTES,
        **{f"package/{name}": MAX_SOURCE_BYTES for name in LICENSE_FILES},
    }


def launcher_member_limits() -> dict[str, int]:
    return {
        "package/package.json": MAX_MANIFEST_BYTES,
        "package/bin/career.js": MAX_SOURCE_BYTES,
        "package/targets.json": MAX_METADATA_BYTES,
        "package/README.md": MAX_README_BYTES,
        **{f"package/{name}": MAX_SOURCE_BYTES for name in LICENSE_FILES},
    }


def read_tarball(
    path: pathlib.Path, member_limits: dict[str, int]
) -> tuple[dict[str, bytes], dict[str, int]]:
    files: dict[str, bytes] = {}
    modes: dict[str, int] = {}
    try:
        before = path.lstat()
        if not stat.S_ISREG(before.st_mode):
            fail("candidate tarball must be a regular non-symlink file")
        if not 1 <= before.st_size <= MAX_TARBALL_BYTES:
            fail("candidate tarball is outside the 32 MiB compressed bound")
        with path.open("rb") as handle:
            opened = os.fstat(handle.fileno())
            if not stat.S_ISREG(opened.st_mode) or not same_file_identity(
                before, opened
            ):
                fail("candidate tarball changed before it could be read")
            with gzip.GzipFile(fileobj=handle, mode="rb") as decompressed:
                bounded = BoundedDecompressedReader(
                    decompressed, MAX_UNCOMPRESSED_TARBALL_BYTES
                )
                with tarfile.open(fileobj=bounded, mode="r|") as archive:
                    for member in archive:
                        if not member.isfile():
                            fail("candidate tarballs may contain only regular files")
                        if member.name not in member_limits:
                            fail("candidate tarball member allowlist mismatch")
                        if member.name in files:
                            fail("candidate tarball contains duplicate paths")
                        maximum = member_limits[member.name]
                        if not 1 <= member.size <= maximum:
                            fail(
                                "candidate tarball member exceeds its reviewed byte bound"
                            )
                        extracted = archive.extractfile(member)
                        if extracted is None:
                            fail("candidate tarball member is unreadable")
                        data = extracted.read(maximum + 1)
                        if len(data) != member.size or len(data) > maximum:
                            fail("candidate tarball member is truncated or oversized")
                        files[member.name] = data
                        modes[member.name] = stat.S_IMODE(member.mode)
        after = path.lstat()
    except (OSError, tarfile.TarError) as error:
        raise CandidateError("candidate tarball is malformed") from error
    if not same_file_identity(opened, after):
        fail("candidate tarball changed while it was being read")
    if set(files) != set(member_limits):
        fail("candidate tarball member count or allowlist mismatch")
    return files, modes


def parse_member_object(files: dict[str, bytes], name: str) -> dict[str, Any]:
    try:
        value = json.loads(files[name].decode("utf-8"))
    except (KeyError, UnicodeError, json.JSONDecodeError) as error:
        raise CandidateError(f"candidate member is invalid: {name}") from error
    if not isinstance(value, dict):
        fail(f"candidate member is invalid: {name}")
    return value


def validate_launcher_metadata(manifest: dict[str, Any]) -> None:
    if manifest.get("author") != "Revaz Zakalashvili":
        fail("candidate launcher author metadata mismatch")
    if manifest.get("homepage") != "https://github.com/revazi/career-core#readme":
        fail("candidate launcher homepage metadata mismatch")
    if manifest.get("bugs") != {"url": "https://github.com/revazi/career-core/issues"}:
        fail("candidate launcher bugs metadata mismatch")
    if manifest.get("keywords") != [
        "career",
        "resume",
        "job-search",
        "matching",
        "cli",
    ]:
        fail("candidate launcher keywords metadata mismatch")
    expected_optional = {target["name"]: VERSION for target in TARGETS.values()}
    optional = manifest.get("optionalDependencies")
    if optional != expected_optional or list(optional) != list(expected_optional):
        fail(
            "candidate launcher optional dependencies are not exact, ordered, and lockstep"
        )
    if manifest.get("bin") != {"career": "bin/career.js"}:
        fail("candidate launcher bin surface mismatch")
    if manifest.get("engines") != {"node": ">=22"}:
        fail("candidate launcher Node engine mismatch")
    if manifest.get("files") != [
        "bin/career.js",
        "targets.json",
        "README.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "THIRD_PARTY_NOTICES.md",
    ]:
        fail("candidate launcher files metadata mismatch")
    if manifest.get("career_launcher") != {
        "schema_version": "career.npm_launcher.v2",
        "executable": "career",
        "target_catalog": "targets.json",
        "platform_packages": [target["name"] for target in TARGETS.values()],
    }:
        fail("candidate launcher metadata mismatch")


def validate_native_manifest_metadata(
    manifest: dict[str, Any], target: dict[str, Any]
) -> None:
    if manifest.get("os") != [target["node_platform"]] or manifest.get("cpu") != [
        target["node_arch"]
    ]:
        fail("candidate native os/cpu metadata mismatch")
    if target["libc"] is None:
        if "libc" in manifest:
            fail("non-Linux candidate must not contain libc metadata")
    elif manifest.get("libc") != [target["libc"]]:
        fail("Linux candidate libc metadata mismatch")
    if manifest.get("files") != [
        target["executable"],
        "provenance.json",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "THIRD_PARTY_NOTICES.md",
    ]:
        fail("candidate native files metadata mismatch")
    if manifest.get("exports") != {"./package.json": "./package.json"}:
        fail("candidate native exports metadata mismatch")
    expected_native = {
        "schema_version": "career.npm_native_package.v2",
        "platform_key": target["platform_key"],
        "node_platform": target["node_platform"],
        "node_arch": target["node_arch"],
        "libc_family": target["libc_family"],
        "rust_target": target["rust_target"],
        "binary_file": target["executable"],
        "provenance_file": "provenance.json",
        "binary_format": target["binary_format"],
        "binary_architecture": target["binary_architecture"],
        "file_invariant": target["file_invariant"],
        "archive_mode": target["archive_mode"],
        "executable_mode": target["executable_mode"],
        "maximum_binary_size_bytes": target["maximum_binary_size_bytes"],
        "minimum_glibc_version": target["minimum_glibc_version"],
    }
    if manifest.get("career_native") != expected_native:
        fail("candidate native package metadata mismatch")


def validate_public_manifest(manifest: dict[str, Any], expected_name: str) -> None:
    if manifest.get("name") != expected_name or manifest.get("version") != VERSION:
        fail("candidate package identity/version mismatch")
    if set(manifest) != manifest_property_set(
        expected_name, publication_candidate=True
    ):
        fail("candidate package property set is not exact")
    if manifest.get("publishConfig") != {"access": "public", "provenance": True}:
        fail("candidate package public access/provenance metadata mismatch")
    validate_shared_manifest_metadata(manifest)
    allow_optional = expected_name == LAUNCHER_NAME
    if not no_code_fields(manifest, allow_optional=allow_optional):
        fail("candidate package contains a lifecycle script or forbidden dependency")
    if expected_name == LAUNCHER_NAME:
        validate_launcher_metadata(manifest)
        return
    target = next(
        (value for value in TARGETS.values() if value["name"] == expected_name), None
    )
    if target is None:
        fail("candidate native package is not approved")
    validate_native_manifest_metadata(manifest, target)


def valid_binary_header(binary: bytes, target: dict[str, Any]) -> bool:
    binary_format = target["binary_format"]
    architecture = target["binary_architecture"]
    if binary_format.startswith("mach-o-64-"):
        cpu = 0x0100000C if architecture == "aarch64" else 0x01000007
        return (
            len(binary) >= 8
            and int.from_bytes(binary[0:4], "little") == 0xFEEDFACF
            and int.from_bytes(binary[4:8], "little") == cpu
        )
    if binary_format.startswith("elf-64-"):
        machine = 0xB7 if architecture == "aarch64" else 0x3E
        return (
            len(binary) >= 20
            and binary[:6] == bytes([0x7F, 0x45, 0x4C, 0x46, 2, 1])
            and int.from_bytes(binary[18:20], "little") == machine
        )
    if not binary_format.startswith("pe32+-") or len(binary) < 64:
        return False
    offset = int.from_bytes(binary[0x3C:0x40], "little")
    machine = 0xAA64 if architecture == "aarch64" else 0x8664
    return (
        binary[:2] == b"MZ"
        and 64 <= offset
        and offset + 26 <= len(binary)
        and binary[offset : offset + 4] == b"PE\0\0"
        and int.from_bytes(binary[offset + 4 : offset + 6], "little") == machine
        and int.from_bytes(binary[offset + 24 : offset + 26], "little") == 0x020B
    )


def validate_runner(runner: Any, target: dict[str, Any]) -> None:
    expected_images = {
        "darwin-arm64": "macos-14",
        "darwin-x64": "macos-15-intel",
        "linux-x64-gnu": "ubuntu-22.04",
        "linux-arm64-gnu": "ubuntu-24.04-arm+ubuntu:22.04",
        "linux-x64-musl": (
            "ubuntu-22.04+node:22.19.0-alpine3.22+sha256:"
            "d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9"
        ),
        "linux-arm64-musl": (
            "ubuntu-24.04-arm+node:22.19.0-alpine3.22+sha256:"
            "d2166de198f26e17e5a442f537754dd616ab069c47cc57b889310a717e0abbf9"
        ),
        "win32-x64-msvc": "windows-2025",
        "win32-arm64-msvc": "windows-11-arm",
    }
    if not exact_keys(runner, {"os", "arch", "image", "libc"}):
        fail("candidate runner provenance property set mismatch")
    if (
        runner.get("os") != target["runner_os"]
        or runner.get("arch") != target["runner_arch"]
    ):
        fail("candidate runner OS/architecture mismatch")
    if runner.get("image") != expected_images.get(target["platform_key"]):
        fail("candidate runner image is not the exact reviewed native environment")
    libc = runner.get("libc")
    if target["libc"] is None:
        if libc is not None:
            fail("non-Linux candidate must not claim a libc")
    elif target["libc"] == "musl":
        if libc != "musl":
            fail("musl candidate must record an exact musl build runtime")
    elif libc != f"glibc {target['minimum_glibc_version']}":
        fail("GNU candidate must record the exact reviewed glibc build runtime")


def validate_provenance(
    provenance: dict[str, Any],
    manifest: dict[str, Any],
    target: dict[str, Any],
    source_sha: str,
) -> None:
    if (
        not exact_keys(
            provenance,
            {"schema_version", "package", "source", "build", "executable", "integrity"},
        )
        or provenance.get("schema_version") != "career.npm_native_provenance.v2"
    ):
        fail("candidate native provenance schema/property mismatch")
    package = provenance.get("package")
    expected_package = {
        "name": target["name"],
        "version": VERSION,
        "platform_key": target["platform_key"],
        "node_platform": target["node_platform"],
        "node_arch": target["node_arch"],
        "libc_family": target["libc_family"],
        "rust_target": target["rust_target"],
        "minimum_glibc_version": target["minimum_glibc_version"],
    }
    if package != expected_package:
        fail("candidate native package provenance mismatch")
    source = provenance.get("source")
    expected_source = {
        "repository": REPOSITORY,
        "git_sha": source_sha,
        "git_ref": REF,
        "git_tag": TAG,
        "git_dirty": False,
        "publication_candidate": True,
    }
    if source != expected_source:
        fail("candidate source provenance mismatch")
    build = provenance.get("build")
    expected_command = [
        "cargo",
        "build",
        "--release",
        "--locked",
        "-p",
        "career-cli",
        "--target",
        target["rust_target"],
    ]
    if not exact_keys(
        build,
        {"command", "profile", "locked", "rustc_version", "cargo_version", "runner"},
    ):
        fail("candidate build provenance property set mismatch")
    if build.get("command") != expected_command or build.get("profile") != "release":
        fail("candidate build command/profile mismatch")
    if build.get("locked") is not True:
        fail("candidate build is not locked")
    if not bounded_text(build.get("rustc_version"), 128) or not bounded_text(
        build.get("cargo_version"), 128
    ):
        fail("candidate toolchain provenance is invalid")
    if not build["rustc_version"].startswith("rustc 1.97.1 "):
        fail("candidate rustc provenance is not exact reviewed 1.97.1")
    if not build["cargo_version"].startswith("cargo 1.97.1 "):
        fail("candidate Cargo provenance is not exact reviewed 1.97.1")
    validate_runner(build.get("runner"), target)
    expected_integrity = {
        "npm_registry_integrity": "external_to_launcher_runtime",
        "package_contained_sha256": "consistency_only",
        "independent_signature": "absent",
    }
    if provenance.get("integrity") != expected_integrity:
        fail("candidate integrity/signature claims mismatch")
    executable = provenance.get("executable")
    if not exact_keys(
        executable,
        {
            "file_name",
            "binary_format",
            "binary_architecture",
            "file_invariant",
            "archive_mode",
            "mode",
            "size_bytes",
            "sha256",
        },
    ):
        fail("candidate executable provenance property set mismatch")
    expected_executable = {
        "file_name": target["executable"],
        "binary_format": target["binary_format"],
        "binary_architecture": target["binary_architecture"],
        "file_invariant": target["file_invariant"],
        "archive_mode": target["archive_mode"],
        "mode": target["executable_mode"],
    }
    if not all(
        executable.get(key) == value for key, value in expected_executable.items()
    ):
        fail("candidate executable target provenance mismatch")


def verify_native_tarball(
    path: pathlib.Path, platform_key: str, source_sha: str
) -> None:
    target = TARGETS.get(platform_key)
    if target is None:
        fail("unsupported native candidate target")
    files, modes = read_tarball(path, native_member_limits(target))
    binary_member = f"package/{target['executable']}"
    expected_files = {
        "package/package.json",
        binary_member,
        "package/provenance.json",
        *(f"package/{name}" for name in LICENSE_FILES),
    }
    if set(files) != expected_files:
        fail("native candidate tarball allowlist mismatch")
    expected_modes = {name: 0o644 for name in expected_files}
    expected_modes[binary_member] = int(target["archive_mode"], 8)
    if modes != expected_modes:
        fail("native candidate tarball mode allowlist mismatch")
    manifest = parse_member_object(files, "package/package.json")
    validate_public_manifest(manifest, target["name"])
    provenance = parse_member_object(files, "package/provenance.json")
    validate_provenance(provenance, manifest, target, source_sha)
    binary = files[binary_member]
    executable = provenance["executable"]
    if not 1 <= len(binary) <= target["maximum_binary_size_bytes"] or executable.get(
        "size_bytes"
    ) != len(binary):
        fail("candidate executable size mismatch")
    if modes[binary_member] != int(target["archive_mode"], 8):
        fail("candidate executable packed mode mismatch")
    if executable.get("sha256") != hashlib.sha256(binary).hexdigest():
        fail("candidate executable SHA-256 mismatch")
    if not valid_binary_header(binary, target):
        fail("candidate executable target format mismatch")


def verify_launcher_tarball(path: pathlib.Path) -> None:
    files, modes = read_tarball(path, launcher_member_limits())
    expected_files = {
        "package/package.json",
        "package/bin/career.js",
        "package/targets.json",
        "package/README.md",
        *(f"package/{name}" for name in LICENSE_FILES),
    }
    if set(files) != expected_files:
        fail("launcher candidate tarball allowlist mismatch")
    expected_modes = {name: 0o644 for name in expected_files}
    expected_modes["package/bin/career.js"] = 0o755
    if modes != expected_modes:
        fail("launcher candidate tarball mode allowlist mismatch")
    manifest = parse_member_object(files, "package/package.json")
    validate_public_manifest(manifest, LAUNCHER_NAME)
    if modes["package/bin/career.js"] != 0o755:
        fail("candidate launcher mode mismatch")


def file_integrity(path: pathlib.Path) -> dict[str, str]:
    data = bounded_file_bytes(path, MAX_TARBALL_BYTES, "candidate tarball")
    sha512 = base64.b64encode(hashlib.sha512(data).digest()).decode("ascii")
    return {"sha256": hashlib.sha256(data).hexdigest(), "integrity": f"sha512-{sha512}"}


def package_rows(directory: pathlib.Path) -> list[dict[str, Any]]:
    values: list[dict[str, Any]] = []
    for target in TARGETS.values():
        path = directory / target["file"]
        values.append(
            {
                "order": target["order"],
                "role": "internal_native",
                "name": target["name"],
                "version": VERSION,
                "file": target["file"],
                **file_integrity(path),
            }
        )
    launcher_path = directory / LAUNCHER_FILE
    values.append(
        {
            "order": 90,
            "role": "user_facing_launcher",
            "name": LAUNCHER_NAME,
            "version": VERSION,
            "file": LAUNCHER_FILE,
            **file_integrity(launcher_path),
        }
    )
    return values


def expected_publication_manifest(
    directory: pathlib.Path, source_sha: str
) -> dict[str, Any]:
    return {
        "schema_version": "career.npm_publication_candidate.v1",
        "source": {
            "repository": REPOSITORY,
            "git_sha": source_sha,
            "git_ref": REF,
            "git_tag": TAG,
            "git_dirty": False,
            "publication_candidate": True,
        },
        "release": {
            "version": VERSION,
            "node_version": NODE_VERSION,
            "npm_version": NPM_VERSION,
            "access": "public",
            "npm_provenance": "required",
            "package_contained_sha256": "consistency_only",
            "independent_signature": "absent",
        },
        "packages": package_rows(directory),
    }


def write_publication_manifest(directory: pathlib.Path, source_sha: str) -> None:
    write_object(
        directory / "publication-manifest.json",
        expected_publication_manifest(directory, source_sha),
    )


def verify_reviewed_package_bytes(
    directory: pathlib.Path, repository_root: pathlib.Path
) -> None:
    launcher_limits = launcher_member_limits()
    launcher_files, _ = read_tarball(directory / LAUNCHER_FILE, launcher_limits)
    reviewed = {
        "package/bin/career.js": repository_root / "npm/career/bin/career.js",
        "package/targets.json": repository_root / "npm/career/targets.json",
        "package/README.md": repository_root / "npm/career/README.md",
    }
    for name in LICENSE_FILES:
        reviewed[f"package/{name}"] = repository_root / name
    for member, source_path in reviewed.items():
        source = bounded_file_bytes(
            source_path, launcher_limits[member], f"reviewed {member}"
        )
        if launcher_files.get(member) != source:
            fail(f"launcher candidate {member} differs from reviewed source")
    if not 1 <= len(launcher_files["package/README.md"]) <= 16 * 1024:
        fail("launcher README is outside its 16 KiB publication bound")
    for target in TARGETS.values():
        limits = native_member_limits(target)
        native_files, _ = read_tarball(directory / target["file"], limits)
        for name in LICENSE_FILES:
            member = f"package/{name}"
            source = bounded_file_bytes(
                repository_root / name, limits[member], f"reviewed {name}"
            )
            if native_files.get(member) != source:
                fail(f"native candidate {name} differs from reviewed source")


def bounded_directory_entries(
    directory: pathlib.Path, maximum: int
) -> list[pathlib.Path]:
    entries: list[pathlib.Path] = []
    try:
        for path in directory.iterdir():
            if len(entries) >= maximum:
                fail("publication candidate directory exceeds its entry bound")
            entries.append(path)
    except OSError as error:
        raise CandidateError(
            "publication candidate directory could not be read"
        ) from error
    return entries


def verify_candidate(
    directory: pathlib.Path,
    source_sha: str,
    repository_root: Optional[pathlib.Path] = None,
) -> None:
    expected_names = {
        *(target["file"] for target in TARGETS.values()),
        LAUNCHER_FILE,
        "publication-manifest.json",
    }
    entries = bounded_directory_entries(directory, len(expected_names))
    actual = {path.name for path in entries}
    if actual != expected_names or any(
        path.is_symlink() or not path.is_file() for path in entries
    ):
        fail("publication candidate directory allowlist mismatch")
    for platform_key, target in TARGETS.items():
        verify_native_tarball(directory / target["file"], platform_key, source_sha)
    verify_launcher_tarball(directory / LAUNCHER_FILE)
    actual_manifest = load_object(
        directory / "publication-manifest.json", MAX_METADATA_BYTES
    )
    expected_manifest = expected_publication_manifest(directory, source_sha)
    if actual_manifest != expected_manifest:
        fail("publication manifest/order/integrity mismatch")
    rows = actual_manifest["packages"]
    if [row["order"] for row in rows] != [10, 20, 30, 40, 50, 60, 70, 80, 90]:
        fail("publication order must be all eight exact native packages, then launcher")
    if rows[-1]["name"] != LAUNCHER_NAME or rows[-1]["role"] != "user_facing_launcher":
        fail("user-facing launcher must be published last")
    if repository_root is not None:
        verify_reviewed_package_bytes(directory, repository_root)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    public_manifest = subparsers.add_parser("public-manifest")
    public_manifest.add_argument("source", type=pathlib.Path)
    public_manifest.add_argument("destination", type=pathlib.Path)

    promote = subparsers.add_parser("promote-provenance")
    promote.add_argument("path", type=pathlib.Path)
    promote.add_argument("--source-sha", required=True)
    promote.add_argument("--runner-os", required=True)
    promote.add_argument("--runner-arch", required=True)
    promote.add_argument("--runner-image", required=True)

    verify_native = subparsers.add_parser("verify-native")
    verify_native.add_argument("tarball", type=pathlib.Path)
    verify_native.add_argument("--platform-key", required=True, choices=sorted(TARGETS))
    verify_native.add_argument("--source-sha", required=True)

    write_manifest = subparsers.add_parser("write-manifest")
    write_manifest.add_argument("directory", type=pathlib.Path)
    write_manifest.add_argument("--source-sha", required=True)

    verify = subparsers.add_parser("verify")
    verify.add_argument("directory", type=pathlib.Path)
    verify.add_argument("--source-sha", required=True)
    verify.add_argument("--repository-root", type=pathlib.Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "public-manifest":
            make_public_manifest(args.source, args.destination)
        elif args.command == "promote-provenance":
            promote_provenance(
                args.path,
                args.source_sha,
                args.runner_os,
                args.runner_arch,
                args.runner_image,
            )
        elif args.command == "verify-native":
            verify_native_tarball(args.tarball, args.platform_key, args.source_sha)
        elif args.command == "write-manifest":
            write_publication_manifest(args.directory, args.source_sha)
        elif args.command == "verify":
            verify_candidate(args.directory, args.source_sha, args.repository_root)
        else:  # pragma: no cover
            fail("unsupported command")
    except CandidateError as error:
        print(
            f"npm publication candidate verification failed: {error}", file=sys.stderr
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
