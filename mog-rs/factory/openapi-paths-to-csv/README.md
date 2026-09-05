# OpenAPI paths to CSV

Convert an OpenAPI 3 spec to a CSV table of endpoints

Turn an OpenAPI 3.x description into a CSV endpoint table -- one row per path+method, with the columns path,method,operation_id,summary,tags. Input may be JSON or YAML (YAML is a superset of JSON, so the same parse handles both) and is read with the real YAML/JSON parsers, not scraped: the spec is parsed, the paths object is lifted out as a balanced JSON value, and re-serialized as canonical YAML so every key sits at a known indent. The methods are the HTTP verb keys under each path (get, put, post, delete, options, head, patch, trace); anything else at that level -- a path-level $ref, shared parameters, servers, summary/description, x- extensions -- is dropped, and a path that ends up with no operation rows is flagged rather than silently vanishing. Multiple tags are joined with a semicolon. Values are emitted as RFC 4180 CSV, so a summary holding a comma or a quote is quoted and internal quotes are doubled -- sort or filter the result to review an API surface, diff two versions of a spec, or check that every operation has an operationId. Scope: one spec per file, and only operationId, summary and tags are exported (parameters, requestBody, responses, security and deprecated are not). $ref resolution is out of scope: a $ref is not followed, so an operation defined behind one contributes no row and its path is flagged. The webhooks and callbacks sections are not read -- this is the paths object only.

## Run

```
mog -m openapi-paths-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
openapi: 3.0.3
info:
  title: Pet Store
  version: 1.2.0
  description: A small, deliberately awkward spec.
servers:
  - url: https://api.example.com/v1
paths:
  /pets:
    parameters:
      - name: X-Trace-Id
        in: header
        required: false
        schema:
          type: string
    get:
      operationId: listPets
      summary: List pets, newest first
```

_(... 83 more line(s))_

Output:

```
path,method,operation_id,summary,tags
/pets,get,listPets,"List pets, newest first",pets;public
/pets,post,createPet,"Register a ""new"" pet",pets
/pets,delete,,Delete every pet,pets;admin
/pets/{petId},get,getPet,,pets
/pets/{petId},put,replacePet,"Replace a pet: name, tags and it's owner",pets
/pets/{petId},patch,updatePet,"Update a pet's name, colour or age",
# WARN(mog): path entry produced no operation rows; it may be a $ref or an extension key
# WARN(mog): path entry produced no operation rows; it may be a $ref or an extension key
```

## Pipeline

- `yaml_to_json`: Parse the spec (YAML or JSON) and re-serialize it on a single line.
- `replace_regex`: Drop everything ahead of the paths object.
- `extract_json`: Take the balanced paths object and discard the rest of the spec.
- `json_to_yaml`: Re-emit the paths object as canonical YAML so every key sits at a known indent.
- `remove_lines_matching`: Drop everything below the operation fields (parameters, responses, schemas).
- `replace_regex`: Fold each operation-level list onto its own key line.
- `remove_lines_matching`: Keep only operationId, summary and tags at the operation level.
- `replace_regex`: Fold each path-level list onto its own key line.
- `remove_lines_matching`: Keep only the HTTP verb keys at the path level.
- `replace_regex_multiline`: Strip the single quotes YAML added around an operationId or summary.
- `replace`: Undo YAML's doubled apostrophes inside those values.
- `replace`: Remove the unquoting marker.
- `replace_regex`: Fold an operation's fields onto its verb line.
- `replace_regex_multiline`: Close every operation line so a field value cannot run past its own field.
- `replace_regex_multiline`: Capture each operation's operationId.
- `replace_regex_multiline`: Give an operation with no operationId an empty one.
- `replace_regex_multiline`: Capture each operation's summary.
- `replace_regex_multiline`: Give an operation with no summary an empty one.
- `replace_regex_multiline`: Capture each operation's tag list.
- `replace_regex_multiline`: Give an operation with no tags an empty list.
- `replace_regex_multiline`: Rewrite each path line as a path row with the other columns blank.
- `replace_regex`: Strip the leading separator from a tag list.
- `replace_regex_multiline`: Rewrite each operation line as a row with a blank path column.
- `replace`: Join multiple tags with a semicolon.
- `replace_regex`: Mark a path row that no operation row follows.
- `flag_matching`: Flag any leftover line the shapes above did not turn into a row.
- `flag_matching`: Flag a path that produced no operation rows.
- `replace`: Hide the double quotes from the quote-aware fill-down that follows.
- `replace_regex`: Mark genuinely empty cells so the fill-down cannot invent a value for them.
- `fill_down`: Carry each path down onto its own operations.
- `remove_lines_matching`: Drop the path marker rows now that their paths are carried down.
- `replace`: Restore the marked empty cells.
- `replace`: Bring the double quotes back.
- `replace`: Double every embedded double quote.
- `replace_regex`: Wrap any field holding a comma or a quote in double quotes.
- `replace`: Swap each unit separator for a comma.
- `prepend`: Add the CSV header row.

## Tags

`openapi` `swagger` `yaml` `json` `csv` `convert` `api`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
