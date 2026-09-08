# Mask a delimited field

Hide the middle of a field, keeping the last 4

Mask the middle of one delimited field, keeping a few leading and trailing characters. This mog masks the 2nd comma-field keeping its last 4 characters, for redacting a card or account number in a CSV column.

## Run

```
mog -m mask-field <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Alice,4111111111111234
Bob,4222222222225678
```

Output:

```
Alice,************1234
Bob,************5678
```

## Steps

- `mask_field`: Mask field 2, keep last 4

## Tags

`data` `redact` `pii` `csv`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
