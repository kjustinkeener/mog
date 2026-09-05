# Shift all dates by N days

Shift every ISO date by a fixed number of days

Offset every ISO date (YYYY-MM-DD) in the text by a fixed number of days, preserving the interval between them. The demo shifts back one year (-365); edit days on the step. Useful for anonymizing dates in test/sample data while keeping durations intact, or for moving a schedule. Only YYYY-MM-DD tokens change.

## Run

```
mog -m shift-dates <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Kickoff on 2024-03-15, review 2024-03-22, launch 2024-04-01.
No dates on this line.
```

Output:

```
Kickoff on 2023-03-16, review 2023-03-23, launch 2023-04-02.
No dates on this line.
```

## Pipeline

- `shift_dates`: Shift dates back one year

## Tags

`data` `time` `date` `privacy` `numbers` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
