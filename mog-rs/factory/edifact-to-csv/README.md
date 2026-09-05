# UN/EDIFACT to CSV (flat, lossy)

Flatten UN/EDIFACT segments to CSV

Flatten a UN/EDIFACT interchange to CSV: one row per segment, one column per data element. Turns the ' segment terminator into newlines, then re-delimits each segment from the + element separator to comma (quote-aware). Component separators (:) are left inside their cells, so this is a flat text dump, not a semantic message parse. Does not resolve UNA delimiter overrides (assumes the standard ' + : terminators), does not process the release/escape character (?), and does not decode segment/message semantics (UNB/UNH/BGM meanings, qualifiers, code lists). A UNA header line, if present, is flattened verbatim.

## Run

```
mog -m edifact-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
UNA:+.? '
UNB+UNOA:1+SENDER:ZZ+RECEIVER:ZZ+240101:1200+1'
UNH+1+ORDERS:D:96A:UN'
BGM+220+ORDER123+9'
DTM+137:20240101:102'
NAD+BY+BUYER01::91'
LIN+1++PRODUCT1:EN'
QTY+21:100'
UNS+S'
UNT+8+1'
UNZ+1+1'
```

Output:

```
UNA:,.? 
UNB,UNOA:1,SENDER:ZZ,RECEIVER:ZZ,240101:1200,1
UNH,1,ORDERS:D:96A:UN
BGM,220,ORDER123,9
DTM,137:20240101:102
NAD,BY,BUYER01::91
LIN,1,,PRODUCT1:EN
QTY,21:100
UNS,S
UNT,8,1
UNZ,1,1
```

## Pipeline

- `replace`: Segment terminator ' -> newline
- `change_delimiter`: Elements: + -> comma (quote-aware)
- `remove_empty_lines`: Drop blank lines left by the trailing terminator

## Tags

`edi` `csv` `convert` `interchange` `lossy` `best-effort`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
