# Redact UUIDs

Redact UUIDs to [UUID]

Replace every UUID (8-4-4-4-12 hex, like 550e8400-e29b-41d4-a716-446655440000) with [UUID] so identifiers in a log or config can be shared without leaking them. Case-insensitive.

## Run

```
mog -m redact-uuids <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
request 550e8400-e29b-41d4-a716-446655440000 completed
user F47AC10B-58CC-4372-A567-0E02B2C3D479 logged in
no id on this line
```

Output:

```
request [UUID] completed
user [UUID] logged in
no id on this line
```

## Steps

- `replace_regex`: Replace UUIDs with a placeholder

## Tags

`redact` `identifier` `privacy` `security` `log`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
