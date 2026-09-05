# Sanitize stack trace

Normalize stack trace addresses and line numbers for clean diffs

Normalize the volatile parts of a stack trace so two traces diff cleanly: hex addresses -> 0xADDR, and file :line:col -> :LINE:COL.

## Run

```
mog -m stacktrace-sanitize <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Error: boom
    at foo (/app/src/index.js:12:5)
    at bar (/app/src/index.js:34:9)
    handle at 0xdeadbeef
```

Output:

```
Error: boom
    at foo (/app/src/index.js:LINE:COL)
    at bar (/app/src/index.js:LINE:COL)
    handle at 0xADDR
```

## Pipeline

- `replace_regex`: Redact hex memory addresses.
- `replace_regex`: Normalize :line:col positions.

## Tags

`cleanup` `log` `diff` `redact` `observability`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
