# gettext PO to CSV

Convert a gettext .po catalog to a msgid,msgstr CSV

Flatten a gettext .po translation catalog into a two-column CSV (msgid,msgstr), one row per entry, for handoff to a translator or a spreadsheet. Drops all comment lines (#, #., #:, #~), the empty header entry, and folded header continuation lines. Both columns are CSV-quoted. Assumes each msgid and msgstr sits on a single line; multi-line folded strings and plural forms (msgid_plural / msgstr[N]) are not reassembled, so run it on catalogs with single-line entries. Inverse of csv-to-po.

## Run

```
mog -m po-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# French translations for the app.
# Copyright (C) 2026
#
msgid ""
msgstr ""
"Project-Id-Version: app 1.0\n"
"Content-Type: text/plain; charset=UTF-8\n"

#: src/main.rs:12
msgid "Hello, world"
msgstr "Bonjour, le monde"

#. greeting shown at startup
msgid "Goodbye"
msgstr "Au revoir"

msgid "Save"
msgstr "Enregistrer"
```

Output:

```
msgid,msgstr
"Hello, world","Bonjour, le monde"
"Goodbye","Au revoir"
"Save","Enregistrer"
```

## Steps

- `remove_lines_matching`: Drop every comment line (#, #., #:, #~, #,)
- `replace_regex_multiline`: Join each msgid line with the msgstr below it into one quoted CSV row
- `remove_lines_matching`: Drop folded header continuation lines (a lone quoted string)
- `remove_lines_matching`: Drop the empty header row
- `remove_empty_lines`: Remove the blank lines that separated entries
- `prepend`: Add the CSV header row

## Tags

`i18n` `csv` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
