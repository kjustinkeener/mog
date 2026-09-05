# Avro schema to SQL CREATE TABLE

Convert Avro schema to SQL CREATE TABLE

Convert an Avro record schema (.avsc) into a CREATE TABLE DDL statement. Maps scalar Avro types (long -> BIGINT, int -> INTEGER, double -> DOUBLE PRECISION, float -> REAL, boolean -> BOOLEAN, bytes -> BYTEA, string -> TEXT) and the record name to the table name; a nullable union (['null', T] or [T, 'null']) yields a nullable column, a plain type yields NOT NULL. Line/regex based, not an Avro parser: assumes one field object per line and does not handle nested records, arrays, maps, enums, fixed, logical types, defaults, or namespaces.

## Run

```
mog -m avro-to-sql-ddl <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "type": "record",
  "name": "users",
  "fields": [
    { "name": "id", "type": "long" },
    { "name": "name", "type": "string" },
    { "name": "email", "type": ["null", "string"] },
    { "name": "active", "type": "boolean" },
    { "name": "score", "type": "double" }
  ]
}
```

Output:

```
CREATE TABLE users (
  id BIGINT NOT NULL,
  name TEXT NOT NULL,
  email TEXT,
  active BOOLEAN NOT NULL,
  score DOUBLE PRECISION NOT NULL
);
```

## Pipeline

- `eol_lf`: Normalize line endings to LF so the per-line field regexes are reliable. Output stays LF (standard for SQL scripts).
- `replace_regex_multiline`: Nullable union field ['null', T] -> a nullable column (no NOT NULL). The scalar type is tagged @T@ for the type map below.
- `replace_regex_multiline`: Nullable union field [T, 'null'] -> a nullable column, other order.
- `replace_regex_multiline`: Plain scalar field -> a NOT NULL column.
- `replace_map`: Map the tagged Avro scalar types to SQL types (longest key wins).
- `replace_regex_multiline`: Record name -> CREATE TABLE header and open the column list (field 'name' lines are already converted, so only the record name remains).
- `remove_lines_matching`: Drop the Avro scaffolding lines (object braces, the record-type marker, and the fields-array wrapper). Converted column lines have no braces or brackets, so they survive.
- `append`: Close the statement.
- `replace_regex`: Drop the trailing comma after the last column so the DDL is valid.

## Tags

`convert` `schema` `avro` `sql`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
