# Fixed-width to CSV

Convert fixed-width columns to CSV

Split each line into fields by fixed column widths (comma-separated), trimming each field. Set the widths with -D widths=... (default 8,10,6). Characters past the last width are dropped.

## Run

```
mog -m fixed-width-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Ada     Engineer  42
Bo      Manager   7
```

Output:

```
Ada,Engineer,42
Bo,Manager,7
```

## Pipeline

- `fixed_width_to_csv`: Split fixed-width columns into CSV.

## Tags

`convert` `column` `csv` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
