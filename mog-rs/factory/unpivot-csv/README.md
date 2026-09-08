# Unpivot a wide CSV to long form

Reshape a wide CSV into long id, key, value rows

Reshape a wide CSV into tidy long (id, key, value) rows, one row per value: keep the 'id_columns' and turn every other column into its own row, using the header name as the key. The classic 'melt' for feeding a database or a BI tool. Set 'id_columns' (1-based, comma-separated) to keep more than the first column.

## Run

```
mog -m unpivot-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
region,jan,feb,mar
East,10,12,9
West,7,8,11
```

Output:

```
region,key,value
East,jan,10
East,feb,12
East,mar,9
West,jan,7
West,feb,8
West,mar,11
```

## Steps

- `unpivot`: Wide columns to (id, key, value) rows

## Tags

`csv` `reshape` `prose` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
