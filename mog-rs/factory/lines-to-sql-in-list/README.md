# Wrap lines into a SQL IN-list

Turn lines into IN ('a', 'b', 'c')

Wrap the whole set of lines into a single structured snippet: quote each line and join them into a SQL IN (...) list. Change the header/item/separator/footer to emit a JSON array, a function call, or any other bracketed list instead.

## Run

```
mog -m lines-to-sql-in-list <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apple
banana
cherry
```

Output:

```
IN ('apple', 'banana', 'cherry')
```

## Pipeline

- `format_list`: Wrap the lines into a SQL IN-list

## Tags

`prefix` `format` `sql` `list` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
