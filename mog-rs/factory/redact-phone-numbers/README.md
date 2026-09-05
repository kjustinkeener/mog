# Redact phone numbers

Redact phone numbers to [PHONE]

Replace common phone-number formats with [PHONE] so a log, transcript, or message can be shared without exposing numbers. Matches US-style and international shapes: an optional +country code, an optional (area) in parentheses, and digit groups separated by spaces, dots, or hyphens. Long digit sequences that only look like phone numbers may be caught too.

## Run

```
mog -m redact-phone-numbers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Call me at (555) 123-4567 or +1 555.987.6543.
Office: 555-000-1234. No number on this line.
```

Output:

```
Call me at [PHONE] or [PHONE].
Office: [PHONE]. No number on this line.
```

## Pipeline

- `replace_regex`: Replace phone-number-shaped sequences with a placeholder

## Tags

`redact` `pii` `contacts` `privacy` `security`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
