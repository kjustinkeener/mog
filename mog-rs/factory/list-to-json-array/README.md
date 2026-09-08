# Lines to JSON array

Convert a list of lines to a JSON array

Turn a plain list (one item per line) into a single-line JSON array of strings: each line is escaped and quoted as a JSON string, the lines are joined with commas, and the whole thing is wrapped in [ ]. Blank lines are dropped first. Handy for pasting a list into a config or code as a JSON array.

## Run

```
mog -m list-to-json-array <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apple
banana
cherry pie
```

Output:

```
["apple","banana","cherry pie"]
```

## Steps

- `remove_empty_lines`: Drop blank lines
- `escape_json`: Escape + quote each line as a JSON string
- `join_lines`: Join into one line with commas
- `replace_regex`: Drop the trailing newline so the bracket lands correctly
- `prepend`: Open the array
- `append`: Close the array

## Tags

`json` `list` `convert` `encode`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
