# pg_dump (Postgres) tables to Go structs

Generate Go structs (with db tags) from pg_dump --schema-only output

Generate idiomatic Go type declarations from a real 'pg_dump --schema-only' export, one 'type <Table> struct' per table. Discards every non-table statement (sequences, indexes, triggers, views, ALTERs), then rewrites each CREATE TABLE into a struct with one exported field per column. Field names are PascalCased for export, with the exact database column name preserved losslessly in a `db:"..."` struct tag (compatible with sqlx / pgx). Postgres types map to Go types (smallint->int16, integer/serial->int32, bigint/bigserial->int64, real->float32, double precision->float64, boolean->bool, character varying/char/text/uuid->string, timestamp/timestamptz/date/time->time.Time, bytea->[]byte, json/jsonb->json.RawMessage). numeric/decimal map to string (Go has no native decimal) and are flagged. Nullable columns (no NOT NULL) become pointer types (*T); nilable types (slices, json.RawMessage) stay as-is. Types with no clean Go equivalent (enum -> string, text[] -> []string, tsvector -> string) map to a best-effort type flagged inline with a // TODO(mog): comment. Emits the struct declarations only: add your own 'package' clause and imports (time, encoding/json), then run gofmt to align fields. Assumes LF.

## Run

```
mog -m postgres-tables-to-go <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
--
-- PostgreSQL database dump
--

\restrict 5krVriBmrhFkvowk5M0QgoC3r5cwWoi0f0tQKtzrKdseUnYPr7tpFoJmGjqkxX3

-- Dumped from database version 16.15 (Debian 16.15-1.pgdg13+2)
-- Dumped by pg_dump version 16.15 (Debian 16.15-1.pgdg13+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
```

_(... 151 more line(s))_

Output:

```
type Actor struct {
	ActorId int32 `db:"actor_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	LastUpdate time.Time `db:"last_update"`
}

type Staff struct {
	StaffId int32 `db:"staff_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	AddressId int32 `db:"address_id"`
	Email *string `db:"email"`
	StoreId int32 `db:"store_id"`
	Active bool `db:"active"`
	Username string `db:"username"`
	Password *string `db:"password"`
	LastUpdate time.Time `db:"last_update"`
```

_(... 26 more line(s))_

## Pipeline

- `run_mog`: Remove pg_dump banners, SET block, restrict directives and object comments.
- `replace_regex`: Drop CREATE SEQUENCE statements (multi-line, to the first ';').
- `replace_regex`: Drop ALTER SEQUENCE statements.
- `replace_regex`: Drop ALTER TABLE statements (constraints, defaults, foreign keys).
- `replace_regex`: Drop CREATE INDEX statements.
- `replace_regex`: Drop CREATE TRIGGER statements.
- `replace_regex`: Drop CREATE VIEW / MATERIALIZED VIEW statements.
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE public.<tbl> (' into a Go struct header + opening brace.
- `replace_regex`: Rewrite each table-closing ');' into a closing brace.
- `replace_regex`: Drop DEFAULT clauses from column lines (nextval, now(), literals).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag Postgres enum columns (public.<type>): mapped to string.
- `flag_matching`: Flag Postgres array columns: mapped to a Go slice.
- `flag_matching`: Flag Postgres tsvector columns: mapped to string.
- `flag_matching`: Flag Postgres numeric/decimal columns: mapped to string.
- `replace_regex`: character varying(n) -> string.
- `replace_regex`: character varying / varchar -> string.
- `replace_regex`: character(n) / char(n) -> string.
- `replace_regex`: bare character / char / bpchar -> string.
- `replace_regex`: timestamp with/without time zone -> time.Time (via sentinel).
- `replace_regex`: timestamptz -> time.Time (via sentinel).
- `replace_regex`: time with/without time zone / timetz / time -> time.Time (via sentinel).
- `replace_regex`: double precision -> float64.
- `replace_regex`: numeric(p,s) / decimal(p,s) -> string (flagged above).
- `replace_regex`: bare numeric / decimal -> string (flagged above).
- `replace_regex`: bigserial / bigint -> int64.
- `replace_regex`: smallserial / smallint -> int16.
- `replace_regex`: serial / integer / int -> int32.
- `replace_regex`: boolean / bool -> bool.
- `replace_regex`: real -> float32.
- `replace_regex`: bare double -> float64.
- `replace_regex`: Postgres enum reference (public.<type>) -> string (flagged above).
- `replace_regex`: text[] array -> []string (flagged above).
- `replace_regex`: tsvector -> string (flagged above).
- `replace_regex`: text -> string.
- `replace_regex`: uuid -> string.
- `replace_regex`: bytea -> []byte.
- `replace_regex`: date -> time.Time (via sentinel).
- `replace_regex`: jsonb / json -> json.RawMessage.
- `replace_regex`: Resolve the time sentinel to time.Time (kept separate to avoid re-matching the 'time' keyword).
- `replace_regex`: Rewrite nullable columns into a pointer-typed Go field carrying a db tag placeholder.
- `replace_regex`: Rewrite non-nullable columns into a Go field carrying a db tag placeholder.
- `replace_regex`: Nilable types (slices, json.RawMessage) don't need a pointer for nullability -- drop it.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each struct name (the table identifier).
- `to_pascal`: PascalCase each field name (the column identifier); the db tag keeps the exact column name.
- `replace_regex`: Resolve the db tag placeholder into a real Go struct tag.
- `replace_regex`: Convert the 4-space column indent to a tab (Go convention).
- `replace_regex`: Collapse multiple blank lines to a single blank between structs.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `postgres` `go` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
