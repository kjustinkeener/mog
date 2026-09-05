# Convert epoch timestamps to ISO dates

Convert epoch timestamps to ISO dates

Rewrite the Unix-epoch timestamp in a delimited log/CSV to a human-readable ISO datetime (UTC), leaving the other columns alone. Defaults to column 1 in seconds; set field for a different column, unit to millis for millisecond epochs, or format to date for just YYYY-MM-DD. Non-numeric values and empty cells pass through, so a header row or a stray line is safe. Deterministic and reversible with iso-to-epoch.

## Run

```
mog -m log-epoch-to-iso <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
1700000000,alice,login
1700003600,bob,logout
1700007200,carol,login
```

Output:

```
2023-11-14T22:13:20Z,alice,login
2023-11-14T23:13:20Z,bob,logout
2023-11-15T00:13:20Z,carol,login
```

## Pipeline

- `epoch_to_iso`: Column 1 epoch seconds -> ISO datetime (UTC)

## Tags

`data` `time` `log` `csv` `date`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
