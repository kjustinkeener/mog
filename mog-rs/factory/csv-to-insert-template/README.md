# CSV to SQL INSERTs (custom template)

Convert CSV rows to SQL INSERTs via a template

Turn each row of a CSV with a header into a SQL INSERT statement, using a template with ${column} placeholders (by header name). Unlike the fixed csv_to_sql converter, the template is yours: change the table, column order, quoting, or ON CONFLICT clause freely, and reuse the same pattern to emit config lines, HTML, YAML, or form text instead. The header row supplies the ${name} placeholders and is not itself emitted; $$ is a literal dollar sign.

## Run

```
mog -m csv-to-insert-template <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name,email
1,Ada,ada@example.com
2,Bob,bob@example.com
```

Output:

```
INSERT INTO users (id, name, email) VALUES (1, 'Ada', 'ada@example.com');
INSERT INTO users (id, name, email) VALUES (2, 'Bob', 'bob@example.com');
```

## Pipeline

- `row_to_template`: One INSERT per row, filling ${id}/${name}/${email} from the header

## Tags

`data` `codegen` `csv` `sql`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
