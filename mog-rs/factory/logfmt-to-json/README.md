# logfmt to JSON

Convert logfmt log lines to JSON (JSONL)

Convert logfmt log lines (key=value key2="value 2" flag) into JSON, one flat object per line (JSONL). A bare word with no = becomes key=true; all values are strings. Pairs with JSON to logfmt.

## Run

```
mog -m logfmt-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
level=info msg="server started" port=8080
level=error msg="db timeout" retries=3 ok=false
```

Output:

```
{"level":"info","msg":"server started","port":"8080"}
{"level":"error","msg":"db timeout","retries":"3","ok":"false"}
```

## Steps

- `logfmt_to_json`: Parse each logfmt line into a JSON object.

## Tags

`convert` `log` `json` `observability`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
