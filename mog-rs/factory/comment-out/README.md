# Comment Out

Comment out every line with a marker

Prefix every line with a comment marker (default '# '; override with -D marker='// '). Comments out a pasted block. Language-agnostic: set the marker to match (# shell/python/yaml, // c/js, -- sql/lua, ; ini).

## Run

```
mog -m comment-out <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
def hello():
    return 1

x = 2
```

Output:

```
# def hello():
#     return 1
# 
# x = 2
```

## Pipeline

- `prefix_lines`: Prefix each line with the comment marker

## Tags

`comment` `prefix` `fragment`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
