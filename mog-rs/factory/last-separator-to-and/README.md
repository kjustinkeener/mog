# Last list separator to 'and'

Change the last list comma to 'and'

Replace only the LAST comma-space in the text with ' and ', turning 'apples, oranges, pears' into 'apples, oranges and pears' (a non-Oxford list join). Written for a single list on one line; for many lists, run it per line.

## Run

```
mog -m last-separator-to-and <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apples, oranges, pears
```

Output:

```
apples, oranges and pears
```

## Pipeline

- `replace_last`

## Tags

`replace` `list` `prose` `column` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
