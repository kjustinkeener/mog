# JSON to query string

Convert a flat JSON object to a URL query string

Convert a flat JSON object to a URL query string (a=1&b=2), url-encoded. An array value repeats its key; null values are skipped. The read counterpart to query string to JSON.

## Run

```
mog -m json-to-querystring <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"q": "hello world", "page": 2, "tag": ["a", "b"]}
```

Output:

```
q=hello%20world&page=2&tag=a&tag=b
```

## Pipeline

- `json_to_querystring`: Render the JSON object as a query string.

## Tags

`convert` `json` `url` `web`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
