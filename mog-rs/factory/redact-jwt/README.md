# Redact JWT tokens

Redact JWT tokens to [JWT]

Replace JSON Web Tokens (the eyJ... header.payload.signature shape, three dot-separated base64url segments) with [JWT] so an auth log or pasted request can be shared without leaking a bearer token.

## Run

```
mog -m redact-jwt <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U
plain header on this line
```

Output:

```
Authorization: Bearer [JWT]
plain header on this line
```

## Pipeline

- `replace_regex`: Replace JWTs with a placeholder

## Tags

`redact` `secrets` `security` `privacy`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
