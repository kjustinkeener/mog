# Roadmap and scope

Mog is a utility, not a platform. It does one thing: apply a deterministic,
reviewable text transform to many lines or files exactly, from a reusable `.mog`
script, with a preview and a backup. Keeping that scope tight is what makes it
trustworthy and small. This document says what the project is for and, just as
importantly, what it will not become.

## What mog is for

- Repeatable find/replace and text-transform pipelines authored as `.mog` files.
- Literal and regex replacement, line/whitespace/case/EOL operations, encode and
  escape, and reading-oriented format conversion (JSON/CSV/YAML/TOML/XML and
  friends).
- Region, block, field, and character-range scoping; two-input compares;
  redaction and detectors; composition of scripts with `run_mog`.
- A curated, tested library of mogs for common jobs, and an agent-facing
  interface so tools can discover and run those mogs.

New capability is added when it is a genuinely new, composable primitive, or a
mog that beats the target user's best alternative. Most feature requests are
better met by composing existing actions than by growing the engine.

## Non-goals

These are deliberate. They are not "not yet"; they are "not here, on purpose."

- **No embedded AI, network calls, or non-determinism in the default engine.**
  The same input and script always produce the same output. The default build
  makes no network calls.
- **Not a query language.** The format readers exist to transform text, not to
  query or restructure documents the way `jq` or a real parser would. Structural
  JSON/YAML key reordering and deeply nested document rewriting are out; mog will
  flag what it cannot safely handle rather than fake it.
- **No plugin system, custom/external/WASM actions, or scripting language.**
  Extensibility is composition of built-in actions, not arbitrary user code. This
  keeps every run auditable and the binary small.
- **No enterprise platform surface.** No private registry, policy engine, access
  control, audit subsystem, or telemetry (not even opt-in).
- **No distributed or cluster execution.** Mog is a local single-binary tool.
- **Not a general-purpose editor or IDE.** A JSON Schema (`mog --schema`) covers
  editor validation; there is no language server.

## How direction is set

The engine's direction is maintainer-led (see [CONTRIBUTING.md](CONTRIBUTING.md)).
Bug fixes with a test are always welcome. Mogs are the open contribution lane.
For a larger engine change, open an issue to discuss before writing it, so effort
is not wasted on something outside scope.

## Status

Mog is pre-1.0 but feature-complete for its intended scope: the action catalog
and mog library are broad, and the whole suite is tested in CI on Linux,
macOS, and Windows. Pre-1.0 means the `.mog` format and CLI are stable in
practice but not yet formally frozen; any breaking change will be called out in
the [changelog](CHANGELOG.md).
