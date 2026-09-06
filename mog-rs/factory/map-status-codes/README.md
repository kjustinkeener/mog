# Map codes via a lookup table

Replace matched codes with values from a table

For each regex match, look the captured value up in a table and replace it with the mapped value; unmatched keys are left as-is. This mog maps common HTTP status codes to their reason phrase. Edit the map for any code->label table.

## Run

```
mog -m map-status-codes <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
GET /a 200
GET /b 404
GET /c 302
GET /d 500
```

Output:

```
GET /a OK
GET /b Not Found
GET /c 302
GET /d Server Error
```

## Pipeline

- `lookup_replace`: Map 3-digit codes to reason phrases

## Tags

`replace` `lookup` `table` `web`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
