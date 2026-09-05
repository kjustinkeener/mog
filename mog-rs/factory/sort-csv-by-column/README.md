# Sort CSV rows by a column

Sort CSV rows by a chosen column

Sort lines by the Nth delimited field instead of by the whole line. Set the column with 'key_field' (1-based) and the delimiter with 'key_delimiter'; 'numeric' compares the key as a number. Here it sorts by the 2nd comma-separated column numerically. It does not treat a header row specially, so strip and re-add a header around this step if your data has one.

## Run

```
mog -m sort-csv-by-column <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
alice,30,nyc
bob,25,sf
carol,40,la
dave,2,chi
```

Output:

```
dave,2,chi
bob,25,sf
alice,30,nyc
carol,40,la
```

## Pipeline

- `sort_lines`

## Tags

`sort` `csv` `column` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
