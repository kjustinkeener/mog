# Convert Oracle column types to Postgres

Map a list of Oracle data types to Postgres types

Translate a list of SQL data types from Oracle to PostgreSQL, one type per line: VARCHAR2(n)->VARCHAR(n), NUMBER(p,s)->DECIMAL(p,s), NUMBER (no scale)->its Postgres equivalent, DATE->TIMESTAMP(0) (Oracle DATE carries a time component), CLOB->TEXT, BLOB->BYTEA. Blank lines pass through so a column list's spacing is kept. Parse-based (polyglot-sql), so precision and scale are preserved exactly. Converts a column-type list only, not full CREATE TABLE statements.

## Run

```
mog -m oracle-types-to-postgres <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
VARCHAR2(100)
NUMBER(38,0)
NUMBER(10,2)
DATE
CLOB
BLOB
CHAR(1)
```

Output:

```
VARCHAR(100)
DECIMAL(38, 0)
DECIMAL(10, 2)
TIMESTAMP(0)
TEXT
BYTEA
CHAR(1)
```

## Steps

- `sql_datatype_convert`: Convert each line's Oracle type to its Postgres equivalent

## Tags

`sql` `oracle` `postgres` `migrate` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
