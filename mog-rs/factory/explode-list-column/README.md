# Explode a list-valued column

Split a semicolon list in one field into rows

Split a field whose value is an inner list (semicolon-separated here) into multiple rows, one per value, repeating the other columns. Turns 'id, a;b;c' into three rows. A CSV normalize-to-long transform.

## Run

```
mog -m explode-list-column <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
1,a;b;c
2,d
3,e;f
```

Output:

```
1,a
1,b
1,c
2,d
3,e
3,f
```

## Steps

- `explode_field`: Explode field 2's ';' list into rows

## Tags

`data` `csv` `fragment` `column` `normalize`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
