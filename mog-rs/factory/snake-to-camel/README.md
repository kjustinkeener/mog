# snake_case to camelCase

Convert snake_case identifiers to camelCase

Recase each snake_case (or kebab-case) identifier to camelCase: the first word stays lowercase and every following word is capitalized, dropping the separators. The mirror of camel-to-snake. Best on a list or column of identifiers, one per line.

## Run

```
mog -m snake-to-camel <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
user_id
first_name
is_active_flag
parse-json-data
```

Output:

```
userId
firstName
isActiveFlag
parseJsonData
```

## Steps

- `to_camel`: Recase each identifier to camelCase

## Tags

`case` `identifier` `codemod` `convert` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
