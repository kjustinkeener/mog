# Replace only the first match

Replace just the first occurrence of a pattern

Replace only the FIRST occurrence of the find pattern in the input, leaving all later occurrences untouched. Useful for changing a single leading token (a title, a first flag) without a global sweep.

## Run

```
mog -m replace-first-only <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
TODO: a
TODO: b
TODO: c
```

Output:

```
DONE: a
TODO: b
TODO: c
```

## Pipeline

- `replace_first`: Replace the first 'TODO' with 'DONE'

## Tags

`replace` `filter` `single` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
