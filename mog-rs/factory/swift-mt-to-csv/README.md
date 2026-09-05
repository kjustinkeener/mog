# SWIFT MT message to CSV (lossy)

Flatten a SWIFT MT message to tag,value CSV rows

Extract a SWIFT MT message (e.g. MT103) to a two-column CSV of tag,value rows (lossy). Drops the block 1/2/3/5 header and trailer braces ({1:...}{2:...}{4: ... -}) and, for each field line in the text block, emits the :NN[a]: tag and its value (value always quoted, since SWIFT uses a comma as the decimal separator). Multi-line field continuations (address lines under :50K:/:59: etc.) become rows with an EMPTY tag column holding the continuation text. Does NOT decode field sub-formats or qualifiers (a :32A: value stays as one raw string like 240101USD1000,00), does NOT parse the application/user headers in blocks 1-3, and assumes one field or continuation per physical line. Good for pulling the field tags and raw values out of an MT message into a spreadsheet.

## Run

```
mog -m swift-mt-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{1:F01BANKBEBBAXXX0000000000}{2:I103BANKDEFFXXXXN}{3:{108:REF108}}{4:
:20:REFERENCE123
:23B:CRED
:32A:240101USD1000,00
:50K:/12345678
JOHN DOE
221 BAKER STREET
:59:/98765432
JANE SMITH
:70:INVOICE 998877
:71A:SHA
-}
{5:{CHK:ABCDEF123456}}
```

Output:

```
20,"REFERENCE123"
23B,"CRED"
32A,"240101USD1000,00"
50K,"/12345678"
,"JOHN DOE"
,"221 BAKER STREET"
59,"/98765432"
,"JANE SMITH"
70,"INVOICE 998877"
71A,"SHA"
```

## Pipeline

- `remove_lines_matching`: Drop brace-framed header/opener lines ({1:..., {4:, {5:...)
- `remove_lines_matching`: Drop the block-4 trailer line (-})
- `remove_empty_lines`: Drop blank lines
- `replace_regex_multiline`: Continuation lines (no leading tag) -> empty tag + quoted value
- `replace_regex_multiline`: Field lines :NN[a]:value -> tag,"value"

## Tags

`finance` `csv` `convert` `interchange` `lossy` `best-effort`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
