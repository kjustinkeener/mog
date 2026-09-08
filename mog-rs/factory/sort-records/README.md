# Sort paragraph records

Sort blank-line-separated records by first line

Sort blank-line-separated multi-line records (paragraphs) as whole units by their first line, alphabetically. Handy for ordering config stanzas, address blocks, or changelog entries. Records are runs of non-blank lines separated by one or more blank lines; output re-joins them with a single blank line. Set 'by' to whole_block or key_regex, and 'numeric'/'ignore_case' as needed.

## Run

```
mog -m sort-records <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
charlie
z-body

alpha
a-body

bravo
b-body
```

Output:

```
alpha
a-body

bravo
b-body

charlie
z-body
```

## Steps

- `sort_blocks`

## Tags

`sort` `fragment` `records` `prose` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
