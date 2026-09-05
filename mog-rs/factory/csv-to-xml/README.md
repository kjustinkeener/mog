# CSV to XML records

Convert CSV rows to XML records

Turn a CSV with a header row into an XML document: one <record> element per row, wrapped in <records>. Uses a per-row template, so edit the element names on the step to match your columns (the demo maps id and name). Mechanical: values are NOT XML-escaped, so run escape-xml first (or html-escape) if your data can contain & < >.

## Run

```
mog -m csv-to-xml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name
1,Ada
2,Bob
```

Output:

```
<records>
  <record>
    <id>1</id>
    <name>Ada</name>
  </record>
  <record>
    <id>2</id>
    <name>Bob</name>
  </record>
</records>
```

## Pipeline

- `row_to_template`: Render each row as an XML record
- `prepend`: Root open tag
- `append`: Root close tag

## Tags

`csv` `xml` `convert` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
