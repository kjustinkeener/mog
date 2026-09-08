# De-duplicate multi-line records

Drop duplicate blank-line-separated blocks, keep first

Remove duplicate multi-line records (blocks separated by a blank line), keeping the first occurrence of each. Compares whole blocks by default; here it keys on the first line so records with the same header collapse.

## Run

```
mog -m dedupe-records <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id: 1
Ada

id: 2
Bo

id: 1
Ada again
```

Output:

```
id: 1
Ada

id: 2
Bo
```

## Steps

- `dedupe_blocks`: Drop duplicate blocks, key on first line

## Tags

`line` `dedupe` `fragment` `records`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
