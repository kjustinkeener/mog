# Sort lines alphabetically

Sort lines alphabetically (A-Z)

Sort the lines in ascending (A-Z) order. Case-sensitive by default. Useful for ordering a list of names, imports, tags, or requirements.

## Run

```
mog -m sort-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
banana
apple
cherry
apple
```

Output:

```
apple
apple
banana
cherry
```

## Pipeline

- `sort_lines`: Sort lines A-Z

## Tags

`line` `sort` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
