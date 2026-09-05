# Multiply every number

Scale each numeric token by a constant

Apply a constant arithmetic operation to each number in scope. This recipe multiplies by 1.1 and formats to 2 decimals (a 10% uplift). Change the op/by/places via -D or by editing the step to add, subtract, or divide instead.

## Run

```
mog -m scale-numbers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
100
250
subtotal 19.99
```

Output:

```
110.00
275.00
subtotal 21.99
```

## Pipeline

- `arithmetic`: Multiply each number by 1.1, 2 decimals

## Tags

`text` `numbers` `math` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
