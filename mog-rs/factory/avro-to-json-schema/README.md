# Avro schema to JSON Schema

Convert Avro schema to draft-07 JSON Schema

Convert an Avro record schema (.avsc) into a draft-07 JSON Schema object. Maps scalar Avro types (long/int -> integer, float/double -> number, boolean -> boolean, bytes/string -> string) and turns a nullable union (['null', T] or [T, 'null']) into a ['type', 'null'] union. Line/regex based, not an Avro parser: assumes one field object per line and does not handle nested records, arrays, maps, enums, fixed, logical types, defaults, doc, aliases, or namespaces.

## Run

```
mog -m avro-to-json-schema <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "type": "record",
  "name": "User",
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
{
  "type": "object",
  "title": "User",
  "properties": {
    "id": { "type": "integer" },
    "name": { "type": "string" },
    "email": { "type": ["string", "null"] },
    "active": { "type": "boolean" },
    "score": { "type": "number" }
  }
}
```

## Steps

- `eol_lf`: Normalize line endings to LF so the per-line field regexes are reliable.
- `replace_regex_multiline`: Nullable union field ['null', T]: emit a JSON Schema property with a [type, 'null'] union. The scalar type is tagged @T@ for the type map below.
- `replace_regex_multiline`: Nullable union field [T, 'null']: same result, other order.
- `replace_regex_multiline`: Plain scalar field: emit a JSON Schema property with a single tagged @T@ type.
- `replace_map`: Map the tagged Avro scalar types to JSON Schema types (longest key wins, so @int@ and @bytes@ are safe).
- `replace`: Avro record type -> JSON Schema object type.
- `replace_regex_multiline`: Record name -> schema title (field 'name' lines are already converted, so only the record name remains).
- `replace`: Open the properties object in place of the fields array.
- `replace_regex_multiline`: Close the properties object in place of the fields array bracket.
- `replace_regex`: Drop the trailing comma after the last property so the output is valid JSON.

## Tags

`convert` `schema` `avro` `json`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
