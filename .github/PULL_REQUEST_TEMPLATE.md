<!--
Thanks for contributing to Mog. Please keep a PR to one logical change.
See CONTRIBUTING.md for the two lanes (recipes vs. engine) and the bar for each.
-->

## What and why

<!-- What does this change, and why? Link an issue if there is one. -->

## Type of change

- [ ] Recipe (a `.mog` plus its `TestInput` / `TestExpectedOutput` fixture)
- [ ] Engine bug fix (with a test)
- [ ] Docs
- [ ] Other engine change (please confirm it was discussed in an issue first)

## Checklist

- [ ] `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, and
      `cargo test` pass locally.
- [ ] Added or updated tests (a fixture for a recipe, a failing-then-passing
      test for a fix).
- [ ] Updated `CHANGELOG.md` under `Unreleased` if the change is user-visible.
- [ ] No em-dash (U+2014) in committed content.

### For a recipe, additionally

- [ ] Ships a representative, edge-heavy fixture (not gamed to merely pass).
- [ ] Description matches the actual behavior and states any limits plainly.
- [ ] Flags rather than fakes lossy output (`flag_matching` / `assert`).
- [ ] Not redundant with an existing recipe; one clear purpose; good tags.
