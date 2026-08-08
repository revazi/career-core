#!/usr/bin/env python3
"""Independently inspect installed Career Core managed-adapter discovery output."""

import argparse
import json
import pathlib
import re
import subprocess
import tempfile

DRAFT_2020_12 = "https://json-schema.org/draft/2020-12/schema"
MAX_CAPTURE_BYTES = 64 * 1024 * 1024
EXPECTED_OUTPUT_BOUND = 33_554_432
EXPECTED_BOOTSTRAP_OPERATIONS = {
    "core.operations",
    "schema.list",
    "schema.export",
    "schema.bundle",
}


def fail(message):
    raise SystemExit(f"managed-adapter validation failed: {message}")


def run_json(career, working_directory, arguments):
    completed = subprocess.run(
        [str(career), *arguments, "--format", "json-compact"],
        cwd=working_directory,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    label = " ".join(arguments)
    if completed.returncode != 0:
        fail(f"{label} exited nonzero")
    if completed.stderr:
        fail(f"{label} wrote unexpected stderr")
    data = completed.stdout
    if not data or len(data) > MAX_CAPTURE_BYTES:
        fail(f"{label} output is empty or exceeds the independent capture bound")
    if not data.endswith(b"\n") or data.count(b"\n") != 1:
        fail(f"{label} compact JSON framing is invalid")
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError):
        fail(f"{label} output is not valid JSON")
    if not isinstance(value, dict):
        fail(f"{label} must emit a JSON object")
    return value, data


def resolve_pointer(document, reference):
    if not reference.startswith("#"):
        fail("schema bundle contains a non-local $ref")
    fragment = reference[1:]
    if fragment == "":
        return document
    if not fragment.startswith("/"):
        fail("schema bundle contains a non-JSON-Pointer fragment")
    current = document
    for encoded_token in fragment[1:].split("/"):
        token = encoded_token.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict) and token in current:
            current = current[token]
        elif isinstance(current, list) and token.isdigit() and int(token) < len(current):
            current = current[int(token)]
        else:
            fail("schema bundle contains an unresolved local $ref")
    return current


def inspect_references(value, root):
    if isinstance(value, dict):
        reference = value.get("$ref")
        if reference is not None:
            if not isinstance(reference, str):
                fail("schema bundle contains a non-string $ref")
            resolve_pointer(root, reference)
        for child in value.values():
            inspect_references(child, root)
    elif isinstance(value, list):
        for child in value:
            inspect_references(child, root)


def require_exact_keys(value, expected, label):
    if set(value) != set(expected):
        fail(f"{label} has an unexpected property set")


def inspect_operation_catalog(catalog, capabilities, schema_ids, version_output):
    require_exact_keys(catalog, {"schema_version", "core_version", "operations"}, "operation catalog")
    if catalog["schema_version"] != "career.operation_catalog.v1":
        fail("unexpected operation catalog schema version")
    if not isinstance(catalog["core_version"], str) or not 1 <= len(catalog["core_version"]) <= 64:
        fail("operation catalog core version is invalid")
    if version_output != f"career {catalog['core_version']}":
        fail("operation catalog core version does not match the executable")

    operations = catalog["operations"]
    if not isinstance(operations, list) or not 1 <= len(operations) <= 64:
        fail("operation catalog count is outside bounds")

    descriptor_keys = {
        "operation_id",
        "capability_id",
        "availability",
        "cli_path",
        "input_transport",
        "input_schema_id",
        "output_schema_id",
        "maximum_input_bytes",
        "maximum_successful_machine_output_bytes",
    }
    operation_ids = []
    mapped_capability_ids = []
    output_bounds = set()
    bootstrap_ids = set()
    operation_pattern = re.compile(r"^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$")
    path_pattern = re.compile(r"^[a-z][a-z0-9-]*$")

    for index, operation in enumerate(operations):
        if not isinstance(operation, dict):
            fail(f"operation descriptor {index} is not an object")
        require_exact_keys(operation, descriptor_keys, f"operation descriptor {index}")
        operation_id = operation["operation_id"]
        if not isinstance(operation_id, str) or not operation_pattern.fullmatch(operation_id):
            fail(f"operation descriptor {index} has an invalid operation ID")
        operation_ids.append(operation_id)
        if operation["availability"] != "available":
            fail(f"operation {operation_id} is not callable")
        cli_path = operation["cli_path"]
        if (
            not isinstance(cli_path, list)
            or not 1 <= len(cli_path) <= 3
            or any(not isinstance(segment, str) or not path_pattern.fullmatch(segment) for segment in cli_path)
        ):
            fail(f"operation {operation_id} has an invalid CLI path")

        capability_id = operation["capability_id"]
        if capability_id is None:
            bootstrap_ids.add(operation_id)
        elif isinstance(capability_id, str) and operation_pattern.fullmatch(capability_id):
            mapped_capability_ids.append(capability_id)
            if operation_id != capability_id:
                fail(f"capability-backed operation {operation_id} changed its stable ID")
        else:
            fail(f"operation {operation_id} has an invalid capability mapping")

        transport = operation["input_transport"]
        input_schema_id = operation["input_schema_id"]
        maximum_input_bytes = operation["maximum_input_bytes"]
        if transport == "json_file_or_stdin":
            if input_schema_id not in schema_ids:
                fail(f"operation {operation_id} has an unknown input schema")
            if maximum_input_bytes not in (262_144, 1_048_576):
                fail(f"operation {operation_id} has an invalid maximum input size")
        elif transport in ("none", "cli_arguments"):
            if input_schema_id is not None or maximum_input_bytes is not None:
                fail(f"non-document operation {operation_id} declares a JSON input")
        else:
            fail(f"operation {operation_id} has an unknown input transport")

        output_schema_id = operation["output_schema_id"]
        if output_schema_id not in schema_ids and output_schema_id != DRAFT_2020_12:
            fail(f"operation {operation_id} has an unknown output schema")
        output_bound = operation["maximum_successful_machine_output_bytes"]
        if output_bound != EXPECTED_OUTPUT_BOUND:
            fail(f"operation {operation_id} changed the v1 successful output bound")
        output_bounds.add(output_bound)

    if len(set(operation_ids)) != len(operation_ids):
        fail("operation IDs are not unique")
    if bootstrap_ids != EXPECTED_BOOTSTRAP_OPERATIONS:
        fail("bootstrap operation catalog coverage is incomplete")
    available_capability_ids = [
        capability["id"]
        for capability in capabilities.get("capabilities", [])
        if isinstance(capability, dict) and capability.get("status") == "available"
    ]
    if mapped_capability_ids != available_capability_ids:
        fail("available capabilities do not map exactly once in stable order")
    if output_bounds != {EXPECTED_OUTPUT_BOUND}:
        fail("operation output bounds are inconsistent")
    return {operation["operation_id"]: operation for operation in operations}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--career", required=True, type=pathlib.Path)
    parser.add_argument("--output-dir", required=True, type=pathlib.Path)
    arguments = parser.parse_args()

    career = arguments.career.expanduser().resolve(strict=True)
    if not career.is_file():
        fail("career path is not a file")
    output_dir = arguments.output_dir.expanduser().resolve(strict=False)
    output_dir.mkdir(parents=True, exist_ok=True)
    bundle_dir = output_dir / "bundles"
    discovery_dir = output_dir / "discovery"
    bundle_dir.mkdir(exist_ok=True)
    discovery_dir.mkdir(exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="career-managed-adapter-validation.") as temporary:
        working_directory = pathlib.Path(temporary)
        capabilities, capabilities_bytes = run_json(career, working_directory, ["capabilities"])
        catalog, catalog_bytes = run_json(career, working_directory, ["operations"])
        schema_catalog, schema_catalog_bytes = run_json(career, working_directory, ["schema", "list"])

        if capabilities.get("schema_version") != "career.capabilities.v1":
            fail("unexpected capabilities schema version")
        if schema_catalog.get("schema_version") != "career.schema_catalog.v1":
            fail("unexpected schema catalog version")
        schema_entries = schema_catalog.get("schemas")
        if not isinstance(schema_entries, list) or not 1 <= len(schema_entries) <= 128:
            fail("schema catalog count is outside bounds")
        schema_ids = {
            entry.get("id")
            for entry in schema_entries
            if isinstance(entry, dict) and isinstance(entry.get("id"), str)
        }
        if len(schema_ids) != len(schema_entries) or "career.operation_catalog.v1" not in schema_ids:
            fail("schema catalog IDs are invalid or incomplete")

        version = subprocess.run(
            [str(career), "--version"],
            cwd=working_directory,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            text=True,
        )
        if version.returncode != 0 or version.stderr or not version.stdout.endswith("\n"):
            fail("career --version failed")
        operation_by_id = inspect_operation_catalog(
            catalog,
            capabilities,
            schema_ids,
            version.stdout.rstrip("\n"),
        )
        if len(capabilities_bytes) > operation_by_id["core.capabilities"]["maximum_successful_machine_output_bytes"]:
            fail("capabilities exceed their declared output bound")
        if len(catalog_bytes) > operation_by_id["core.operations"]["maximum_successful_machine_output_bytes"]:
            fail("operation catalog exceeds its declared output bound")
        if len(schema_catalog_bytes) > operation_by_id["schema.list"]["maximum_successful_machine_output_bytes"]:
            fail("schema catalog exceeds its declared output bound")

        (discovery_dir / "capabilities.json").write_bytes(capabilities_bytes)
        (discovery_dir / "operations.json").write_bytes(catalog_bytes)
        (discovery_dir / "schema-catalog.json").write_bytes(schema_catalog_bytes)

        bundle_bound = operation_by_id["schema.bundle"]["maximum_successful_machine_output_bytes"]
        for entry in schema_entries:
            require_exact_keys(entry, {"id", "file_name", "title"}, "schema catalog entry")
            schema_id = entry["id"]
            file_name = entry["file_name"]
            if not isinstance(file_name, str) or not re.fullmatch(r"[a-z0-9-]+-v[1-9][0-9]*\.schema\.json", file_name):
                fail("schema catalog file name is invalid")
            bundle, bundle_bytes = run_json(
                career,
                working_directory,
                ["schema", "bundle", "--id", schema_id],
            )
            if len(bundle_bytes) > bundle_bound:
                fail(f"schema bundle exceeds its declared output bound: {schema_id}")
            if bundle.get("$schema") != DRAFT_2020_12 or bundle.get("type") != "object":
                fail(f"schema bundle is not a Draft 2020-12 object schema: {schema_id}")
            inspect_references(bundle, bundle)
            (bundle_dir / f"{file_name}.bundle.json").write_bytes(bundle_bytes)

    print(f"Managed-adapter discovery validation passed for {career}")


if __name__ == "__main__":
    main()
