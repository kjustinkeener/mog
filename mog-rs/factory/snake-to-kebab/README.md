# snake_case to kebab-case

Convert snake_case identifiers to kebab-case

Convert snake_case identifiers to kebab-case by turning each underscore between two alphanumerics into a hyphen. A leading/trailing or standalone underscore is left alone. Best on a list or column of identifiers.

## Run

```
mog -m snake-to-kebab <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
my_var_name
data_source_id
a_b_c_d
_leading_kept
```

Output:

```
my-var-name
data-source-id
a-b-c-d
_leading-kept
```

## Pipeline

- `replace_regex`: Underscore between alphanumerics -> hyphen (pass 1)
- `replace_regex`: Again, to catch adjacent segments the first pass skipped

## Tags

`case` `identifier` `codemod` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
