# Format (pretty-print) JSON

Pretty-print and validate JSON

Parse the whole input as one JSON value and re-serialize it with 2-space indentation. Because it round-trips through a real JSON parser, it also validates the input (invalid JSON is an error) and normalizes key quoting and spacing. Key order is preserved.

## Run

```
mog -m format-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"name":"demo","tags":["a","b"],"nested":{"x":1,"y":2}}
```

Output:

```
{
  "name": "demo",
  "tags": [
    "a",
    "b"
  ],
  "nested": {
    "x": 1,
    "y": 2
  }
}
```

## Pipeline

- `json_pretty`: Re-serialize with 2-space indentation

## Tags

`json` `format` `indent` `scan`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
