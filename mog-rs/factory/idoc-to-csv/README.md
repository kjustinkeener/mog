# SAP IDoc flat file to CSV (lossy)

Flatten a SAP IDoc flat file to CSV

Dump a SAP IDoc flat file to CSV: one row per record line, keyed by the leading segment name (EDI_DC40, E1EDK01, E1EDP01, ...). Trims trailing pad spaces, then splits each line into two columns: the segment name and the rest of the line as a single quoted payload. Does not decode the fixed-width field layout inside a record (client, DOCNUM, SEGNUM, HLEVEL, SDATA stay mashed together in the payload cell), does not distinguish the control record (EDI_DC40) format from data-record formats, and does not resolve segment definitions from the IDoc type. Good for listing which segments a flat IDoc contains and eyeballing raw payloads.

## Run

```
mog -m idoc-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
EDI_DC40                      800000000000012345720ORDERSORDERS05
E1EDK01                       800000000000012345000001 NB0000012345
E1EDK14                       800000000000012345000002 0121000
E1EDP01                       800000000000012345000003 0000010000100
E1EDP19                       800000000000012345000004 001MATERIAL01
E1EDPT1                       800000000000012345000005 0001F
```

Output:

```
EDI_DC40,"800000000000012345720ORDERSORDERS05"
E1EDK01,"800000000000012345000001 NB0000012345"
E1EDK14,"800000000000012345000002 0121000"
E1EDP01,"800000000000012345000003 0000010000100"
E1EDP19,"800000000000012345000004 001MATERIAL01"
E1EDPT1,"800000000000012345000005 0001F"
```

## Pipeline

- `trim_whitespace_right`: Drop trailing pad spaces
- `remove_empty_lines`: Drop blank lines
- `replace_regex_multiline`: Leading segment name -> col1, remainder -> quoted col2

## Tags

`edi` `csv` `convert` `interchange` `lossy` `best-effort`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
