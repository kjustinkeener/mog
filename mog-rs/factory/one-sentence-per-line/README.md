# One sentence per line

Break prose so each sentence is on its own line

Break prose so each sentence sits on its own line, inserting a newline after sentence-ending punctuation (. ! ?) followed by spaces, so git diffs line up per sentence instead of reflowing a whole paragraph. Also breaks after abbreviations like 'e.g.' or 'Mr.'.

## Run

```
mog -m one-sentence-per-line <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Hello world. This is a test! Is it? Yes.
```

Output:

```
Hello world.
This is a test!
Is it?
Yes.
```

## Pipeline

- `replace_regex`: Break after sentence-ending punctuation followed by spaces

## Tags

`text` `prose` `markdown` `diff`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
