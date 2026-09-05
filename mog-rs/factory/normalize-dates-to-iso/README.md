# Normalize dates to ISO 8601

Rewrite mixed date formats to ISO YYYY-MM-DD

Rewrite the common written date formats to ISO 8601 (YYYY-MM-DD) so a text file or a CSV column sorts and compares correctly. Handled deterministically: YYYY/MM/DD and YYYY.MM.DD, YYYYMMDD, DD-Mon-YYYY, DD Month YYYY, Mon DD YYYY and Month DD, YYYY (ordinal suffixes allowed), and the dotted European DD.MM.YYYY. The genuinely ambiguous NN/NN/YYYY slash form is read MONTH FIRST (the US convention); every line where the day-first reading is also valid (both numbers 12 or less) is flagged for review instead of being silently guessed, and a two-digit year is flagged and left alone because the century is unknown. Swap the two group numbers in the slash-date step's replacement to read that form day first.

## Run

```
mog -m normalize-dates-to-iso <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,event,date
1,launch,2024/03/07
2,review,07-Mar-2024
3,audit,Mar 7 2024
4,summit,7 March 2024
5,close,20240307
6,invoice,03/07/2024
7,shipment,12/25/2024
8,legacy,03/07/24
9,filing,07.03.2024
10,keynote,"March 7th, 2024"
11,planning,next Tuesday
12,already,2024-03-07
13,renewal,2024.11.30
14,ticket,1-Sep-2025
```

Output:

```
id,event,date
1,launch,2024-03-07
2,review,2024-03-07
3,audit,2024-03-07
4,summit,2024-03-07
5,close,2024-03-07
6,invoice,2024-03-07 # FIXME(mog): both readings of this slash date are valid; kept the month-first one
7,shipment,2024-12-25
8,legacy,03/07/24 # FIXME(mog): two-digit year so the century is unknown; left as written
9,filing,2024-03-07
10,keynote,"2024-03-07"
11,planning,next Tuesday
12,already,2024-03-07
13,renewal,2024-11-30
14,ticket,2025-09-01
```

## Pipeline

- `flag_matching`: Flag slash dates that read both ways
- `flag_matching`: Flag slash dates with a two-digit year
- `replace_regex`: YYYY/MM/DD and YYYY.MM.DD to dashes
- `replace_regex`: Dotted European DD.MM.YYYY (day first)
- `replace_regex`: Compact YYYYMMDD
- `replace_regex`: Month DD YYYY to year-month-day order
- `replace_regex`: DD Month YYYY to year-month-day order
- `lookup_replace`: Month name to month number
- `replace_regex`: Slash dates NN/NN/YYYY read month first
- `replace_regex`: Zero-pad the month
- `replace_regex`: Zero-pad the day

## Tags

`date` `time` `csv` `normalize` `data` `cleanup`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
