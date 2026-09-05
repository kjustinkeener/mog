# HL7 v2 to CSV (flat)

Flatten HL7 v2 segments to CSV

Flatten HL7 v2 messages into CSV: one row per segment, one column per field. Normalizes the segment separator (HL7 uses CR) to newlines, then re-delimits each segment from | to comma (quote-aware). Component (^) and repetition (~) separators are left inside their cells, so this is a flat dump, not a full structural parse. Good for eyeballing or loading a message into a spreadsheet.

## Run

```
mog -m hl7-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
MSH|^~\&|SENDING|FAC|RECEIVING|FAC|20240101120000||ADT^A01|MSG001|P|2.5
PID|1||12345^^^MRN||Doe^John||19800101|M
PV1|1|I|ICU^101^A
```

Output:

```
MSH,^~\&,SENDING,FAC,RECEIVING,FAC,20240101120000,,ADT^A01,MSG001,P,2.5
PID,1,,12345^^^MRN,,Doe^John,,19800101,M
PV1,1,I,ICU^101^A
```

## Pipeline

- `eol_lf`: Normalize CR segment separators to newlines
- `change_delimiter`: Fields: | -> comma

## Tags

`healthcare` `csv` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
