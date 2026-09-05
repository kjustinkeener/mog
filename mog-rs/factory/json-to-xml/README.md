# JSON to XML

Convert JSON to XML

Emit JSON as XML, inverting XML to JSON: @keys become attributes, #text becomes text, other keys become child elements (arrays repeat). The top level must be an object; a single key is the root element, else it is wrapped in <root>. The write counterpart to XML to JSON.

## Run

```
mog -m json-to-xml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"order": {"@id": "7", "item": ["Pen", "Ink"], "note": "urgent"}}
```

Output:

```
<order id="7">
  <item>Pen</item>
  <item>Ink</item>
  <note>urgent</note>
</order>
```

## Pipeline

- `json_to_xml`: Emit the JSON as XML.

## Tags

`convert` `json` `xml` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
