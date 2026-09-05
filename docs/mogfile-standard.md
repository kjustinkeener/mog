# The mogfile torture-test standard

Every `.mog` script ships with an associated **torture-test input**, an
edge-heavy sample authored to stress what the script does, **and** a **golden
expected output** captured from a known-good run. Both are mandatory: a script is
not complete for inclusion without the pair. This makes "prove it against the
hard cases" a property of every script, not an afterthought.

## Naming convention

Beside `Foo.mog`, in the same directory, **both** required:

| File | Purpose |
| --- | --- |
| `Foo.TestInput.<ext>` | The torture input: the messiest, most edge-case-laden sample the script is meant to handle. `<ext>` is the input's natural extension (`.sql`, `.txt`, ...). |
| `Foo.TestExpectedOutput.<ext>` | The golden: exactly what the script produces from `Foo.TestInput`. Its `<ext>` is the output's natural extension. |

A script may also keep additional narrower fixtures elsewhere (e.g. under
`samples/`) for focused tests; the convention only fixes the one canonical pair.

## What the test harness enforces

`mog-rs/tests/torture_standard.rs` scans every `.mog` in the repo and:

1. **Requires** both `Foo.TestInput.<ext>` and `Foo.TestExpectedOutput.<ext>`
   beside each script (fails the build if either is missing).
2. **Runs** the script over the input and asserts the result matches the golden
   (EOL-normalized), so the golden can't silently drift.

Regenerate a golden after an intended change with:

```bash
mog -m Foo.mog < Foo.TestInput.ext > Foo.TestExpectedOutput.ext
```

## Authoring a good torture input

- Pack in the constructs the script targets *and* the ones it should leave alone.
- Prefer **authentic** samples over hand-written guesses when a tool produces the
  format (e.g. `mysqldump`, SSMS "Generate Scripts"): generate once, commit it.
- Where the script can only partially handle something, the golden should show it
  **flagged** (`flag_matching` -> `FIXME(mog): ...`) rather than silently wrong,
  so the expected output documents the known gap.

## Current pairs

| Script | Input | Golden |
| --- | --- | --- |
| `MSSQL-To-PGSQL-Table.mog` | `MSSQL-To-PGSQL-Table.TestInput.sql` | `MSSQL-To-PGSQL-Table.TestExpectedOutput.pgsql` |
| `MySQL-To-PGSQL.mog` | `MySQL-To-PGSQL.TestInput.sql` | `MySQL-To-PGSQL.TestExpectedOutput.pgsql` |
| `mssql-strip-server-noise.mog` | `mssql-strip-server-noise.TestInput.sql` | `mssql-strip-server-noise.TestExpectedOutput.sql` |
| `mysql-strip-dump-noise.mog` | `mysql-strip-dump-noise.TestInput.sql` | `mysql-strip-dump-noise.TestExpectedOutput.sql` |
| `samples/tidy-list.mog` | `samples/tidy-list.TestInput.txt` | `samples/tidy-list.TestExpectedOutput.txt` |
