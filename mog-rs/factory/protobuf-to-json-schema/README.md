# Protobuf message to JSON Schema

Convert a Protobuf message to JSON Schema

Convert a single Protobuf (proto2/proto3) message into a draft-07 JSON Schema object. Maps scalar field types (int*/uint*/sint*/fixed*/sfixed*->integer, float/double->number, bool->boolean, string/bytes->string), treats a 'repeated' field as a JSON array with 'items', and ignores the 'optional' label and field number. Not a protobuf parser: assumes one field per line and does NOT handle nested/embedded messages, map<>, oneof, enum, imports, packages, options, reserved, or comments.

## Run

```
mog -m protobuf-to-json-schema <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
message User {
  int64 id = 1;
  string name = 2;
  optional string email = 3;
  bool active = 4;
  double score = 5;
  repeated string tags = 6;
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
    "email": { "type": "string" },
    "active": { "type": "boolean" },
    "score": { "type": "number" },
    "tags": { "type": "array", "items": { "type": "string" } }
  }
}
```

## Steps

- `eol_lf`: Normalize line endings to LF so the per-line field regexes are reliable.
- `replace_regex_multiline`: Repeated field -> a JSON array property with an 'items' scalar. Runs before the scalar rule so 'repeated' is not misread as a type.
- `replace_regex_multiline`: Singular scalar field (optional label allowed and dropped) -> a JSON Schema property with a tagged @T@ type.
- `replace_map`: Map the tagged Protobuf scalar types to JSON Schema types (longest key wins).
- `replace_regex_multiline`: message <Name> { -> the JSON Schema object header and open the properties object.
- `replace_regex_multiline`: The message closing brace -> close the properties object and the schema object.
- `replace_regex`: Drop the trailing comma after the last property so the output is valid JSON.

## Tags

`convert` `schema` `json`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
