# CSV to SQL UPDATE statements

Convert CSV rows to SQL UPDATE statements

Generate one SQL UPDATE per CSV row (header row required) using a per-row template: the demo sets name by id on a users table. Edit the template on the step for your table, columns, and WHERE key. Values are placed as written -- SQL-escape them first (escape-sql) if they can contain quotes.

## Run

```
mog -m csv-to-sql-update <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name
1,Ada
2,Bob
```

Output:

```
UPDATE users SET name = 'Ada' WHERE id = 1;
UPDATE users SET name = 'Bob' WHERE id = 2;
```

## Pipeline

- `row_to_template`: One UPDATE per row

## Tags

`csv` `sql` `replace` `codegen` `data`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
