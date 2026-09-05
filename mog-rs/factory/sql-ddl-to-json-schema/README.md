# SQL CREATE TABLE to JSON Schema

Convert SQL CREATE TABLE to JSON Schema

Convert a CREATE TABLE statement into a draft-07 JSON Schema object (the reverse of json-schema-to-sql-ddl). Maps common SQL types to JSON types (INT/BIGINT/SERIAL -> integer; REAL/FLOAT/DOUBLE PRECISION/NUMERIC/DECIMAL -> number; BOOL/BOOLEAN -> boolean; TEXT/VARCHAR/CHAR/UUID/DATE/TIMESTAMP/JSON -> string) and turns column nullability into the type: NOT NULL -> a plain type, a nullable column -> a ['type', 'null'] union. Not a SQL parser: it assumes one column per line, and does not handle table/column constraints on their own line (PRIMARY KEY, FOREIGN KEY, UNIQUE, CHECK), DEFAULT clauses, multi-statement scripts, or bracket/backtick quoting.

## Run

```
mog -m sql-ddl-to-json-schema <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE users (
  id BIGINT NOT NULL,
  name VARCHAR(255) NOT NULL,
  email TEXT,
  active BOOLEAN NOT NULL,
  score DOUBLE PRECISION NOT NULL
);
```

Output:

```
{
  "type": "object",
  "title": "users",
  "properties": {
    "id": { "type": "integer" },
    "name": { "type": "string" },
    "email": { "type": ["string", "null"] },
    "active": { "type": "boolean" },
    "score": { "type": "number" }
  }
}
```

## Pipeline

- `eol_lf`: Normalize line endings to LF so the per-line column regexes are reliable.
- `replace_regex`: Parametrized character types VARCHAR(n)/CHAR(n)/NVARCHAR(n) -> tagged @string@.
- `replace_regex`: Parametrized numeric types NUMERIC(p[,s])/DECIMAL(p[,s]) -> tagged @number@.
- `replace`: Two-word DOUBLE PRECISION -> tagged @number@ (before the bare-word map).
- `replace_map`: Bare SQL type keywords -> tagged JSON types (whole-word, case-insensitive; longest key wins).
- `replace_regex_multiline`: NOT NULL column -> a JSON Schema property with a single type. Runs before the nullable rule.
- `replace_regex_multiline`: Nullable column -> a JSON Schema property with a ['type', 'null'] union.
- `replace_regex_multiline`: CREATE TABLE <name> ( -> the JSON Schema object header and open the properties object.
- `replace_regex_multiline`: The closing ); -> close the properties object and the schema object.
- `replace_regex`: Drop the trailing comma after the last property so the output is valid JSON.

## Tags

`convert` `schema` `sql` `json`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
