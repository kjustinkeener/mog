# JSON to JSONL

Convert a JSON array to JSONL, one object per line

Expand a JSON array into JSONL: one compact JSON object (or value) per line. A non-array document becomes a single line. Pairs with JSONL to JSON.

## Run

```
mog -m json-to-jsonl <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
[{"id": 1, "name": "Ada"}, {"id": 2, "name": "Bo"}]
```

Output:

```
{"id":1,"name":"Ada"}
{"id":2,"name":"Bo"}
```

## Pipeline

- `json_to_jsonl`: Expand the JSON array into one value per line.

## Tags

`convert` `json` `jsonl`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
