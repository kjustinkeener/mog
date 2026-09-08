# Convert ISO dates to epoch

Convert ISO dates to Unix epoch

Rewrite the ISO date/datetime in a delimited log/CSV to a Unix-epoch number (seconds, UTC), leaving the other columns alone. Defaults to column 1; set field for a different column, or unit to millis. Values that do not parse and empty cells pass through. The inverse of log-epoch-to-iso.

## Run

```
mog -m iso-to-epoch <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
2023-11-14T22:13:20Z,alice,login
2023-11-14T23:13:20Z,bob,logout
```

Output:

```
1700000000,alice,login
1700003600,bob,logout
```

## Steps

- `iso_to_epoch`: Column 1 ISO datetime -> epoch seconds

## Tags

`data` `time` `date` `csv`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
