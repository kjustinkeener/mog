# Minify JSON

Minify JSON to compact one-line output

Parse the whole input as one JSON value and re-serialize it compactly, with no insignificant whitespace. Valid by construction (it round-trips through a real JSON parser, so it also validates the input). Key order is preserved. The inverse of format-json.

## Run

```
mog -m minify-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

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

Output:

```
{"name":"demo","tags":["a","b"],"nested":{"x":1,"y":2}}
```

## Pipeline

- `json_minify`: Re-serialize compactly

## Tags

`json` `minify` `scan`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
