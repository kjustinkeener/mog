# XML to JSON

Convert XML to JSON

Parse XML into JSON (reading only). Attributes become @name keys, child elements become keys by tag (repeated tags become arrays), and text becomes #text (or the element's value when it has no attributes or children). Single-document XML.

## Run

```
mog -m xml-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
<order id="7">
  <item>Pen</item>
  <item>Ink</item>
  <note>urgent</note>
</order>
```

Output:

```
{
  "order": {
    "@id": "7",
    "item": [
      "Pen",
      "Ink"
    ],
    "note": "urgent"
  }
}
```

## Pipeline

- `xml_to_json`: Convert the XML document to JSON.

## Tags

`convert` `xml` `json` `data`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
