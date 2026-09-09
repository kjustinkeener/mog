# Mog

Mog scripts repeatable text search/replace pipelines. Instead of a series of
one-off find/replaces, you author a **`.mog` file** (a JSON list of ordered
steps) and run it over one or many files. Think of it as **`sed` with a
human-readable JSON script**: full-word action and option names, literal *and*
regex replace, plus line/whitespace/case/EOL transforms, all in one ordered pass.

mog is a **cross-platform command-line tool for batch-processing files**. The
engine and CLI live in [`mog-rs/`](mog-rs) (Rust).

## Quick start

```
cargo build --release           # from mog-rs/, or grab a release binary
mog -m convert.mog "src/**/*.sql" --in-place --backup .bak -j 8
```

`mog` takes a `.mog` script plus file paths or globs, and can write in place,
into an out-dir, or to stdout; preview with `--dry-run`/`--diff`; process in
parallel. Full CLI usage and the complete action catalog are in
[`mog-rs/README.md`](mog-rs/README.md).

## The `.mog` format

A script is **strict JSON**: an ordered `steps` array, each step an `action` plus
an `options` object. Comments and trailing commas are a parse error; use the
optional `description` field on the pipeline and on each step for human
annotation.

```json
{
  "name": "Tidy list",
  "description": "Normalize a scratch list into a clean, sorted set of lines.",
  "steps": [
    { "action": "trim_whitespace", "description": "Strip leading/trailing spaces." },
    { "action": "remove_empty_lines", "options": { "include_whitespace": true } },
    { "action": "sort_lines", "options": { "ignore_case": true } },
    { "action": "to_proper" }
  ]
}
```

An optional top-level `constants` object defines named string values, and any
`{{name}}` placeholder in a step's option strings is expanded from it **at load
time** (before any action runs, so it never collides with regex `$1` / `${1}`
backrefs). Override or add values per run with `--define name=value` (`-D`,
repeatable); CLI values win over the script's defaults. A script with no
`constants` and no `--define` is left untouched, so literal `{{…}}` text stays
safe to find/replace.

```json
{
  "constants": { "schema_prefix": "public." },
  "steps": [
    { "action": "replace", "options": { "find": "dbo.", "replace_with": "{{schema_prefix}}" } }
  ]
}
```

Actions cover literal/regex/extended replace (with `$1` backrefs plus `$#` /
`$lineno` / `$uuid` per-occurrence tokens), line operations (filter; sort, including
`sort_ip` / `sort_versions` / `sort_imports`; dedupe, number, join…),
blank/whitespace, EOL conversion, case conversion, encode/escape (URL, HTML,
base64, regex, shell, SQL, CSV, `normalize_url`), per-line affixing, and format
conversion for the modern data stack (JSON/CSV/YAML/TOML/XML/logfmt/INI and more,
reading-only, not a query language). Beyond one input there are two-input
compares (`diff`, `reconcile`, `intersect`, `subtract`) against a second source,
`detect_secrets` / `detect_pii` and redaction, an `assert` gate, mail-merge from
external `sources`, and composing scripts with `run_mog`. A step can be scoped to
a block, a line range, a delimited **field**, or a **character range**, and the
pipeline can pin its own `output_encoding`. Browse
[`mog-rs/factory/`](mog-rs/factory) for the bundled mog library (each mog
is its own folder with a tested input/output example), including an MSSQL to
PostgreSQL schema conversion, and run `mog --list-actions --json` for the full,
current catalog.

## Repository layout

| Path | What it is |
|------|------------|
| `mog-rs/` | The Rust engine (`[lib]`) and batch CLI (`mog`). The product. |
| `mog-rs/factory/` | The bundled mog library: one folder per mog (`<name>/<name>.mog` + a tested `tests/` example). |
| `docs/` | The `.mog` format standard and the marketplace guide. |
| `studio/` | The desktop authoring app (Tauri + Svelte) over the same engine. |

A desktop **studio** for authoring `.mog` files is being built separately with
Tauri + Svelte over the same Rust engine.

## Requirements

Rust (stable). Prebuilt binaries for Linux, macOS, and Windows are attached to
tagged releases via CI.

## Safety

mog changes files in place, so it is built to be safe to run. Every run can be
previewed first with `--dry-run` or `--diff`; `--backup` writes a copy before
overwriting; and every transform is deterministic, so the same script and input
always produce the same output. A guardrail refuses a surprising bulk change
unless you pass its named override. mog is provided as-is, without warranty (see
[LICENSE](LICENSE)).

## Contributing

Mogs are the most welcome contribution, and each one self-verifies through a
golden test fixture. See [CONTRIBUTING.md](CONTRIBUTING.md) for the two
contribution lanes (mogs and the engine), including the project's scope and
non-goals.

## License

Released under the [MIT License](LICENSE).
