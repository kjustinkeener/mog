# Remove common leading indent

Strip the shared leading whitespace from all lines

Remove the longest common leading-whitespace prefix shared by all non-blank lines, so an over-indented block returns to the left margin while relative indentation is kept. The inverse of indent-block.

## Run

```
mog -m dedent-block <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
    def foo():
        return 1
    # done
```

Output:

```
def foo():
    return 1
# done
```

## Steps

- `dedent`: Remove the common leading indent

## Tags

`whitespace` `indent` `format` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
