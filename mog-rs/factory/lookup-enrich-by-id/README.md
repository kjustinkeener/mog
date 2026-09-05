# Enrich rows by id from a lookup table

Join a lookup table onto rows by id (VLOOKUP)

Join a reference table onto the input by a key column (a keyed lookup / VLOOKUP). Point the 'directory' source at the lookup table with --source directory=<path>. Each input row is matched by its user_id against the directory's id column, and the directory's remaining columns (name, region) are appended. With has_header set, the key columns are named and the appended columns keep their header. Rows whose key has no match get blank cells (on_miss=blank); switch on_miss to leave, drop, or error to change that. The first matching reference row wins on a duplicate key.

## Run

```
mog -m lookup-enrich-by-id <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
order,user_id
1001,2
1002,1
1003,9
```

Output:

```
order,user_id,name,region
1001,2,Bob,EU
1002,1,Alice,US
1003,9,,
```

## Pipeline

- `lookup`: Append name and region from the directory, matched by user_id -> id

## Tags

`compare` `lookup` `join` `csv` `records`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
