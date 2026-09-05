# Redact IPv4 addresses

Redact IPv4 addresses to [IP]

Replace every IPv4 address (like 192.168.1.10) with [IP] so logs or configs can be shared without exposing addresses. Matches any dotted quad without validating each octet is 0-255, so a version-like 1.2.3.4 is also masked.

## Run

```
mog -m redact-ipv4 <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Client 192.168.1.10 connected to 10.0.0.1 at port 8080.
Gateway is 172.16.254.1.
```

Output:

```
Client [IP] connected to [IP] at port 8080.
Gateway is [IP].
```

## Pipeline

- `replace_regex`: Replace dotted-quad addresses with a placeholder

## Tags

`redact` `network` `privacy` `security` `log`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
