# Reconcile two CSV exports by key

Reconcile two CSV exports by key column

Match the rows of a CSV input against a previous CSV snapshot by a key column and annotate what changed. Point the 'previous' source at the older export with --source previous=<path>. Each input row is prefixed with a status field: ADDED (key not in the previous export), CHANGED (key present but the row differs), or SAME; keys present in the previous export but gone from the input are emitted as REMOVED. With has_header set, the key is a column name and a 'status' header is prepended. Compares whole rows; it does not merge fields.

## Run

```
mog -m reconcile-exports <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name,role
1,Alice,admin
2,Bob,admin
4,Dave,viewer
5,Erin,editor
```

Output:

```
status,id,name,role
SAME,1,Alice,admin
CHANGED,2,Bob,admin
ADDED,4,Dave,viewer
SAME,5,Erin,editor
REMOVED,3,Carol,viewer
```

## Pipeline

- `reconcile`: Annotate each row ADDED / CHANGED / SAME / REMOVED versus the previous export, keyed by id

## Tags

`compare` `csv` `diff` `audit` `records`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
