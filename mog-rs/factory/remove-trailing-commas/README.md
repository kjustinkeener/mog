# Remove trailing commas (JSON5 to JSON)

Remove trailing commas to make JSON5 valid JSON

Delete trailing commas that appear right before a closing } or ] (the JSON5 / JavaScript convenience that strict JSON rejects), so the text parses as valid JSON. A comma+bracket inside a string literal would also be changed, so review data containing such strings.

## Run

```
mog -m remove-trailing-commas <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "name": "demo",
  "items": [
    1,
    2,
    3,
  ],
  "nested": {
    "a": true,
  },
}
```

Output:

```
{
  "name": "demo",
  "items": [
    1,
    2,
    3
  ],
  "nested": {
    "a": true
  }
}
```

## Steps

- `replace_regex`: Drop a comma before a closing bracket

## Tags

`json` `javascript` `cleanup`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
