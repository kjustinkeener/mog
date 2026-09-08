# Round numbers to N places

Round each number to a fixed number of decimals

Round every number in scope to a fixed number of decimal places (default 2 here). Non-numeric text is untouched. Useful for tidying measurement or currency columns before display.

## Run

```
mog -m round-decimals <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
3.14159
2.5
total 19.999 units
```

Output:

```
3.14
2.50
total 20.00 units
```

## Steps

- `round_numbers`: Round each number to 2 decimals

## Tags

`text` `numbers` `math` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
