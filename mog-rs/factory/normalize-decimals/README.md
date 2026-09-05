# Normalize decimal separators (EU to US)

Convert European number format to US decimals

Rewrite numbers from the European convention (1.234,56: dot thousands, comma decimal) to the US convention (1,234.56: comma thousands, dot decimal). Only numbers that carry a separator are changed, so bare integers such as years are left alone. Change the 'from'/'to' options for other conventions (us, eu, swiss, space_comma, space_dot) or set 'grouping' false to drop thousands separators.

## Run

```
mog -m normalize-decimals <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Invoice total: 1.234.567,89 EUR
Unit price 12,50, qty 2024
Subtotal 999,00
```

Output:

```
Invoice total: 1,234,567.89 EUR
Unit price 12.50, qty 2024
Subtotal 999.00
```

## Pipeline

- `decimal_separator_normalize`: EU number format to US number format

## Tags

`numbers` `unicode` `csv` `normalize` `finance`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
