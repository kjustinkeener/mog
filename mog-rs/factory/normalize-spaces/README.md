# Normalize runs of spaces

Collapse runs of spaces and tabs to one space

Collapse runs of spaces and tabs down to a single space on each line (line breaks are kept). Cleans up text that was aligned or padded with extra spaces. To also trim leading/trailing space, follow with trim_whitespace.

## Run

```
mog -m normalize-spaces <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
foo    bar	baz
  spaced   out  
```

Output:

```
foo bar baz
 spaced out 
```

## Steps

- `collapse_whitespace`: Collapse space/tab runs to a single space

## Tags

`whitespace` `normalize` `cleanup` `text`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
