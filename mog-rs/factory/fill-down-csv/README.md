# Fill down blank CSV cells

Fill down blank CSV cells from above

Fill each blank cell with the last non-blank value seen above it in the same column, the classic fix for a spreadsheet export where merged cells left gaps. Blank lines pass through. Quote-aware; option delimiter (default ,) on the step for other delimiters.

## Run

```
mog -m fill-down-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
2024,Q1,10
,,20
,Q2,30
2025,Q3,40
```

Output:

```
2024,Q1,10
2024,Q1,20
2024,Q2,30
2025,Q3,40
```

## Steps

- `fill_down`: Carry the last non-blank value down each column

## Tags

`csv` `fill` `table` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
