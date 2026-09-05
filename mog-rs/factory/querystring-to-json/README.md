# Query string to JSON

Convert a URL query string to JSON

Parse a URL query string (a=1&b=2) into a url-decoded JSON object; a repeated key becomes an array. The inverse of JSON to query string.

## Run

```
mog -m querystring-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
q=hello+world&page=2&tag=a&tag=b
```

Output:

```
{
  "q": "hello world",
  "page": "2",
  "tag": [
    "a",
    "b"
  ]
}
```

## Pipeline

- `querystring_to_json`: Parse the query string into a JSON object.

## Tags

`convert` `url` `json` `web`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
