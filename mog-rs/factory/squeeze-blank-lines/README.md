# Squeeze blank lines

Collapse runs of blank lines to a single blank

Collapse each run of two or more blank lines down to a single blank line, so paragraphs stay separated but extra vertical space is removed. To delete all blank lines instead, use remove-blank-lines.

## Run

```
mog -m squeeze-blank-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
a



b

c
```

Output:

```
a

b

c
```

## Steps

- `squeeze_blank_lines`: Collapse blank runs to one

## Tags

`line` `whitespace` `cleanup` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
