# snake_case to PascalCase

Convert snake_case identifiers to PascalCase

Recase each snake_case (or kebab-case) identifier to PascalCase (UpperCamelCase): every word is capitalized and separators are dropped. Best on a list or column of identifiers, one per line.

## Run

```
mog -m snake-to-pascal <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
user_id
first_name
http_response
parse-json-data
```

Output:

```
UserId
FirstName
HttpResponse
ParseJsonData
```

## Steps

- `to_pascal`: Recase each identifier to PascalCase

## Tags

`case` `identifier` `codemod` `convert` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
