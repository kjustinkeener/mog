# ldapsearch output to CSV

Flatten ldapsearch LDIF output to CSV

Flatten ldapsearch LDIF-style output (blank-line-separated blocks of attr: value lines) into a CSV table, one row per entry with a column per attribute seen. Splits each line on the first colon. Good for turning a directory dump into a spreadsheet. Base64 (attr:: value) and folded continuation lines are not decoded.

## Run

```
mog -m ldapsearch-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
dn: uid=alice,ou=people,dc=example,dc=com
uid: alice
cn: Alice Smith
mail: alice@example.com

dn: uid=bob,ou=people,dc=example,dc=com
uid: bob
cn: Bob Jones
mail: bob@example.com
```

Output:

```
dn,uid,cn,mail
"uid=alice,ou=people,dc=example,dc=com",alice,Alice Smith,alice@example.com
"uid=bob,ou=people,dc=example,dc=com",bob,Bob Jones,bob@example.com
```

## Steps

- `records_to_columns`: Pivot each entry block into a CSV row

## Tags

`config` `csv` `convert` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
