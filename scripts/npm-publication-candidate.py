#!/usr/bin/env python3
"""Build-time validation helpers for the exact Career Core npm candidate.

This module uses only the Python standard library. It never contacts npm,
authenticates, or publishes.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import pathlib
import re
import stat
import sys
import tarfile
from typing import Any, Optional

VERSION = "0.1.0"
TAG = "v0.1.0"
REF = "refs/tags/v0.1.0"
REPOSITORY = "https://github.com/revazi/career-core"
NODE_VERSION = "v22.19.0"
NPM_VERSION = "11.6.2"
MAX_TARBALL_BYTES = 32 * 1024 * 1024
MAX_BINARY_BYTES = 16 * 1024 * 1024
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
TARGETS = {
    "darwin-arm64": {
        "name": "@revazi/career-darwin-arm64",
        "node_platform": "darwin",
        "node_arch": "arm64",
        "rust_target": "aarch64-apple-darwin",
        "binary_format": "mach-o-64-aarch64",
        "runner_os": "macOS",
        "runner_arch": "ARM64",
        "libc": None,
        "order": 10,
        "file": "10-revazi-career-darwin-arm64-0.1.0.tgz",
    },
    "linux-x64-gnu": {
        "name": "@revazi/career-linux-x64-gnu",
        "node_platform": "linux",
        "node_arch": "x64",
        "rust_target": "x86_64-unknown-linux-gnu",
        "binary_format": "elf-64-x86_64",
        "runner_os": "Linux",
        "runner_arch": "X64",
        "libc": "glibc",
        "order": 20,
        "file": "20-revazi-career-linux-x64-gnu-0.1.0.tgz",
    },
}
LAUNCHER_FILE = "30-revazi-career-0.1.0.tgz"


class CandidateError(ValueError):
    pass


def fail(message: str) -> None:
    raise CandidateError(message)


def load_object(path: pathlib.Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise CandidateError(f"invalid JSON object: {path.name}") from error
    if not isinstance(value, dict):
        fail(f"invalid JSON object: {path.name}")
    return value


def write_object(path: pathlib.Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def exact_keys(value: Any, keys: set[str]) -> bool:
    return isinstance(value, dict) and set(value) == keys


def no_code_fields(manifest: dict[str, Any], allow_optional: bool) -> bool:
    forbidden = {"scripts", "dependencies", "devDependencies", "peerDependencies"}
    if not allow_optional:
        forbidden.add("optionalDependencies")
    return not forbidden.intersection(manifest) and not LIFECYCLE_NAMES.intersection(manifest)


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
    if name == TARGETS["linux-x64-gnu"]["name"]:
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
        if manifest.get("optionalDependencies") != expected:
            fail("launcher optional dependencies are not exact and lockstep")


def make_public_manifest(source: pathlib.Path, destination: pathlib.Path) -> None:
    manifest = load_object(source)
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


def read_tarball(path: pathlib.Path) -> tuple[dict[str, bytes], dict[str, int]]:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise CandidateError("candidate tarball is missing") from error
    if not 1 <= size <= MAX_TARBALL_BYTES:
        fail("candidate tarball is outside the 32 MiB bound")
    files: dict[str, bytes] = {}
    modes: dict[str, int] = {}
    try:
        with tarfile.open(path, "r:gz") as archive:
            members = archive.getmembers()
            for member in members:
                if not member.isfile():
                    fail("candidate tarballs may contain only regular files")
                if member.name in files:
                    fail("candidate tarball contains duplicate paths")
                extracted = archive.extractfile(member)
                if extracted is None:
                    fail("candidate tarball member is unreadable")
                files[member.name] = extracted.read()
                modes[member.name] = stat.S_IMODE(member.mode)
    except (OSError, tarfile.TarError) as error:
        raise CandidateError("candidate tarball is malformed") from error
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
    if manifest.get("keywords") != ["career", "resume", "job-search", "matching", "cli"]:
        fail("candidate launcher keywords metadata mismatch")
    expected_optional = {target["name"]: VERSION for target in TARGETS.values()}
    if manifest.get("optionalDependencies") != expected_optional:
        fail("candidate launcher optional dependencies are not exact and lockstep")
    if manifest.get("bin") != {"career": "bin/career.js"}:
        fail("candidate launcher bin surface mismatch")
    if manifest.get("engines") != {"node": ">=22"}:
        fail("candidate launcher Node engine mismatch")
    if manifest.get("files") != [
        "bin/career.js",
        "README.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "THIRD_PARTY_NOTICES.md",
    ]:
        fail("candidate launcher files metadata mismatch")
    if manifest.get("career_launcher") != {
        "schema_version": "career.npm_launcher.v1",
        "executable": "career",
        "platform_packages": [
            "@revazi/career-darwin-arm64",
            "@revazi/career-linux-x64-gnu",
        ],
    }:
        fail("candidate launcher metadata mismatch")


def validate_native_manifest_metadata(manifest: dict[str, Any], target: dict[str, Any]) -> None:
    if manifest.get("os") != [target["node_platform"]] or manifest.get("cpu") != [target["node_arch"]]:
        fail("candidate native os/cpu metadata mismatch")
    if target["libc"] is None:
        if "libc" in manifest:
            fail("Darwin candidate must not contain libc metadata")
    elif manifest.get("libc") != ["glibc"]:
        fail("Linux candidate libc metadata mismatch")
    if manifest.get("files") != [
        "career",
        "provenance.json",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "THIRD_PARTY_NOTICES.md",
    ]:
        fail("candidate native files metadata mismatch")
    if manifest.get("exports") != {"./package.json": "./package.json"}:
        fail("candidate native exports metadata mismatch")
    platform_key = next(key for key, value in TARGETS.items() if value is target)
    expected_native = {
        "schema_version": "career.npm_native_package.v1",
        "platform_key": platform_key,
        "node_platform": target["node_platform"],
        "node_arch": target["node_arch"],
        "rust_target": target["rust_target"],
        "binary_file": "career",
        "provenance_file": "provenance.json",
        "executable_mode": "0755",
        "maximum_binary_size_bytes": MAX_BINARY_BYTES,
        "minimum_glibc_version": "2.35" if target["libc"] is not None else None,
    }
    if manifest.get("career_native") != expected_native:
        fail("candidate native package metadata mismatch")


def validate_public_manifest(manifest: dict[str, Any], expected_name: str) -> None:
    if manifest.get("name") != expected_name or manifest.get("version") != VERSION:
        fail("candidate package identity/version mismatch")
    if set(manifest) != manifest_property_set(expected_name, publication_candidate=True):
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
    target = next((value for value in TARGETS.values() if value["name"] == expected_name), None)
    if target is None:
        fail("candidate native package is not approved")
    validate_native_manifest_metadata(manifest, target)


def valid_binary_header(binary: bytes, binary_format: str) -> bool:
    if binary_format == "mach-o-64-aarch64":
        return len(binary) >= 8 and binary[:8] == bytes.fromhex("cffaedfe0c000001")
    return (
        len(binary) >= 20
        and binary[:6] == bytes([0x7F, 0x45, 0x4C, 0x46, 2, 1])
        and int.from_bytes(binary[18:20], "little") == 0x3E
    )


def validate_runner(runner: Any, target: dict[str, Any]) -> None:
    if not exact_keys(runner, {"os", "arch", "image", "libc"}):
        fail("candidate runner provenance property set mismatch")
    if runner.get("os") != target["runner_os"] or runner.get("arch") != target["runner_arch"]:
        fail("candidate runner OS/architecture mismatch")
    if not bounded_text(runner.get("image"), 128):
        fail("candidate runner image is invalid")
    libc = runner.get("libc")
    if target["libc"] is None:
        if libc is not None:
            fail("Darwin candidate must not claim a libc")
    elif libc != "glibc 2.35":
        fail("Linux candidate must record the exact reviewed glibc 2.35 build runtime")


def validate_provenance(
    provenance: dict[str, Any], manifest: dict[str, Any], target: dict[str, Any], source_sha: str
) -> None:
    if not exact_keys(
        provenance, {"schema_version", "package", "source", "build", "executable", "integrity"}
    ) or provenance.get("schema_version") != "career.npm_native_provenance.v1":
        fail("candidate native provenance schema/property mismatch")
    package = provenance.get("package")
    expected_package = {
        "name": target["name"],
        "version": VERSION,
        "platform_key": next(key for key, value in TARGETS.items() if value is target),
        "node_platform": target["node_platform"],
        "node_arch": target["node_arch"],
        "rust_target": target["rust_target"],
        "minimum_glibc_version": "2.35" if target["libc"] is not None else None,
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
        build, {"command", "profile", "locked", "rustc_version", "cargo_version", "runner"}
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
    if not exact_keys(executable, {"file_name", "binary_format", "mode", "size_bytes", "sha256"}):
        fail("candidate executable provenance property set mismatch")
    if executable.get("file_name") != "career" or executable.get("mode") != "0755":
        fail("candidate executable name/mode provenance mismatch")
    if executable.get("binary_format") != target["binary_format"]:
        fail("candidate executable format provenance mismatch")


def verify_native_tarball(path: pathlib.Path, platform_key: str, source_sha: str) -> None:
    target = TARGETS.get(platform_key)
    if target is None:
        fail("unsupported native candidate target")
    files, modes = read_tarball(path)
    expected_files = {
        "package/package.json",
        "package/career",
        "package/provenance.json",
        *(f"package/{name}" for name in LICENSE_FILES),
    }
    if set(files) != expected_files:
        fail("native candidate tarball allowlist mismatch")
    expected_modes = {name: 0o644 for name in expected_files}
    expected_modes["package/career"] = 0o755
    if modes != expected_modes:
        fail("native candidate tarball mode allowlist mismatch")
    manifest = parse_member_object(files, "package/package.json")
    validate_public_manifest(manifest, target["name"])
    provenance = parse_member_object(files, "package/provenance.json")
    validate_provenance(provenance, manifest, target, source_sha)
    binary = files["package/career"]
    executable = provenance["executable"]
    if not 1 <= len(binary) <= MAX_BINARY_BYTES or executable.get("size_bytes") != len(binary):
        fail("candidate executable size mismatch")
    if modes["package/career"] != 0o755:
        fail("candidate executable packed mode mismatch")
    if executable.get("sha256") != hashlib.sha256(binary).hexdigest():
        fail("candidate executable SHA-256 mismatch")
    if not valid_binary_header(binary, target["binary_format"]):
        fail("candidate executable target format mismatch")


def verify_launcher_tarball(path: pathlib.Path) -> None:
    files, modes = read_tarball(path)
    expected_files = {
        "package/package.json",
        "package/bin/career.js",
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
    data = path.read_bytes()
    sha512 = base64.b64encode(hashlib.sha512(data).digest()).decode("ascii")
    return {"sha256": hashlib.sha256(data).hexdigest(), "integrity": f"sha512-{sha512}"}


def package_rows(directory: pathlib.Path) -> list[dict[str, Any]]:
    values: list[dict[str, Any]] = []
    for platform_key in ("darwin-arm64", "linux-x64-gnu"):
        target = TARGETS[platform_key]
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
            "order": 30,
            "role": "user_facing_launcher",
            "name": LAUNCHER_NAME,
            "version": VERSION,
            "file": LAUNCHER_FILE,
            **file_integrity(launcher_path),
        }
    )
    return values


def expected_publication_manifest(directory: pathlib.Path, source_sha: str) -> dict[str, Any]:
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
    write_object(directory / "publication-manifest.json", expected_publication_manifest(directory, source_sha))


def verify_reviewed_package_bytes(directory: pathlib.Path, repository_root: pathlib.Path) -> None:
    launcher_files, _ = read_tarball(directory / LAUNCHER_FILE)
    reviewed = {
        "package/bin/career.js": repository_root / "npm/career/bin/career.js",
        "package/README.md": repository_root / "npm/career/README.md",
    }
    for name in LICENSE_FILES:
        reviewed[f"package/{name}"] = repository_root / name
    for member, source_path in reviewed.items():
        if launcher_files.get(member) != source_path.read_bytes():
            fail(f"launcher candidate {member} differs from reviewed source")
    if not 1 <= len(launcher_files["package/README.md"]) <= 16 * 1024:
        fail("launcher README is outside its 16 KiB publication bound")
    for platform_key in ("darwin-arm64", "linux-x64-gnu"):
        native_files, _ = read_tarball(directory / TARGETS[platform_key]["file"])
        for name in LICENSE_FILES:
            if native_files.get(f"package/{name}") != (repository_root / name).read_bytes():
                fail(f"native candidate {name} differs from reviewed source")


def verify_candidate(
    directory: pathlib.Path, source_sha: str, repository_root: Optional[pathlib.Path] = None
) -> None:
    expected_names = {
        TARGETS["darwin-arm64"]["file"],
        TARGETS["linux-x64-gnu"]["file"],
        LAUNCHER_FILE,
        "publication-manifest.json",
    }
    entries = list(directory.iterdir())
    actual = {path.name for path in entries}
    if actual != expected_names or any(path.is_symlink() or not path.is_file() for path in entries):
        fail("publication candidate directory allowlist mismatch")
    verify_native_tarball(directory / TARGETS["darwin-arm64"]["file"], "darwin-arm64", source_sha)
    verify_native_tarball(directory / TARGETS["linux-x64-gnu"]["file"], "linux-x64-gnu", source_sha)
    verify_launcher_tarball(directory / LAUNCHER_FILE)
    actual_manifest = load_object(directory / "publication-manifest.json")
    expected_manifest = expected_publication_manifest(directory, source_sha)
    if actual_manifest != expected_manifest:
        fail("publication manifest/order/integrity mismatch")
    rows = actual_manifest["packages"]
    if [row["order"] for row in rows] != [10, 20, 30]:
        fail("publication order must be Darwin native, Linux native, then launcher")
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
                args.path, args.source_sha, args.runner_os, args.runner_arch, args.runner_image
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
        print(f"npm publication candidate verification failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
