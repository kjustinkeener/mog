# List JSON object keys

List the top-level keys of JSON objects

Emit the top-level keys of each JSON object (one JSONL object per line), comma-joined. Handy for eyeballing the schema/shape of a JSON or JSONL file. For a single JSON object, put it on one line.

## Run

```
mog -m json-keys <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"id": 1, "name": "Ada", "role": "Lead"}
{"id": 2, "name": "Bob", "active": true}
```

Output:

```
id,name,role
id,name,active
```

## Pipeline

- `json_keys`: Emit each object's keys, comma-joined

## Tags

`json` `jsonl` `schema` `scan`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
