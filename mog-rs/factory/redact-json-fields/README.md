# Redact JSON fields

Mask sensitive fields in JSON logs

Mask sensitive fields in a JSONL log stream (one object per line) by setting them to [REDACTED] so logs can be shared without leaking secrets. Demo redacts password and token; edit the step paths for your fields (dotted paths like user.email work). The field is added if a line lacks it, so best on a uniform stream where the field is present. For a single JSON object, drop the jsonl option.

## Run

```
mog -m redact-json-fields <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"user": "alice", "password": "s3cret", "token": "abc123", "action": "login"}
{"user": "bob", "password": "hunter2", "token": "def456", "action": "logout"}
```

Output:

```
{"user":"alice","password":"[REDACTED]","token":"[REDACTED]","action":"login"}
{"user":"bob","password":"[REDACTED]","token":"[REDACTED]","action":"logout"}
```

## Pipeline

- `json_set`: Redact password
- `json_set`: Redact token

## Tags

`json` `jsonl` `redact` `pii` `secrets` `privacy`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
