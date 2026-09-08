# pg_dump (Postgres) tables to TypeScript interfaces

Generate TypeScript interfaces from pg_dump --schema-only output

Generate TypeScript 'export interface' declarations from a pg_dump --schema-only export, one interface per table, 2-space indented. Discards non-table statements (sequences, indexes, triggers, ALTERs). Column/member names are kept exactly as in the database (snake_case, matching driver row keys); only the interface name is PascalCased. Postgres types map to TypeScript types (integer/serial/smallint/bigint/numeric/decimal/real/double precision -> number, boolean -> boolean, character varying/varchar/char/bpchar/text/uuid -> string, timestamp/timestamptz/date/time/timetz -> string as ISO-8601 text, bytea -> Uint8Array, json/jsonb -> unknown). Nullable columns get a '<type> | null' union rather than an optional '?' member. Exotic types (enum -> string, text[] -> string[], tsvector -> string) map best-effort and are flagged inline with a // TODO(mog): comment. Assumes LF.

## Run

```
mog -m postgres-tables-to-typescript <file>
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
export interface Actor {
  actor_id: number;
  first_name: string;
  last_name: string;
  last_update: string;
}

export interface Staff {
  staff_id: number;
  first_name: string;
  last_name: string;
  address_id: number;
  email: string | null;
  store_id: number;
  active: boolean;
  username: string;
  password: string | null;
  last_update: string;
```

_(... 26 more line(s))_

## Steps

- `run_mog`: Remove pg_dump banners, SET block, restrict directives and object comments.
- `replace_regex`: Drop CREATE SEQUENCE statements (multi-line, to the first ';').
- `replace_regex`: Drop ALTER SEQUENCE statements.
- `replace_regex`: Drop ALTER TABLE statements (constraints, defaults, foreign keys).
- `replace_regex`: Drop CREATE INDEX statements.
- `replace_regex`: Drop CREATE TRIGGER statements.
- `replace_regex`: Drop CREATE VIEW / MATERIALIZED VIEW statements.
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE public.<tbl> (' into a TypeScript interface header + opening brace.
- `replace_regex`: Rewrite each table-closing ');' into a closing brace.
- `replace_regex`: Drop DEFAULT clauses from column lines (nextval, now(), literals).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag Postgres enum columns (public.<type>): mapped to string.
- `flag_matching`: Flag Postgres array columns: mapped to a TypeScript array.
- `flag_matching`: Flag Postgres tsvector columns: mapped to string.
- `replace_regex`: character varying(n) -> string.
- `replace_regex`: character varying / varchar -> string.
- `replace_regex`: character(n) / char(n) -> string.
- `replace_regex`: bare character / char / bpchar -> string.
- `replace_regex`: timestamp with/without time zone -> string (ISO-8601 text).
- `replace_regex`: timestamptz -> string (ISO-8601 text).
- `replace_regex`: time with/without time zone / timetz / time -> string (ISO-8601 text).
- `replace_regex`: double precision -> number.
- `replace_regex`: numeric(p,s) / decimal(p,s) -> number.
- `replace_regex`: bare numeric / decimal -> number.
- `replace_regex`: bigserial / bigint -> number.
- `replace_regex`: smallserial / smallint -> number.
- `replace_regex`: serial / integer / int -> number.
- `replace_regex`: boolean / bool -> boolean.
- `replace_regex`: real -> number.
- `replace_regex`: bare double -> number.
- `replace_regex`: Postgres enum reference (public.<type>) -> string (flagged above).
- `replace_regex`: text[] array -> string[] (flagged above).
- `replace_regex`: tsvector -> string (flagged above).
- `replace_regex`: text -> string.
- `replace_regex`: uuid -> string.
- `replace_regex`: bytea -> Uint8Array.
- `replace_regex`: date -> string (ISO-8601 text).
- `replace_regex`: jsonb / json -> unknown.
- `replace_regex`: Rewrite nullable columns into a TypeScript member with a '| null' union (2-space indent).
- `replace_regex`: Rewrite non-nullable columns into a TypeScript member (2-space indent).
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each interface name (the table identifier); column members stay snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between interfaces.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `postgres` `typescript` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
