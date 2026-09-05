# Redact email addresses

Redact email addresses to [EMAIL]

Replace every email address with [EMAIL] so a log, config, or message can be shared without exposing addresses. Matches the common local@domain.tld shape. For a broader PII sweep (keys, phone numbers, IPs too) use the redact-pii recipe.

## Run

```
mog -m redact-emails <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Contact ada.lovelace@example.com or bob+test@mail.co.uk today.
No address on this line.
```

Output:

```
Contact [EMAIL] or [EMAIL] today.
No address on this line.
```

## Pipeline

- `replace_regex`: Replace email addresses with a placeholder

## Tags

`redact` `pii` `email` `privacy` `security`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
