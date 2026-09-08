# JSON to logfmt

Convert JSONL to logfmt log lines

Convert JSONL (one flat JSON object per line) into logfmt log lines (key=value ...), quoting any value that holds a space, = or quote. Null values are skipped. The read counterpart to logfmt to JSON.

## Run

```
mog -m json-to-logfmt <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"level":"info","msg":"server started","port":8080}
{"level":"error","msg":"db timeout","retries":3,"ok":false}
```

Output:

```
level=info msg="server started" port=8080
level=error msg="db timeout" retries=3 ok=false
```

## Steps

- `json_to_logfmt`: Render each JSON object as a logfmt line.

## Tags

`convert` `json` `log` `observability`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
