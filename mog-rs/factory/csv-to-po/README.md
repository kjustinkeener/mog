# CSV to gettext PO

Convert a msgid,msgstr CSV to a gettext .po catalog

Turn a two-column CSV (header row msgid,msgstr) back into a gettext .po catalog, one entry per row, and prepend a minimal PO header so the result is a loadable catalog. Uses a per-row template, so you can edit the step to emit reference comments or a msgctxt. The inverse of po-to-csv. Expects the exact header names msgid and msgstr; values are written verbatim between the quotes, so pre-escape any embedded double quotes or newlines in the CSV.

## Run

```
mog -m csv-to-po <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
msgid,msgstr
"Hello, world","Bonjour, le monde"
"Goodbye","Au revoir"
"Save","Enregistrer"
```

Output:

```
msgid ""
msgstr ""
"Content-Type: text/plain; charset=UTF-8\n"

msgid "Hello, world"
msgstr "Bonjour, le monde"

msgid "Goodbye"
msgstr "Au revoir"

msgid "Save"
msgstr "Enregistrer"
```

## Pipeline

- `row_to_template`: Render each row as a PO entry
- `prepend`: Add a minimal PO header entry

## Tags

`i18n` `csv` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
