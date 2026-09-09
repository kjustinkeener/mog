# Contributing to Mog

Thanks for your interest. Mog is a small, deterministic text-transform engine,
and it stays useful by staying focused. This guide explains what kinds of
contributions fit, and how the project is structured so you know where your
change belongs.

Mog is provided as-is and maintained best-effort by one person. Issues and pull
requests are welcome, but there is no guaranteed response time. Please be patient
and kind.

## Two lanes

Mog has two contribution surfaces with deliberately different bars.

### 1. Mogs (the `factory/` library): contributions welcome

Mogs are `.mog` scripts plus a `TestInput` / `TestExpectedOutput` fixture
pair. They are the easiest and most welcome way to contribute, because each
mog self-verifies through its golden fixture: `mog market test` (or the test
suite) runs the mog over its input and checks the output byte-for-byte, so a
mog either works exactly or it fails loudly.

A mog earns a place when it beats the best alternative its target user
actually has:

- For technical / shell-fluent users, that means it lives in the moat: region
  or block operations, dialect conversion, stateful or multi-line work, a fiddly
  tested regex pipeline you would not want to re-derive, or a reusable artifact
  you commit and rerun. A mog that is a one-line `sed` / `tr` / `re.sub`
  equivalent does not earn a slot for this audience.
- For non-CLI users, an everyday cleanup (dedupe, sort, case-fix, trim a list)
  can still earn a slot, because the win is mog's preview, backup, and "nothing
  silently dropped or altered."

Please also make sure a mog:

- ships a representative fixture (edge-heavy, not gamed to merely pass),
- has a description that matches what it actually does, and states its limits
  plainly (for example "names need review", "multi-line quoted CSV unsupported"),
- flags rather than fakes: where it cannot guarantee a result, it uses
  `flag_matching` on the residue and/or an `assert` post-condition instead of
  silently emitting lossy output,
- is not redundant with an existing mog (a better version of an existing
  mog is a revision, not a new slot),
- has one clear purpose, readable steps, and good tags for search.

### 2. The engine (`mog-rs/src/`): maintainer-led

The engine's scope is intentionally narrow, and its direction is set by the
maintainer. Bug fixes with a failing test are always welcome. For anything
larger, a new action, a flag, or a behavior change, please open an issue to
discuss it first rather than sending an unsolicited large PR. Most "mog should
also do X" requests are better served by composing existing actions or by a mog.

## Development

```
cd mog-rs
cargo build
cargo test
```

Before opening a PR, please make sure:

```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

all pass. CI runs the same checks on Linux, macOS, and Windows.

> If you have checked the repo out inside a synced folder (Google Drive,
> Dropbox), point Cargo's build output elsewhere so the sync client does not
> thrash on `target/`:
> ```
> export CARGO_TARGET_DIR="$HOME/.cache/mog-target"
> ```

## Pull requests

- Keep a PR to one logical change.
- Include a test: a fixture for a mog, a failing-then-passing test for an
  engine fix.
- Match the surrounding code and comment style.
- Update `CHANGELOG.md` under the `Unreleased` heading if the change is
  user-visible.

## Commit hygiene

Please avoid the em-dash character (U+2014) in committed content; a plain hyphen,
comma, colon, or parentheses reads the same and avoids tooling that chokes on it.

By contributing, you agree that your contributions are licensed under the
project's [MIT license](LICENSE).
