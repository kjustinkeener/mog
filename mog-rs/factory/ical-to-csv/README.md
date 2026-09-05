# iCalendar events to CSV

Convert iCalendar events to CSV

Flatten the VEVENTs of an iCalendar (.ics) file into a CSV table, one row per event. Drops the VCALENDAR wrapper (BEGIN:VCALENDAR / VERSION / PRODID / BEGIN:VEVENT / END:VCALENDAR), treats each END:VEVENT as a record boundary, then pivots the remaining FIELD:value lines (SUMMARY, DTSTART, DTEND, LOCATION, ...) into columns. Fields with parameters (e.g. DTSTART;TZID=...) keep the parameter in the column name, so best on plain calendars.

## Run

```
mog -m ical-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//EN
BEGIN:VEVENT
SUMMARY:Standup
DTSTART:20240101T090000Z
DTEND:20240101T091500Z
LOCATION:Room A
END:VEVENT
BEGIN:VEVENT
SUMMARY:Review
DTSTART:20240102T100000Z
END:VEVENT
END:VCALENDAR
```

Output:

```
SUMMARY,DTSTART,DTEND,LOCATION
Standup,20240101T090000Z,20240101T091500Z,Room A
Review,20240102T100000Z,,
```

## Pipeline

- `remove_lines_matching`: Drop the calendar wrapper and per-event BEGIN lines
- `replace_regex_multiline`: Turn each END:VEVENT into a blank line (a record boundary)
- `records_to_columns`: Pivot FIELD:value stanzas into a CSV table

## Tags

`date` `csv` `convert` `interchange`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
