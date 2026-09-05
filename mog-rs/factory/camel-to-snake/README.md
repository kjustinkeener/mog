# camelCase to snake_case

Convert camelCase identifiers to snake_case

Convert camelCase / PascalCase identifiers to snake_case: insert an underscore at each lower-to-upper boundary, then lowercase. Best on a list or column of identifiers, since it lowercases all letters. Runs of capitals (acronyms like HTTP) are kept together (getHTTP -> get_http, not get_h_t_t_p).

## Run

```
mog -m camel-to-snake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
myVarName
getHTTPResponse
userID
parseJSONData
```

Output:

```
my_var_name
get_httpresponse
user_id
parse_jsondata
```

## Pipeline

- `replace_regex`: Insert _ at each lowercase/digit -> uppercase boundary
- `to_lower`: Lowercase the result

## Tags

`case` `identifier` `codemod` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
