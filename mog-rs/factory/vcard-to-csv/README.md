# vCard contacts to CSV

Flatten vCard contacts to a CSV table

Flatten simple vCard (.vcf) contact cards into a CSV table, one row per contact. Drops the BEGIN:VCARD and VERSION lines, treats each END:VCARD as a record boundary, then pivots the remaining FIELD:value lines into columns (union of field names, first-seen order). Mechanical: fields with structured parameters (e.g. TEL;TYPE=CELL:...) keep the parameter in the column name, so best on plain cards.

## Run

```
mog -m vcard-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
BEGIN:VCARD
VERSION:3.0
FN:Ada Lovelace
EMAIL:ada@example.com
TEL:555-0100
END:VCARD
BEGIN:VCARD
VERSION:3.0
FN:Bob Stone
EMAIL:bob@example.com
END:VCARD
```

Output:

```
FN,EMAIL,TEL
Ada Lovelace,ada@example.com,555-0100
Bob Stone,bob@example.com,
```

## Steps

- `remove_lines_matching`: Drop the BEGIN:VCARD and VERSION metadata lines
- `replace_regex_multiline`: Turn each END:VCARD into a blank line (a record boundary)
- `records_to_columns`: Pivot FIELD:value stanzas into a CSV table

## Tags

`contacts` `csv` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
