# Paste a column from a list

Attach a second column from a parallel list (paste)

Attach a second column to each line by pairing input line i with line i of a named source (Unix paste), joined by a delimiter (tab by default). Bind the source with --source ids=<path> or a 'sources' entry. Stitches parallel lists back into rows (labels + values, names + ids). When the source is shorter than the input, remaining lines get an empty cell.

## Run

```
mog -m paste-column <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
alice
bob
carol
```

Output:

```
alice,U-001
bob,U-002
carol,U-003
```

## Pipeline

- `paste_column`: Append the matching id from the ids source as a comma-separated column

## Tags

`paste` `column` `join` `csv` `list`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
