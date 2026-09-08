# XML to CSV

Convert flat repeated XML records to CSV

Turn flat, repeated XML record elements into CSV -- the read counterpart to CSV to XML records. The XML is parsed for real (xml_to_json), the repeated-record array is lifted out, and each record becomes a row: the header is the union of the records' child element names in first-seen order, a field missing from a record is an empty cell, and entity references (&amp; &lt; &quot;) are decoded. Output is RFC 4180 CSV, so values holding commas or quotes are quoted. SCOPE, plainly: this handles the flat shape only -- a root element wrapping two or more identical record elements whose children are leaf text fields, e.g. <orders><order><id>1</id>...</order>...</orders>. Deeply nested or attribute-heavy XML is NOT in scope: a nested child collapses into one cell of compact JSON and an attribute arrives as an @name column rather than a plain field. Those cells are FLAGGED for review rather than silently dropped, so you can see exactly which records did not fit. A document with only one record element has no array to lift and is rejected by the assert step. For nested XML, run XML to JSON instead and reshape from there.

## Run

```
mog -m xml-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
<?xml version="1.0" encoding="UTF-8"?>
<orders exported="2026-08-14">
  <order>
    <id>1001</id>
    <customer>Ada Lovelace</customer>
    <city>London, UK</city>
    <note>Gift wrap &amp; card</note>
    <total>249.95</total>
  </order>
  <order>
    <id>1002</id>
    <customer>Grace &quot;Amazing&quot; Hopper</customer>
    <city>Arlington</city>
    <note>Leave at the &lt;side&gt; door</note>
    <total>1120.00</total>
  </order>
  <order>
    <id>1003</id>
```

_(... 13 more line(s))_

Output:

```
id,customer,city,note,total
1001,Ada Lovelace,"London, UK",Gift wrap & card,249.95
1002,"Grace ""Amazing"" Hopper",Arlington,Leave at the <side> door,1120.00
1003,Ken Thompson,Murray Hill,,18.40
1004,Barbara Liskov,"Cambridge, MA",Fragile,77.05
```

## Steps

- `xml_to_json`: Parse the XML document into JSON (repeated tags become an array).
- `json_minify`: Re-serialize it on a single line.
- `assert`: Refuse a document with no repeated record element.
- `replace_regex`: Drop the root wrapper ahead of the record array.
- `extract_json`: Take the balanced record array and discard the trailing wrapper.
- `json_to_csv`: Flatten the records to CSV: header is the union of field names, missing fields are blank.
- `flag_matching`: Flag rows whose record did not fit the flat shape.

## Tags

`xml` `csv` `convert` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
