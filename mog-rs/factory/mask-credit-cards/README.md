# Mask credit card numbers

Mask 16-digit credit card numbers with [CARD]

Replace 16-digit credit-card-shaped numbers (four groups of four, optionally separated by spaces or hyphens) with [CARD], so a log or message can be shared without exposing card numbers. Matches by shape, not by Luhn check, so some 16-digit sequences that are not cards would also be masked.

## Run

```
mog -m mask-credit-cards <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Paid with 4111 1111 1111 1111 today.
Card 4012-8888-8888-1881 on file. Order #12345.
```

Output:

```
Paid with [CARD] today.
Card [CARD] on file. Order #12345.
```

## Steps

- `replace_regex`: Replace 16-digit card-shaped numbers with a placeholder

## Tags

`redact` `pii` `privacy` `security`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
