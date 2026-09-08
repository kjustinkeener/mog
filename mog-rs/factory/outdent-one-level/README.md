# Outdent one level

Remove up to N leading spaces from each line

Remove up to a fixed number of leading spaces (default 4) from each line, stripping one indent level without touching deeper relative indentation. Lines with less leading space than the level are trimmed to the margin.

## Run

```
mog -m outdent-one-level <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
        deeply
    once
flush
```

Output:

```
    deeply
once
flush
```

## Steps

- `outdent`: Remove up to 4 leading spaces

## Tags

`whitespace` `indent` `format` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
