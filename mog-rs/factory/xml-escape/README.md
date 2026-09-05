# XML/HTML-escape each line

Escape & < > " ' as XML entities

Replace the five XML special characters ( & < > " ' ) with their entity references on every line, so the text is safe to drop inside an XML or HTML document as character data or an attribute value.

## Run

```
mog -m xml-escape <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
a < b && c > d
say "hi" & 'bye'
```

Output:

```
a &lt; b &amp;&amp; c &gt; d
say &quot;hi&quot; &amp; &apos;bye&apos;
```

## Pipeline

- `escape_xml`: Escape XML special characters

## Tags

`escape` `xml` `html` `encode` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
