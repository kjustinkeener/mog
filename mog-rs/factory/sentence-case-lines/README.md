# Sentence-case each line

Capitalize the first letter of every sentence

Capitalize the first letter of each sentence in every line, leaving the rest of the casing untouched. Useful for tidying prose, headings, or notes that were typed in a hurry.

## Run

```
mog -m sentence-case-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
hello world. how are you? fine, thanks.
the cat sat. the dog ran.
```

Output:

```
Hello world. How are you? Fine, thanks.
The cat sat. The dog ran.
```

## Steps

- `to_sentence`: Capitalize the first letter of each sentence

## Tags

`case` `text` `prose` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
