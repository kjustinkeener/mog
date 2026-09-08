# RIS to BibTeX

Convert RIS references to BibTeX

Convert RIS reference records (from EndNote / Zotero / PubMed) to BibTeX. Maps the RIS TY type to a BibTeX entry type (JOUR->article, BOOK->book, CPAPER->inproceedings, GEN->misc) and RIS tags to BibTeX fields (AU->author, TI->title, JO->journal, T2->booktitle, PY->year, VL->volume, IS->number, SP->pages, PB->publisher, DO->doi, UR->url). Each entry gets a numbered placeholder cite key (ref1, ref2, ...); rename them to taste. The inverse of bibtex-to-ris. Multiple authors on separate AU lines each become their own author line.

## Run

```
mog -m ris-to-bibtex <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
TY  - JOUR
AU  - Jane Smith
TI  - A Great Paper
JO  - Nature
PY  - 2024
VL  - 12
SP  - 1--10
ER  -
TY  - BOOK
AU  - John Doe
TI  - The Book
PB  - Acme
PY  - 2020
ER  -
```

Output:

```
@article{ref1,
  author = {Jane Smith},
  title = {A Great Paper},
  journal = {Nature},
  year = {2024},
  volume = {12},
  pages = {1--10},
}
@book{ref2,
  author = {John Doe},
  title = {The Book},
  publisher = {Acme},
  year = {2020},
}
```

## Steps

- `replace_regex_multiline`: JOUR -> article
- `replace_regex_multiline`: BOOK -> book
- `replace_regex_multiline`: CPAPER -> inproceedings
- `replace_regex_multiline`: GEN -> misc
- `replace_regex_multiline`: author
- `replace_regex_multiline`: title
- `replace_regex_multiline`: journal
- `replace_regex_multiline`: booktitle
- `replace_regex_multiline`: year
- `replace_regex_multiline`: volume
- `replace_regex_multiline`: number
- `replace_regex_multiline`: pages
- `replace_regex_multiline`: publisher
- `replace_regex_multiline`: doi
- `replace_regex_multiline`: url
- `replace_regex_multiline`: ER -> closing brace
- `stamp_sequence`: Number the placeholder cite keys

## Tags

`citation` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
