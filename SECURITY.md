# Security Policy

## Supported versions

Mog is pre-1.0. Security fixes are applied to the latest release only. There is
no back-porting to older tags.

## Reporting a vulnerability

Please report suspected vulnerabilities privately, not in a public issue.

Use GitHub's private reporting: open the repository's **Security** tab and click
**Report a vulnerability** (GitHub Security Advisories). This keeps the report
confidential until a fix is available.

Please include:

- what the issue is and the impact you see,
- steps or a minimal `.mog` script and input that reproduce it,
- the mog version (`mog --version`) and your platform.

## What to expect

This is a small, best-effort project maintained by one person. Reports are
triaged when time allows; there is no guaranteed response time. Once a valid
issue is confirmed, a fix and a new release are prioritized over other work.

## Scope

Mog reads a `.mog` script and input files and writes transformed output. It runs
locally with the permissions of the invoking user and makes no network calls in
its default build. Things that are in scope: a crafted `.mog` or input causing
memory-unsafety, a crash on trusted input, a path-traversal or write outside the
intended targets, or a composition (`run_mog`) escaping its confinement. Things
that are out of scope: running an untrusted `.mog` that does exactly what its
steps say (a `.mog` is code you are choosing to run), and denial-of-service from
deliberately pathological input on an unbounded run.
