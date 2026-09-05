# Pseudonymize an id/email column with a stable hash

Replace an id/email column with a stable salted hash

Replace the first column (an email or user id) with a stable, salted hash so the data stays joinable across runs while the real identifier is gone. The same value always maps to the same token (a 16-char hex digest with a 'user_' prefix), so you can still group or join on it; set an explicit salt to make tokens unguessable but stable. Other columns pass through untouched; empty cells stay blank. Input is treated as headerless (every line's field is hashed): scope the step or drop a header row first if your input has one.

## Run

```
mog -m pseudonymize-id-column <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
alice@example.com,2024-01-01,login
bob@example.com,2024-01-01,login
alice@example.com,2024-01-02,logout
```

Output:

```
user_9b7d3cda87eb00e8,2024-01-01,login
user_3e85f808902dba78,2024-01-01,login
user_9b7d3cda87eb00e8,2024-01-02,logout
```

## Pipeline

- `hash_field`: Hash column 1 to a 16-char prefixed token, salted for stability

## Tags

`data` `encode` `privacy` `redact` `pii`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
