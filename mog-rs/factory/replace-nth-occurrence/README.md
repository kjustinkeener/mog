# Replace the Nth match

Replace only the Nth occurrence of a pattern

Replace ONLY the Nth occurrence (1-based) of the find pattern, leaving the others untouched. This recipe replaces the 2nd match. Positional replace for when you need to change one specific occurrence.

## Run

```
mog -m replace-nth-occurrence <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
col_a, col_b, col_c, col_d
```

Output:

```
col_a, COL_b, col_c, col_d
```

## Pipeline

- `replace_nth`: Replace the 2nd 'col' with 'COL'

## Tags

`replace` `filter` `column` `single` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
