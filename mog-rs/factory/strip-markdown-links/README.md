# Unwrap Markdown links to text

Unwrap Markdown links and images to their text

Replace Markdown links and images with just their visible text: [label](url) becomes label, and ![alt](img) becomes alt. Leaves the surrounding prose intact. Useful for turning Markdown into plain text or stripping URLs from a document. Reference-style links ([label][ref]) are not handled.

## Run

```
mog -m strip-markdown-links <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
See [the docs](https://example.com/docs) for details.
Here is a logo: ![company logo](assets/logo.png).
Plain text stays plain.
```

Output:

```
See the docs for details.
Here is a logo: company logo.
Plain text stays plain.
```

## Pipeline

- `replace_regex`: Images: ![alt](img) -> alt
- `replace_regex`: Links: [label](url) -> label

## Tags

`markdown` `links` `strip` `plaintext` `docs`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
