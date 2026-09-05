# Paste cleanup: Excel / Sheets

Clean up a table pasted from Excel or Sheets

Tidy a table pasted from Excel or Google Sheets. Normalizes line endings to LF, trims the whitespace padding around each tab-separated cell, and drops fully-empty rows (blank or only tabs, including the trailing empty row a sheet paste leaves). Cells stay tab-separated. Drops every fully-empty row, not only trailing ones, so an intentional blank separator row would go too.

## Run

```
mog -m paste-cleanup-excel <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Name 	 Age 	 City
 Alice 	30	 Boston 
		
Bob	 25 	NYC
		
```

Output:

```
Name	Age	City
Alice	30	Boston
Bob	25	NYC
```

## Pipeline

- `eol_lf`: Convert CRLF/CR to LF, stripping stray carriage returns.
- `replace_regex_multiline`: Remove spaces padding either side of a tab delimiter.
- `replace_regex_multiline`: Trim leading and trailing spaces on each row (first and last cell).
- `remove_lines_matching`: Remove rows that are blank or only tabs.

## Tags

`paste` `cleanup` `csv` `tsv` `table`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
