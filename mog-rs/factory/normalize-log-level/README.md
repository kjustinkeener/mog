# Normalize log level names

Normalize log level names to canonical forms

Collapse the many spellings of a log level to one canonical form (WARNING -> WARN, ERR -> ERROR, INFORMATION -> INFO, CRITICAL/FATAL -> CRIT). Case-insensitive, whole-word. Useful before diffing or grouping logs from tools that disagree on level names.

## Run

```
mog -m normalize-log-level <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
[WARNING] disk space low
[ERR] connection refused
[info] service started
[Critical] out of memory
```

Output:

```
[WARN] disk space low
[ERROR] connection refused
[info] service started
[CRIT] out of memory
```

## Pipeline

- `replace_map`: Map level variants to a canonical form

## Tags

`log` `indent` `normalize` `observability` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
