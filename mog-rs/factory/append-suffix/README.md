# Append text to every line

Add a fixed suffix to the end of each line

Append a fixed string to the end of every line. This recipe adds a semicolon; change the suffix via -D or by editing the step (e.g. a trailing comma for a list, or a Markdown line break).

## Run

```
mog -m append-suffix <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT 1
SELECT 2
SELECT 3
```

Output:

```
SELECT 1;
SELECT 2;
SELECT 3;
```

## Pipeline

- `suffix_lines`: Append ';' to each line

## Tags

`prefix` `line` `format` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
