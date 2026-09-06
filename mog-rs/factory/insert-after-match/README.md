# Insert a line after each match

Add a new line after every line matching a pattern

Insert a fixed line of text immediately after every line matching a regex. This mog stamps a comment after each line beginning with BEGIN; change the pattern/text to annotate any marker.

## Run

```
mog -m insert-after-match <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
BEGIN alpha
body one
END
BEGIN beta
body two
END
```

Output:

```
BEGIN alpha
-- inserted by mog
body one
END
BEGIN beta
-- inserted by mog
body two
END
```

## Pipeline

- `insert_after_matching`: Insert a comment after each BEGIN line

## Tags

`line` `sql` `comment` `fragment` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
