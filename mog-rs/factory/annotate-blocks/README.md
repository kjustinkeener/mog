# Indent each delimited block

Run a sub-recipe over every BEGIN..END block

For each region between a '-- BEGIN' line and the next '-- END' line, run a sub-recipe over the whole block and replace it with the result. Demonstrates block-scoped composition: swap the run recipe to comment, renumber, or reformat each block. Delimiters are matched by regex.

## Run

```
mog -m annotate-blocks <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
-- BEGIN alpha
first line
second line
-- END
outside the block
-- BEGIN beta
only line
-- END
```

Output:

```
    -- BEGIN alpha
    first line
    second line
    -- END
outside the block
    -- BEGIN beta
    only line
    -- END
```

## Pipeline

- `for_each_block`: Indent every -- BEGIN..-- END block

## Tags

`docker` `fragment` `single-action` `indent`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
