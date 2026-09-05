# Indent every line

Add leading spaces to each line

Add a fixed amount of leading indentation (default 4 spaces) to every line. Useful for nesting a block under a parent, or formatting a snippet for a Markdown code fence. The inverse of dedent-block.

## Run

```
mog -m indent-block <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
line one
line two
line three
```

Output:

```
    line one
    line two
    line three
```

## Pipeline

- `indent`: Indent each line by 4 spaces

## Tags

`whitespace` `indent` `format` `prefix` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
