# CSV to vCard contacts

Convert a CSV of contacts to vCard (.vcf)

Turn a CSV of contacts (header row with FN, EMAIL, TEL) into vCard (.vcf) cards, one card per row. Uses a per-row template, so you can edit the fields on the step to add ORG, ADR, etc. The inverse of vcard-to-csv. Expects the exact header names FN, EMAIL, TEL; blank cells still emit their line.

## Run

```
mog -m csv-to-vcard <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
FN,EMAIL,TEL
Ada Lovelace,ada@example.com,555-0100
Bob Stone,bob@example.com,555-0200
```

Output:

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
TEL:555-0200
END:VCARD
```

## Pipeline

- `row_to_template`: Render each row as a vCard card

## Tags

`csv` `contacts` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
