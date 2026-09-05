# CSV to HTML table

Convert CSV to an HTML table

Render CSV (with a header row) as an HTML <table> (thead + tbody), with cell text HTML-escaped.

## Run

```
mog -m csv-to-html <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name,role
Ada,Engineer
Bo,Manager
```

Output:

```
<table>
  <thead>
    <tr><th>name</th><th>role</th></tr>
  </thead>
  <tbody>
    <tr><td>Ada</td><td>Engineer</td></tr>
    <tr><td>Bo</td><td>Manager</td></tr>
  </tbody>
</table>
```

## Pipeline

- `csv_to_html`: Render the CSV as an HTML table.

## Tags

`convert` `csv` `html` `table`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
