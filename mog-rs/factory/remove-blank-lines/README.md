# Remove blank lines

Delete every blank line, closing the gaps

Delete every empty line, closing the gaps. Non-blank lines keep their order. To collapse runs of blanks to a single blank instead, use squeeze-blank-lines.

## Run

```
mog -m remove-blank-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
one

two


three
```

Output:

```
one
two
three
```

## Pipeline

- `remove_empty_lines`: Drop empty lines

## Tags

`line` `whitespace` `cleanup` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
