# HTML table to CSV

Convert an HTML table to CSV

Extract an HTML <table> into CSV (regex-based, for simple tables): cell tags are stripped and common entities decoded. Non-table markup is ignored. The read counterpart to CSV to HTML table.

## Run

```
mog -m html-table-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
<table>
  <tr><th>name</th><th>role</th></tr>
  <tr><td>Ada</td><td>Engineer</td></tr>
  <tr><td>Bo</td><td>Manager</td></tr>
</table>
```

Output:

```
name,role
Ada,Engineer
Bo,Manager
```

## Steps

- `html_table_to_csv`: Extract the HTML table rows into CSV.

## Tags

`convert` `html` `csv` `table`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
