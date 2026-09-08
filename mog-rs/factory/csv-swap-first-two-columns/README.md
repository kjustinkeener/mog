# Swap the first two CSV columns

Swap the first two CSV columns

Reorder a CSV so column 2 comes before column 1, keeping columns 3+ in place (quote-aware). Handy for fixing a last-name,first-name vs first-name,last-name mismatch. Change the fields option on the step for a different reorder.

## Run

```
mog -m csv-swap-first-two-columns <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
last,first,city
Smith,Jane,London
Doe,John,Paris
```

Output:

```
first,last,city
Jane,Smith,London
John,Doe,Paris
```

## Steps

- `cut_fields`: Emit columns in the order 2, 1, 3

## Tags

`csv` `column` `sort` `replace` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
