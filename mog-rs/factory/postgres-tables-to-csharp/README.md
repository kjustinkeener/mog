# pg_dump (Postgres) tables to C# classes

Generate C# POCO classes from pg_dump --schema-only output

Generate idiomatic C# POCO classes from a real 'pg_dump --schema-only' export, one class per table. Discards every non-table statement (sequences, indexes, triggers, ALTERs), then rewrites each CREATE TABLE into a 'public class' with one auto-property per column. Postgres types map to C# types (integer->int, bigint->long, smallint->short, boolean->bool, numeric->decimal, real->float, double precision->double, character varying/text/char->string, timestamp/timestamptz->DateTime, date->DateOnly, time->TimeOnly, uuid->Guid, bytea->byte[], json/jsonb->string). Nullable columns (no NOT NULL) get a nullable type ('?'); timestamptz maps to DateTime (not DateTimeOffset). Table and column identifiers are PascalCased. Types with no clean C# equivalent (enum, tsvector, array text[]) map to a best-effort type flagged with a // TODO(mog): comment. Assumes LF.

## Run

```
mog -m postgres-tables-to-csharp <file>
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
public class Actor
{
    public int ActorId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime LastUpdate { get; set; }
}

public class Staff
{
    public int StaffId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public int AddressId { get; set; }
    public string? Email { get; set; }
    public int StoreId { get; set; }
    public bool Active { get; set; }
    public string Username { get; set; }
```

_(... 29 more line(s))_

## Pipeline

- `run_mog`: Remove pg_dump banners, SET block, restrict directives and object comments.
- `replace_regex`: Drop CREATE SEQUENCE statements (multi-line, to the first ';').
- `replace_regex`: Drop ALTER SEQUENCE statements.
- `replace_regex`: Drop ALTER TABLE statements (constraints, defaults, foreign keys).
- `replace_regex`: Drop CREATE INDEX statements.
- `replace_regex`: Drop CREATE TRIGGER statements.
- `replace_regex`: Drop CREATE VIEW / MATERIALIZED VIEW statements.
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE public.<tbl> (' into a C# class header + opening brace.
- `replace_regex`: Rewrite each table-closing ');' into a closing brace.
- `replace_regex`: Drop DEFAULT clauses from column lines (nextval, now(), literals).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag Postgres enum columns (public.<type>): mapped to string.
- `flag_matching`: Flag Postgres array columns: mapped to a C# array.
- `flag_matching`: Flag Postgres tsvector columns: mapped to string.
- `replace_regex`: character varying(n) -> string.
- `replace_regex`: character varying / varchar -> string.
- `replace_regex`: character(n) / char(n) -> string.
- `replace_regex`: bare character / char / bpchar -> string.
- `replace_regex`: timestamp with/without time zone -> DateTime (timestamptz maps to DateTime, not DateTimeOffset).
- `replace_regex`: timestamptz -> DateTime.
- `replace_regex`: time with/without time zone / timetz / time -> TimeOnly.
- `replace_regex`: double precision -> double.
- `replace_regex`: numeric(p,s) / decimal(p,s) -> decimal.
- `replace_regex`: bare numeric / decimal -> decimal.
- `replace_regex`: bigserial / bigint -> long.
- `replace_regex`: smallserial / smallint -> short.
- `replace_regex`: serial / integer / int -> int.
- `replace_regex`: boolean / bool -> bool.
- `replace_regex`: real -> float.
- `replace_regex`: bare double -> double.
- `replace_regex`: Postgres enum reference (public.<type>) -> string (flagged above).
- `replace_regex`: text[] array -> string[] (flagged above).
- `replace_regex`: tsvector -> string (flagged above).
- `replace_regex`: text -> string.
- `replace_regex`: uuid -> Guid.
- `replace_regex`: bytea -> byte[].
- `replace_regex`: date -> DateOnly.
- `replace_regex`: jsonb / json -> string.
- `replace_regex`: Rewrite nullable columns into a C# auto-property with a nullable type.
- `replace_regex`: Rewrite non-nullable columns into a C# auto-property.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier).
- `to_pascal`: PascalCase each property name (the column identifier).
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `postgres` `csharp` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
