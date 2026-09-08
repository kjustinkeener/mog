# JSONL to JSON

Convert JSONL to a single JSON array

Collect JSONL (one JSON value per line) into a single pretty JSON array. The read counterpart to JSON to JSONL.

## Run

```
mog -m jsonl-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"id": 1, "name": "Ada"}
{"id": 2, "name": "Bo"}
```

Output:

```
[
  {
    "id": 1,
    "name": "Ada"
  },
  {
    "id": 2,
    "name": "Bo"
  }
]
```

## Steps

- `jsonl_to_json`: Collect the lines into one JSON array.

## Tags

`convert` `jsonl` `json`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
