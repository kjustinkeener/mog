# Transpose a CSV table

Swap the rows and columns of a CSV table

Swap the rows and columns of a CSV (or other delimited) table: the first row becomes the first column, and so on. Quote-aware; jagged rows are padded to the widest row. Set 'delimiter' for TSV or pipe-delimited input.

## Run

```
mog -m transpose-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
quarter,q1,q2,q3
revenue,120,135,150
churn,4,3,5
```

Output:

```
quarter,revenue,churn
q1,120,4
q2,135,3
q3,150,5
```

## Pipeline

- `transpose`: Rows become columns and vice versa

## Tags

`csv` `reshape` `table` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
