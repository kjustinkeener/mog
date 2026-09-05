# redact-pii

Redact common PII and secrets with placeholders

Redact shaped PII and secrets (emails, SSNs, credit cards, phones, IPs, AWS/Stripe keys) with [token] placeholders, then assert no obvious secret or SSN pattern survived. Names need NER and are NOT redacted.

## Run

```
mog -m redact-pii <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Contact: jane.doe@example.com or 555-123-4567
SSN 123-45-6789, card 4111 1111 1111 1234
Server 192.168.1.10, key AKIAABCDEFGHIJKLMNOP
stripe sk_live_abcdefghij1234567890
Bob Smith called
```

Output:

```
Contact: [email] or [phone]
SSN [ssn], card [card]
Server [ip], key [secret]
stripe [secret]
Bob Smith called
```

## Pipeline

- `replace_regex`: emails
- `replace_regex`: US SSN
- `replace_regex`: credit card (16 digits)
- `replace_regex`: AWS access key
- `replace_regex`: Stripe live key
- `replace_regex`: IPv4 address
- `replace_regex`: phone (US-ish)
- `assert`: prove no obvious secret/PII survived

## Tags

`redact` `security` `pii` `privacy` `secrets`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
