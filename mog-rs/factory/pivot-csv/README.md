# Pivot a long CSV to wide form

Pivot a long CSV into a wide table

Reshape a long (id, key, value) CSV into a wide table: distinct values of the key column become columns, the value column fills the cells, and rows group by the id columns. Set 'key_column' and 'value_column' (1-based). A guardrail refuses more than 'max_columns' distinct keys (default 1000); set max_columns to 0 to lift the cap. 'aggregate' (first/last/error) resolves a repeated group+key.

## Run

```
mog -m pivot-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
day,metric,value
Mon,sales,100
Mon,visits,40
Tue,sales,120
Tue,visits,55
```

Output:

```
day,sales,visits
Mon,100,40
Tue,120,55
```

## Pipeline

- `pivot`: Long (id, key, value) rows to a wide table

## Tags

`csv` `reshape` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
