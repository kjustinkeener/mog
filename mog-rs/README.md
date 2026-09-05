# mog (Rust)

A fast, single-binary, cross-platform CLI for **batch text processing**. It
applies a `.mog` script (an ordered pipeline of find/replace and text-transform
steps) to one or more files.

Mog is a single self-contained binary with no runtime dependencies. It runs the
sample `sqlserver-tables-to-postgres.mog` over the sample input to produce a full SQL
Server to PostgreSQL schema conversion. The `.mog` file format and the full
action catalog are documented in the [repository README](../README.md).

## Install

Download a prebuilt binary from the project's GitHub Releases (Linux, macOS, and
Windows, x64 and arm64), or build from source:

```
cargo build --release
```

The binary is at `target/release/mog` (`mog.exe` on Windows).

## Usage

```
mog -m <SCRIPT.mog> [OPTIONS] [INPUTS]...

  [INPUTS]            Zero or more file paths or glob patterns (e.g. "src/**/*.sql").
                      Omit, or pass "-", to read STDIN and write STDOUT (filter mode).
  -m, --script        Path to the .mog script
  -i, --in-place      Overwrite each input file with its transformed output
      --backup <SUF>  Before an in-place write, save a backup with this suffix
  -o, --out-dir <D>   Write outputs into directory D, mirroring input names
      --dry-run       Report what would change without writing anything
      --check         Report whether any file WOULD change; exit 1 if so (CI gate)
      --diff          Print a unified diff (before -> after) per changed file, to stdout
      --explain       Explain what the pipeline does in plain language, then exit
      --sample <N>    Preview the transform on the first N lines of one input, to stdout
      --stream        Process STDIN in bounded (constant) memory; streamable pipelines only
      --parallel[=N]  Transform one large file across N cores (default: all); byte-identical
      --files-from <F> Read extra input paths (one per line) from F, or stdin when "-"
      --lossy         Read input as UTF-8 lossily (invalid bytes replaced) instead of erroring
      --exclude <G>   Drop any input file whose path matches this glob (repeatable)
  -j, --jobs <N>      Process files in parallel across FILES (default: available parallelism)
      --overwrite     Allow overwriting existing files in the output directory
```

With a single input and no output flag, the transformed text is written to
**stdout** (the per-file summary goes to stderr, so redirects stay clean).
Multiple inputs require an output mode (`--in-place` or `--out-dir`), except
under `--dry-run`.

### Filter mode (stdin -> stdout)

With **no input arguments**, or a single input of `-`, mog reads STDIN, runs the
pipeline, and writes the result to STDOUT (the summary goes to stderr). This lets
mog work as a Unix filter:

```
cat notes.txt | mog -m tidy.mog
```

### Previewing changes with `--diff`

`--diff` prints a plain unified diff (before -> after) for each changed file to
stdout, instead of the transformed content. It pairs naturally with `--dry-run`
to preview a run without writing anything:

```
mog -m convert.mog "src/**/*.sql" --dry-run --diff
```

### Big files: `--stream`, `--parallel`, `--sample`

For huge or unbounded input, three options avoid loading and rewriting the whole
file at once. All three are **byte-identical** to the normal path.

- `--stream` processes STDIN in **constant memory** (reads newline-delimited
  blocks, transforms, emits as it goes). It requires an all-*streamable* pipeline
  (per-line, EOL-preserving actions) and `--encoding utf-8`, and errors with the
  offending step otherwise. Also works on a single file to stdout or in place.

  ```
  cat huge.log | mog -m tidy.mog --stream --encoding utf-8
  ```
- `--parallel[=N]` transforms one large file across CPU cores (split at line
  boundaries, reassembled in order). Streamable pipelines only; others run
  sequentially. This is *intra-file*; `-j/--jobs` parallelizes across files.
- `--sample N` previews the transform on just the first N lines (read-bounded, so
  it stays fast on a multi-GB file), to stdout. Pair with `--diff` to see the
  sample as a before/after diff.

Which actions are streamable/parallel-capable is reported by `mog --list-actions
--json` (the `streamable` flag) and per action by `mog --describe <action> --json`.

### CI pipelines: `--check`, `--in-place`, `--files-from`

mog fits the `prettier --check` / `gofmt` shape. `--check` reports whether a
transform *would* change files and exits non-zero if so (fails the build);
`--in-place` (`-i`) applies it. `--files-from <FILE|->` reads input paths (one per
line) from a file or stdin, for changed-file runs:

```
git diff --name-only | mog -m fix.mog --check      --files-from -   # gate
git diff --name-only | mog -m fix.mog --in-place   --files-from -   # apply
```

Exit codes: `0` clean, `1` would-change or a flag/assert fired, `2` error. See
[`packaging/`](../packaging) for a Dockerfile, a GitHub Action, and a GitLab
template. `mog --explain -m recipe.mog` prints a plain-language summary of a
pipeline.

### Guardrails always have an override

Every guardrail in mog is either **off by default** (you opt in) or ships with an
**explicit escape hatch**: a guard never blocks a run with no way out, and the
refusal message names the override. This is a standing design rule: any new
guardrail must offer a way to lift it.

| Guardrail | Default | Override |
| --- | --- | --- |
| `--timeout <secs>` | off | omit it, or raise the value |
| `--max-steps <n>` | off | omit it, or raise the value |
| `--max-input-bytes <n>` | off | omit it, or raise the value |
| `--max-shrink <pct>` (data-loss guard) | off | omit it, or raise the value |
| `--refuse-binary` | off | omit it |
| report sampling caps (diff size/count) | on | `--report-full`, or `--report-limit key=n` (`0` = unlimited) |
| `pivot` `max_columns` (runaway-grid guard) | 1000 | set `max_columns` to `0` to lift the cap |

### Character encodings

mog decodes input to text on the way in and encodes it back on the way out, like
a text editor's Encoding menu.

`--encoding <NAME>` picks the **input** encoding. The default `auto` sniffs a BOM
(UTF-8, UTF-16 LE/BE), else reads UTF-8, else falls back to Windows-1252
("ANSI"), so a UTF-16 file (for example an SSMS "Generate Scripts" export, which
is UTF-16 by default) just works. Force a specific one with a label like
`utf-16le`, `windows-1252`/`ansi`, `shift_jis`, or `iso-8859-1`.

`--output-encoding <NAME>` picks the **output** encoding. The default `preserve`
re-emits whatever the input was (including its BOM); or force `utf-8`,
`utf-8-bom`, `utf-16le`, `utf-16be`, `ansi`/`windows-1252`, or another label.

```
# Convert a UTF-16 SSMS export straight to a UTF-8 Postgres script:
mog -m sqlserver-tables-to-postgres.mog --output-encoding utf-8 < schema.sql > schema.pgsql
```

`--lossy` only affects a strict `--encoding utf-8` request: invalid bytes become
the replacement character instead of aborting. (In `auto` mode a non-UTF-8 file
is decoded as Windows-1252 rather than replaced.) `--exclude <GLOB>` (repeatable)
drops any matched input file, e.g. skip vendored trees:

```
mog -m fix.mog "src/**/*.js" --in-place --exclude "**/vendor/**" --exclude "*.min.js"
```

### Examples

Preview a conversion across a tree without touching anything:

```
mog -m convert.mog "src/**/*.sql" --dry-run
```

Convert every matching file in place, in parallel, keeping `.bak` backups:

```
mog -m convert.mog "src/**/*.sql" --in-place --backup .bak -j 8
```

Write results into a separate directory:

```
mog -m convert.mog "in/*.txt" -o out
```

Transform one file to stdout:

```
mog -m tidy.mog notes.txt > tidy.txt
```

## The `.mog` format

A `.mog` file is **strict JSON**: an ordered list of steps, each an `action`
plus an `options` object. Comments and trailing commas are a parse error; use
the `description` field on the pipeline and on each step for human annotation.

A pipeline may also carry an optional top-level `summary`: a short, one-line
human blurb, kept distinct from the longer `description` (which stays the full,
agent-facing explanation). `mog market list` / `market search` show the `summary`
in their human output (falling back to a truncated `description` when a recipe has
none), while `--json` (and therefore the MCP server and Mog Studio) returns the
full `description` and now also includes `summary`.

```json
{
  "name": "Tidy List",
  "steps": [
    { "action": "trim_whitespace" },
    { "action": "remove_empty_lines", "options": { "include_whitespace": true } },
    { "action": "sort_lines", "options": { "ignore_case": true } },
    { "action": "to_proper" }
  ]
}
```

### Constants (`{{name}}` placeholders)

An optional top-level `constants` object defines named string values. Any
`{{name}}` placeholder in a step's option strings (or its `only_lines_matching`
/ `except_lines_matching` scope) is expanded from `constants` **at load time**,
before any action runs, so placeholders never collide with regex `$1` / `${1}`
backrefs. Override or add values per run with `--define name=value` (repeatable);
CLI values win over the script's defaults.

```json
{
  "constants": { "schema_prefix": "public." },
  "steps": [
    { "action": "replace", "options": { "find": "dbo.", "replace_with": "{{schema_prefix}}" } }
  ]
}
```

```bash
mog -m convert.mog -D schema_prefix=app. < in.sql   # override the default
```

Interpolation is active whenever a script defines any `constants` (or you pass
`--define`); when active, an undefined `{{name}}` is an error, so typos surface
immediately. A script with **no** constants and no `--define` is left completely
untouched, so literal `{{...}}` text stays safe to find/replace. Constant values
may be strings, numbers, or bools (all stringified). A `{{...}}` run that is not
a bare identifier is always left verbatim.

See the [repository README](../README.md) for the base list of actions
(replace/regex, line operations, blank operations, EOL conversion, case
conversion, inserts) and their options. The actions below are new in this Rust
port.

## New actions

Every action name and option below is a full, descriptive English word: `.mog`
files are meant to read like plain-English config, not cryptic sed/awk. All
line-based actions preserve the file's EOL style and trailing-newline state.

> The tables below highlight selected families. The action catalog has since grown
> well beyond them: format converters (JSON/CSV/YAML/TOML/XML/logfmt/INI), two-input
> compares (`diff`, `reconcile`, `intersect`, `subtract`), detectors and redaction
> (`detect_secrets`, `detect_pii`), data reshaping, the escape family, and the
> `sort_ip` / `sort_versions` / `sort_imports` sorts among them. **`mog --list-actions
> --json` is the authoritative, always-current catalog** (add `--describe <action>
> --json` for one action's full parameter list, complexity, streamability, and
> reversibility).

### Extended (Notepad++-style) replace

| Action | Description | Options |
| --- | --- | --- |
| `replace_extended` | A **literal** (non-regex) find/replace that interprets escape sequences in `find` and `replace_with`, matching Notepad++'s "Extended" search mode. | `find` (string, required), `replace_with` (string, default ""), `ignore_case` (bool, default false) |

Escape sequences are decoded in **both** `find` and `replace_with` before the
literal replace runs: `\n` (LF), `\r` (CR), `\t` (tab), `\0` (NUL), `\\` (a
single backslash), `\xHH` (a 2-hex-digit byte, e.g. `\x41` -> `A`), and
`\uHHHH` (a 4-hex-digit Unicode scalar; `\u` followed by `00e9` yields `é`). Any other escape
(an unknown letter like `\q`, an invalid `\x`/`\u`, or a lone trailing `\`) is
left literal: the backslash and the following character are kept unchanged.

To pair a **regex** find with those escapes in the replacement, set
`decode_replacement: true` on `replace_regex` / `replace_regex_multiline`: the
same escapes are decoded in `replace_with` before `$1` / `${name}` expansion, so
you can, for example, split on a regex and emit a real newline. (Without it, the
replacement is taken verbatim, so a real newline must be a literal newline in the
JSON string.)

This is the tool to reach for when you need to match or insert real control
characters (newlines, tabs) as part of an otherwise literal search. For example,
collapsing a `GO` batch separator that sits between two lines:

```json
{
  "steps": [
    { "action": "replace_extended",
      "options": { "find": "GO\r\n\r\nGO", "replace_with": "GO" } }
  ]
}
```

Here `\r\n` decodes to a real CRLF, so the step matches the two `GO` lines
separated by a blank line and collapses them into a single `GO`.

### Positional replace

| Action | Description | Options |
| --- | --- | --- |
| `replace_nth` | Replace only the Nth match of a regex `find`. `n` is 1-based; a negative `n` counts from the end (`-1` = last); an out-of-range `n` is a no-op. Occurrences are counted across the whole input, or within a scope span. | `find` (string, required), `replace_with` (string, default ""), `n` (integer, default 1), `ignore_case` (bool, default false) |
| `replace_first` | Replace only the first match (`replace_nth` with `n` = 1). | `find` (required), `replace_with` (default ""), `ignore_case` (bool) |
| `replace_last` | Replace only the last match (`replace_nth` with `n` = -1). | `find` (required), `replace_with` (default ""), `ignore_case` (bool) |

`replace_with` uses the same `$1` / `${name}` substitution as `replace_regex`
(`$$` for a literal `$`), not `\1`.

### Per-occurrence replacement tokens

In `replace_regex` and `replace_regex_multiline`, the replacement string supports
three tokens that expand per match, alongside the usual `$1` / `${name}` backrefs:

| Token | Expands to |
| --- | --- |
| `$#` | the 1-based match number |
| `$lineno` | the 1-based line the match starts on |
| `$uuid` | a fresh RFC-4122 v4 UUID, unique per match |

`$uuid` is deterministic under `--pin-seed` (and `mog --test`, which pins the
clock and RNG) so recipes that stamp UUIDs get stable goldens; without a pinned
seed it is random. Double the leading `$` to keep any of these literal, e.g.
`$$uuid` emits the text `$uuid`.

### Line filtering

| Action | Description | Options |
| --- | --- | --- |
| `keep_lines_matching` | Keep only lines whose text matches `pattern` (a regex). | `pattern` (string, required), `ignore_case` (bool, default false) |
| `remove_lines_matching` | Drop lines whose text matches `pattern`. | `pattern` (string, required), `ignore_case` (bool, default false) |
| `insert_before_matching` | Insert a new line of `text` before every line matching `pattern`. | `pattern` (string, required), `text` (string, default ""), `ignore_case` (bool) |
| `insert_after_matching` | Insert a new line of `text` after every line matching `pattern`. | `pattern` (string, required), `text` (string, default ""), `ignore_case` (bool) |
| `flag_matching` | Annotate lines matching `pattern` with a canonical, detectable marker `<comment> <TAG>(mog): <message>`, for constructs that need human review. | `pattern`, `message` (required), `comment` (required line-comment prefix, e.g. `//` `#` `--`), `tag` (FIXME/TODO/NOTE/HACK/XXX/WARN, default FIXME), `position` (inline/before/after), `ignore_case` |

**Flags.** `flag_matching` emits a fixed `<TAG>(mog): <message>` marker so a
converter can call out what it couldn't fully handle instead of leaving it to
fail downstream. Because the format is guaranteed, the CLI scans the output and
prints a prominent report of every flag (with line numbers) to stderr, and any
tooling (e.g. Mog Studio) can list them the same way. The `(mog)` scope keeps
them distinct from your own `TODO`/`FIXME` comments.

### Region-aware

| Action | Description | Options |
| --- | --- | --- |
| `hoist_from_block` | Within each `block_start`..`block_end` region, lift lines matching `extract` out and re-emit them as statements before/after the block. Templates interpolate the block header's captures as `${bN}` and the extracted line's captures as `${N}`, so relocated statements can carry context (e.g. the table name). Cleans up a dangling trailing comma left in the block. | `block_start`, `block_end`, `extract` (regexes, required), `emit_before` / `emit_after` (templates), `ignore_case` (bool) |
| `for_each_block` | For each `block_start`..`block_end` region, bind the header line's captures as constants (`bind`) and run a sub-`.mog` (`run`) over the block, replacing it with the result. The child sees the header's captures as `{{name}}` placeholders, so per-block edits can use context the block header carries. | `block_start`, `block_end` (regexes, required), `run` (script name/path, required), `bind` (object of `name` -> `${N}` template), `ignore_case` (bool) |

`hoist_from_block` handles the **relocation** case (emit new statements outside
the block); `for_each_block` handles **in-place** per-block edits with the header
context bound as constants, e.g. a child that runs `{{table}}`-aware rules over
one `CREATE TABLE` at a time.

This is the primitive for structural moves a stateless pass can't do, such as
turning MySQL's inline `KEY name (cols)` index definitions into standalone
`CREATE INDEX name ON <table> (cols);` statements after the table:

```json
{ "action": "hoist_from_block",
  "options": {
    "block_start": "CREATE TABLE (\\w+) \\(",
    "block_end": "^\\) ENGINE",
    "extract": "^\\s*(?:FULLTEXT )?KEY (\\w+) \\(([^)]*)\\),?",
    "emit_after": "CREATE INDEX ${1} ON ${b1} (${2});" } }
```

### Per-line affixing

| Action | Description | Options |
| --- | --- | --- |
| `prefix_lines` | Put `text` at the start of every line. | `text` (string, default "") |
| `suffix_lines` | Put `text` at the end of every line. | `text` (string, default "") |
| `wrap_lines` | Put `prefix` before and `suffix` after every line. | `prefix` (string, default ""), `suffix` (string, default "") |
| `indent` | Add leading indentation to every line. | `spaces` (integer, default 4); or `text` (string) to use that exact indent |
| `outdent` | Remove up to `spaces` leading spaces from every line; a leading tab is removed as one level. | `spaces` (integer, default 4) |

### Numbering

| Action | Description | Options |
| --- | --- | --- |
| `number_lines` | Prefix each line with an incrementing number, right-aligned/padded so the columns line up. | `start` (integer, default 1), `separator` (string, default ". ") |

### Numbers and math

Numeric-token transforms. By default each matches every standalone integer or
decimal (`-?\d+(\.\d+)?`); restrict the set with a `find` regex. Because they
honor a step's `scope`, running one under a `field` scope does per-column math on
a delimited file.

| Action | Description | Options |
| --- | --- | --- |
| `arithmetic` | Apply an operation to each number in scope by a constant operand. | `op` (add/subtract/multiply/divide, default add), `by` (number, required), `find` (regex, optional), `places` (integer, optional: format the result to N decimals) |
| `increment_numbers` | Add `by` to each matched number (same matching/formatting rules as `arithmetic`). | `by` (number, default 1), `find` (regex, optional), `places` (integer, optional) |
| `round_numbers` | Round each matched number to `places` decimals. | `places` (integer, default 0), `find` (regex, optional) |
| `pad_numbers` | Left-pad each integer run to a fixed width (a run already at least `width` long is left alone). | `width` (integer, required), `pad` (string, default "0") |

### Sorting and dedupe (records and keys)

Beyond the base `sort_lines`, mog can sort and dedupe by an extracted key, and can
treat blank-line-separated paragraphs as whole records. `sort_lines` itself now
also accepts `key_field` + `key_delimiter` (sort by the Nth delimited field),
`key_regex` (sort by a regex match, or its capture group 1), and `by_frequency`
(order by how often each key occurs; `order: "asc"` flips to ascending count).

| Action | Description | Options |
| --- | --- | --- |
| `sort_blocks` | Sort blank-line-separated paragraph records as whole units (stable). Key semantics mirror `sort_lines`. | `by` (whole_block/first_line/key_regex, default whole_block), `key_regex` (string), `order` (asc/desc, default asc), `ignore_case`, `numeric`, `natural` (all bool) |
| `dedupe_blocks` | Drop duplicate paragraph records, keeping the first (works on non-adjacent duplicates). | `by` (whole_block/first_line/key_regex, default whole_block), `key_regex` (string), `ignore_case` (bool) |
| `dedupe_by` | Drop duplicate lines by an EXTRACTED key, keeping the first per key (across non-adjacent lines). | `key_regex` (string) XOR `key_field` (integer) + `key_delimiter` (string, default TAB), `ignore_case` (bool) |

### Blank / whitespace

| Action | Description | Options |
| --- | --- | --- |
| `squeeze_blank_lines` | Collapse runs of consecutive blank lines down to a single blank line. | `include_whitespace` (bool) treats whitespace-only lines as blank |
| `collapse_whitespace` | Within each line, replace runs of spaces/tabs with a single space (newlines untouched). | (none) |
| `squeeze_spaces` | Collapse runs of 2+ spaces to one, but keep each line's leading indentation and leave tabs alone (unlike `collapse_whitespace`). | (none) |

The literal `replace` action also takes a `whole_word` (bool) option: with it, the
find is matched only as a standalone token (bounded by `\b`), so `datetime` is
replaced but the `datetime` inside `sysdatetime` is not. Handy for swapping type
names or keywords without hand-writing a regex.

### Encoding helpers (whole document)

| Action | Description | Options |
| --- | --- | --- |
| `url_encode` / `url_decode` | Percent-encode / decode the whole text. | (none) |
| `html_encode` / `html_decode` | HTML entity encode / decode of `& < > " '`. | (none) |
| `base64_encode` / `base64_decode` | Base64 the whole text (standard alphabet). | (none) |
| `rot13` | Rotate ASCII letters by 13 (self-inverse: apply twice to restore); other characters untouched. | (none) |
| `hex_encode` / `hex_decode` | Hexlify each byte to two lowercase hex digits (no separators) / unhexlify back to text (errors on odd length or a non-hex character). | (none) |

## Scoped steps

Any step may carry an optional **line scope** so its action runs only on certain
lines. Each matching line is transformed individually (as a one-line document);
non-matching lines pass through unchanged, and the original order and EOL style
are preserved. Scoping is meant for line-wise transforms (replace, case,
prefix/suffix, trim, number); for whole-file actions like `sort_lines` it
degenerates to a near no-op.

The fields sit at the top level of a step, alongside `action`:

| Field | Description |
| --- | --- |
| `only_lines_matching` | Regex: apply the action ONLY to lines that match. |
| `except_lines_matching` | Regex: apply the action only to lines that do NOT match. |
| `match_ignore_case` | Bool: case-insensitive matching for the two fields above. |

```json
{
  "steps": [
    { "action": "to_upper", "only_lines_matching": "error", "match_ignore_case": true },
    { "action": "prefix_lines", "options": { "text": "// " }, "except_lines_matching": "^\\s*$" }
  ]
}
```

### Structured `scope`

For more than a per-line regex, a step may carry a structured `scope` object with
exactly one selector. The line-span selectors run the action once over a
multi-line region; the within-line selectors transform a substring of each line
and splice it back, preserving the rest:

| Selector | Runs the action on… |
| --- | --- |
| `in_block` `{ start, end, ignore_case? }` | each `start`..`end` region (inclusive), once per region, so a block-scoped `sort_lines` sorts within each block. |
| `line_range` `{ from?, to? }` | a 1-based inclusive line range (negatives count from the end; `-1` = last line). |
| `field` `{ index, delimiter? }` | one delimited field of each line (1-based `index`, negative from the end; `delimiter` default `,`). A line with too few fields passes through. |
| `char_range` `{ from?, to? }` | a 1-based inclusive character range of each line (negatives from the end; Unicode-safe). |
| `invert: true` | the complement of the selected line spans (line-span selectors only). |

```json
{
  "steps": [
    { "action": "sort_lines", "scope": { "in_block": { "start": "^BEGIN", "end": "^END" } } },
    { "action": "to_upper",   "scope": { "field": { "index": 2, "delimiter": "," } } }
  ]
}
```

A `scope` object composes with the `only_lines_matching` / `except_lines_matching`
line filters on the same step: the action runs on the intersection of the
scope-selected span and the filter-passing lines. (Previously, a line filter was
silently ignored whenever a `scope` was also set.)

## Composing scripts with `run_mog`

The `run_mog` action loads another `.mog` file and runs its steps inline at that
point in the pipeline. The `file` option is resolved **relative to the directory
of the `.mog` that contains the step**, so scripts can be composed and reused.
Recursion is guarded with a depth limit of 25 (a clear error is raised if a cycle
is hit).

```json
{
  "name": "Full clean",
  "steps": [
    { "action": "run_mog", "options": { "file": "shared/trim.mog" } },
    { "action": "run_mog", "options": { "file": "shared/number.mog" } }
  ]
}
```

## Warehouse SQL conversion

mog ships a family of **warehouse-SQL dialect converters** centered on Snowflake:
`snowflake-tables-to-bigquery` / `bigquery-tables-to-snowflake`, `redshift-tables-to-snowflake`,
`postgres-tables-to-snowflake` / `snowflake-tables-to-postgres`, and
`databricks-tables-to-snowflake` / `snowflake-tables-to-databricks` (run `mog market search
snowflake`). They are **best-effort, minimal-diff** rewrites in mog's regex +
composition lane: identifier requoting, type-keyword maps, safe function renames,
and dialect-specific cleanups (e.g. stripping Redshift `DISTKEY`/`SORTKEY`). They
are deliberately NOT a semantic transpiler, so they never silently reshape
arguments; each recipe's description states its limits, and you review the diff.
That reviewable, comment-preserving minimal diff is the point.

For full semantic transpilation (function/type/quoting semantics across dialects),
an **optional** `sql_transpile` action wraps the pure-Rust
[`polyglot-sql`](https://crates.io/crates/polyglot-sql) (a sqlglot port). It is
feature-gated so the default binary stays small; build it in only when you want it:

```
cargo build --release --features sql-transpile
mog -m transpile.mog schema.sql     # {"steps":[{"action":"sql_transpile","options":{"from":"snowflake","to":"bigquery"}}]}
```

Unlike the regex converters, `sql_transpile` parses and regenerates the SQL, so it
reformats the output (not a minimal diff). Use the regex converters for a
reviewable nudge; use `sql_transpile` when you want correctness over diff.

## Building and testing

```
cargo build
cargo test
```

An extensive automated test suite (unit, integration, and end-to-end CLI, run in
CI on Linux, macOS, and Windows) covers the engine, every deterministic action
(including `replace_extended` and its `decode_extended` escape decoder, line
filtering, per-line affixing, numbering, blank/whitespace, and encoding actions),
scoped steps, `run_mog` composition, EOL/trailing-newline preservation, and the
CLI itself (via `assert_cmd`), including stdin filtering, `--diff`, `--exclude`,
and `--lossy`.

> Note: if this repo is checked out inside a synced folder (e.g. Google Drive),
> point Cargo's build output elsewhere so the sync client does not thrash on
> `target/`:
> ```
> export CARGO_TARGET_DIR="$LOCALAPPDATA/MogBuild/mog-rs-target"   # Windows/Git Bash
> ```

## Behavior notes

- **Regex** uses the `fancy-regex` crate, so backreferences and lookaround are
  supported. Replacement uses `$1` / `${name}` substitution (not `\1`).
- **Case folding** uses Rust's Unicode-default casing: identical for typical and
  ASCII text, differing only under special-casing locales.
- **Seeded `shuffle_lines` / `random_case`** are deterministic given a seed, so a
  seeded run reproduces exactly; without a seed they vary from run to run.
