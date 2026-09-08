# Redact Secrets

Mask common secrets and PII for sharing

Mask common secrets and PII before sharing a log or config: emails, IPv4 addresses, AWS access keys, Stripe-style provider keys (sk_live_/pk_live_/rk_test_ ...), key=value secrets (api_key/secret/password/token/key/access_key), Bearer tokens, and JWTs. A convenience pass for sharing, NOT a security guarantee: it can over-redact (any 4-part dotted number reads as an IP) and under-redact (names, phone numbers, and unknown secret shapes slip through).

## Run

```
mog -m redact-secrets <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Contact: alice@example.com for access
Server 10.0.12.34 refused connection
AWS key AKIAIOSFODNN7EXAMPLE in config
export API_KEY=sk_live_abc123DEF456
password: hunter2
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjMifQ.abc-DEF_123
token = eyJ0eXAiOiJKV1QifQ.payloadpart.sigpart
GET /api key=sk_live_9f8a7b6c5d4e3f2a done
publishable pk_live_51H8xYzAbCdEf appears here
version 1.2.3.4 build
just a normal line with no secrets
```

Output:

```
Contact: [EMAIL] for access
Server [IP] refused connection
AWS key [AWS_KEY] in config
export API_KEY=[REDACTED]
password: [REDACTED]
Authorization: Bearer [TOKEN]
token = [REDACTED]
GET /api key=[REDACTED] done
publishable [REDACTED] appears here
version [IP] build
just a normal line with no secrets
```

## Steps

- `replace_regex`: Emails
- `replace_regex`: IPv4 addresses
- `replace_regex`: AWS access key IDs
- `replace_regex`: Provider secret keys (Stripe-style sk/pk/rk live/test)
- `replace_regex`: key=value secrets (keep the key, mask the value)
- `replace_regex`: Bearer tokens (keep the scheme, mask the token)
- `replace_regex`: Standalone JWTs

## Tags

`redact` `secrets` `pii` `email` `paste` `log`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
