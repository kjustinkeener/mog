# JSON Schema to SQL CREATE TABLE

Convert JSON Schema to SQL CREATE TABLE

Convert a draft-07 JSON Schema object into a CREATE TABLE DDL statement. Maps JSON types (string -> TEXT, integer -> BIGINT, number -> DOUBLE PRECISION, boolean -> BOOLEAN) and the schema 'title' to the table name. Nullability comes from the type: a plain type -> NOT NULL, a ['type', 'null'] union -> nullable. Line/regex based, not a JSON Schema parser: it assumes one property per line ('name': { 'type': ... }). It does not correlate a separate top-level 'required' array to columns, and does not handle nested objects, arrays, $ref, enum, format, allOf/anyOf/oneOf, or constraints.

## Run

```
mog -m json-schema-to-sql-ddl <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "title": "users",
  "type": "object",
  "properties": {
    "id": { "type": "integer" },
    "name": { "type": "string" },
    "email": { "type": ["string", "null"] },
    "active": { "type": "boolean" },
    "score": { "type": "number" }
  }
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

## Steps

- `eol_lf`: Normalize line endings to LF so the per-line property regexes are reliable. Output stays LF (standard for SQL scripts).
- `replace_regex_multiline`: Nullable union property (['type', 'null']) -> a nullable column (no NOT NULL). The JSON type is tagged @T@ for the type map below.
- `replace_regex_multiline`: Plain-typed property -> a NOT NULL column.
- `replace_map`: Map the tagged JSON Schema scalar types to SQL types.
- `replace_regex_multiline`: Schema title -> CREATE TABLE header and open the column list.
- `remove_lines_matching`: Drop the JSON scaffolding lines (object braces, the 'type': 'object' marker, the 'properties' wrapper, and any top-level 'required' array). Converted column lines have no braces, so they survive.
- `append`: Close the statement.
- `replace_regex`: Drop the trailing comma after the last column so the DDL is valid.

## Tags

`convert` `schema` `json` `sql`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
