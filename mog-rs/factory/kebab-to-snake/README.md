# kebab-case to snake_case

Convert kebab-case identifiers to snake_case

Convert kebab-case identifiers to snake_case by turning each hyphen between two alphanumerics into an underscore. A spaced dash (a - b) or a leading/standalone hyphen is left alone. Best on a list or column of identifiers.

## Run

```
mog -m kebab-to-snake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
my-var-name
data-source-id
a-b-c-d
spaced - dash
```

Output:

```
my_var_name
data_source_id
a_b_c_d
spaced - dash
```

## Steps

- `replace_regex`: Hyphen between alphanumerics -> underscore (pass 1)
- `replace_regex`: Again, to catch adjacent segments the first pass skipped

## Tags

`case` `identifier` `codemod` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
