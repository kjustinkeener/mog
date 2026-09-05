# Remove accents (de-accent to ASCII)

Strip accents, mapping Latin letters to ASCII

Strip diacritics from accented Latin letters, mapping them to their base ASCII letter (cafe from cafe-with-accent, naive, resume, Muenchen-style). Useful for building ASCII slugs, search keys, or filenames. Non-Latin scripts are left unchanged.

## Run

```
mog -m remove-accents <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
café naïve résumé Zürich São Paulo
```

Output:

```
cafe naive resume Zurich Sao Paulo
```

## Pipeline

- `normalize`: Fold accented letters to their base ASCII form

## Tags

`text` `unicode` `normalize` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
