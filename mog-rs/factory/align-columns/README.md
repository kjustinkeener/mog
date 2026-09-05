# Align columns on a delimiter

Pad delimited cells so columns line up

Pad the cells of a delimited block so every column lines up (like column -t / Tabularize). Splits each line on the separator, trims cells, and pads each column to its widest value. Defaults to a comma; change the separator on the step for tabs or another delimiter. Great for making CSV or config blocks human-readable.

## Run

```
mog -m align-columns <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name,city
1,Ada,London
2,Robert,Paris
```

Output:

```
id , name   , city
1  , Ada    , London
2  , Robert , Paris
```

## Pipeline

- `align_columns`: Pad each column to its widest cell

## Tags

`indent` `column` `format` `table` `whitespace` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
