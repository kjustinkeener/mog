# BibTeX to RIS

Convert BibTeX references to RIS

Convert well-formatted BibTeX entries to RIS (the reference format EndNote / Zotero / RefWorks import). Maps the entry type to a RIS TY tag (article->JOUR, book->BOOK, inproceedings->CPAPER, misc->GEN) and common fields to RIS tags (author->AU, title->TI, journal->JO, booktitle->T2, year->PY, volume->VL, number->IS, pages->SP, publisher->PB, doi->DO, url->UR), closing each record with ER. Expects one field per line with a brace- or quote-delimited value; does not handle multi-line values, nested braces, @string macros, or LaTeX accents.

## Run

```
mog -m bibtex-to-ris <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
@article{smith2024,
  author = {Jane Smith},
  title = {A Great Paper},
  journal = {Nature},
  year = {2024},
  volume = {12},
  pages = {1--10}
}

@book{doe2020,
  author = {John Doe},
  title = {The Book},
  publisher = {Acme},
  year = {2020}
}
```

Output:

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

## Pipeline

- `replace_regex_multiline`: @article -> JOUR
- `replace_regex_multiline`: @book -> BOOK
- `replace_regex_multiline`: @inproceedings -> CPAPER
- `replace_regex_multiline`: @misc -> GEN
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
- `replace_regex_multiline`: closing brace -> ER

## Tags

`citation` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
