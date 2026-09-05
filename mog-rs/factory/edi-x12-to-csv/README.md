# EDI X12 to CSV (flat)

Flatten EDI X12 segments to CSV

Flatten an EDI X12 interchange into CSV: one row per segment, one column per element. Turns the ~ segment terminator into newlines, then re-delimits each segment from the * element separator to comma (quote-aware). Composite (>) separators are left inside their cells, so this is a flat dump, not a full transaction parse. Good for inspecting or loading an X12 file into a spreadsheet.

## Run

```
mog -m edi-x12-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
ISA*00*SENDER*00*RECEIVER*240101*1200*U*00401*000000001*0*P*>~GS*PO*SENDER*RECEIVER*20240101*1200*1*X*004010~ST*850*0001~SE*2*0001~
```

Output:

```
ISA,00,SENDER,00,RECEIVER,240101,1200,U,00401,000000001,0,P,>
GS,PO,SENDER,RECEIVER,20240101,1200,1,X,004010
ST,850,0001
SE,2,0001
```

## Pipeline

- `replace`: Segment terminator ~ -> newline
- `change_delimiter`: Elements: * -> comma
- `remove_empty_lines`: Drop the trailing empty segment

## Tags

`edi` `csv` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
