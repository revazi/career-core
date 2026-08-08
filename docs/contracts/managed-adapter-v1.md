# Managed-adapter discovery contracts v1

Phase 8 adds CLI metadata needed by a reviewed managed adapter. It does not add adapter state, handles, projections, persistence, provider/model behavior, networking, UI, or acceptance of assisted text by authoritative operations.

## Operation catalog

`career operations` emits `career.operation_catalog.v1`. `core_version` identifies the exact runtime contract source for non-sensitive adapter caches. Descriptor order is stable.

The catalog intentionally includes both:

- one descriptor for each available `career.capabilities.v1` entry, where `operation_id == capability_id`
- four bootstrap descriptors—`core.operations`, `schema.list`, `schema.export`, and `schema.bundle`—where `capability_id` is null

A bootstrap descriptor has no capability mapping because `career.capabilities.v1` is closed and byte-stable. JSON document operations use `json_file_or_stdin` and identify their exact input schema and either the 262,144-byte ordinary envelope or 1,048,576-byte composite envelope. Schema ID arguments use `cli_arguments` and therefore have no JSON input schema or document byte ceiling.

## Self-contained schema bundles

`career schema bundle --id <schema-id>` starts only from reviewed schemas embedded in the executable. It recursively finds exact sibling-file `$ref` targets in the same embedded catalog. The requested root remains the schema resource and retains its `$schema` and `$id`.

Dependencies are embedded beneath:

```text
#/$defs/careerSchemaBundle/$defs/<embedded-file-name>
```

Dependency `$schema` and `$id` keys are removed so local fragments resolve in the retained root resource. Root-local references remain root-local; dependency-local and sibling references become absolute root-local JSON Pointers. The command rejects unknown files, remote/URI references, unsupported fragments, malformed embedded documents, and reserved-definition collisions. It uses no source checkout, filesystem schema lookup, or network.

All 26 emitted bundles are checked for deterministic bytes, Draft 2020-12 metaschema validity, and locally resolvable `$ref` values. Representative operation inputs and outputs are independently validated against emitted bundles.

## Successful machine-output ceiling

Every successful JSON mode has one v1 ceiling of **33,554,432 bytes (32 MiB) including the final newline**. Serialization completes into an in-memory byte vector before any stdout write, so an oversized result cannot be partially consumed or silently truncated.

The ceiling conservatively covers all algorithm-produced values allowed by the existing core limits, including Serde JSON's worst accepted scalar escaping. A control scalar may occupy six ASCII bytes as `\\u00XX`; quotes/backslashes need two bytes and unescaped Unicode needs at most four, so the derivation charges **six output bytes per bounded character**.

The largest structural surface is `career.job_match.v1`. Its six categories contain at most 100 items each. Charging each item two maximum 2,000-character values and two maximum 160-character source excerpts gives 2,592,000 bounded characters, or 15,552,000 worst-case escaped bytes. Charging the same six-byte rate for every other maximum string surface—six category explanations, 72 metric values, ten strength/gap item-and-reason pairs with source excerpts, the recommendation reason, 30 blockers, and five warnings—keeps bounded string content below 16.3 MB. Fixed keys, enum strings, numbers, booleans, punctuation, array/object framing, canonical pretty-print whitespace, and the final newline remain well below the additional 15 MiB margin to 32 MiB. Other results are smaller because resume/job source is limited to 50,000 characters, normalization collections are explicitly capped, and Phase 7 proposals are aggregate-bounded. The complete set of embedded schemas is also far below the ceiling.

The CLI unit boundary admits a serialized document exactly at its configured ceiling and rejects a document one byte over before writing. A production ceiling violation returns exit `6` and the existing `career.error.v1` `output_write_failed` code; v1 error-schema bytes are not changed.

## Compatibility

This is an additive CLI/public-schema change:

- `career.capabilities.v1` bytes and schema are unchanged
- successful existing capability-backed operation bytes are unchanged
- existing unbundled schema-export bytes are unchanged
- `career.schema_catalog.v1` gains the new public operation-catalog schema entry
- Swift facade functions and generated Swift bytes are unchanged

Generic raw agents remain capability/schema discovery-first. A separately reviewed managed adapter may discover the operation catalog and required bundles once per verified `core_version`, cache only this non-sensitive metadata, and keep those bootstrap calls out of model-visible normal workflow.
