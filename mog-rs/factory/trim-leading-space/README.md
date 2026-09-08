# Trim leading whitespace

Remove leading whitespace from each line

Remove leading whitespace (spaces and tabs) from the start of every line, leaving trailing whitespace and blank lines as-is. A left-only trim.

## Run

```
mog -m trim-leading-space <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
   indented
	tabbed
flush
```

Output:

```
indented
tabbed
flush
```

## Steps

- `trim_whitespace_left`: Remove leading whitespace

## Tags

`whitespace` `cleanup` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
