# CSV to SQL INSERT

Convert CSV to SQL INSERT statements

Convert CSV (with a header row) into INSERT INTO <table> (...) VALUES (...); statements. Numbers are bare, empty cells become NULL, everything else is a single-quoted escaped string. Set the table with -D table=... (default my_table).

## Run

```
mog -m csv-to-sql <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name,note
1,Ada,
2,"Bo, Jr",VIP
```

Output:

```
INSERT INTO my_table (id, name, note) VALUES (1, 'Ada', NULL);
INSERT INTO my_table (id, name, note) VALUES (2, 'Bo, Jr', 'VIP');
```

## Steps

- `csv_to_sql`: Emit an INSERT per CSV row.

## Tags

`convert` `csv` `sql` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
