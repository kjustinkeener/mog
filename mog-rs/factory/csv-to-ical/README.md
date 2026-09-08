# CSV to iCalendar events

Convert a CSV of events to iCalendar (.ics)

Turn a CSV of events (header row with SUMMARY, DTSTART, DTEND) into an iCalendar (.ics) file: one VEVENT per row, wrapped in a VCALENDAR. Uses a per-row template plus a calendar header/footer. The inverse of ical-to-csv. Timestamps should already be in iCal form (e.g. 20240101T090000Z).

## Run

```
mog -m csv-to-ical <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SUMMARY,DTSTART,DTEND
Standup,20240101T090000Z,20240101T091500Z
Review,20240102T100000Z,20240102T110000Z
```

Output:

```
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//mog//EN
BEGIN:VEVENT
SUMMARY:Standup
DTSTART:20240101T090000Z
DTEND:20240101T091500Z
END:VEVENT
BEGIN:VEVENT
SUMMARY:Review
DTSTART:20240102T100000Z
DTEND:20240102T110000Z
END:VEVENT
END:VCALENDAR
```

## Steps

- `row_to_template`: Render each row as a VEVENT
- `prepend`: Calendar header
- `append`: Calendar footer

## Tags

`csv` `date` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
