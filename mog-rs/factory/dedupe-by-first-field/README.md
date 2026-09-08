# Dedupe rows by first column

Remove duplicate rows by first column key

Remove duplicate lines by the first comma-separated field, keeping the first occurrence of each key. Works on non-adjacent duplicates, unlike a consecutive-only dedupe. Set the key with 'key_field' (1-based) and 'key_delimiter', or use 'key_regex' instead. Does not treat a header row specially.

## Run

```
mog -m dedupe-by-first-field <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
acme,100
beta,200
acme,999
gamma,300
beta,404
```

Output:

```
acme,100
beta,200
gamma,300
```

## Steps

- `dedupe_by`

## Tags

`dedupe` `csv` `column` `json` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
