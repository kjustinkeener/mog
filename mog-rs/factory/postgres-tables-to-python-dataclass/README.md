# pg_dump (Postgres) tables to Python dataclasses

Generate Python @dataclass classes from pg_dump --schema-only output

Generate Python standard-library @dataclass classes from a pg_dump --schema-only export, one class per table. Discards non-table statements (sequences, indexes, triggers, ALTERs). Column names stay snake_case; only the class name is PascalCased (film_actor -> FilmActor). Postgres types map to Python types (integer/serial/smallint/bigint->int, boolean->bool, character varying/text/char->str, numeric/decimal->Decimal, real/double precision->float, timestamp/timestamptz->datetime, date->date, time->time, uuid->UUID, bytea->bytes, json/jsonb->dict). Nullable columns become 'type | None'. Emits a fixed stdlib import header with 'from __future__ import annotations' so the module runs on Python 3.7+. Exotic types (enum, tsvector, text[]) map best-effort (str, list[str]) and are flagged with a # TODO(mog): comment. Assumes LF.

## Run

```
mog -m postgres-tables-to-python-dataclass <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
--
-- PostgreSQL database dump
--

\restrict pcl6lAUWBNEJA4fmbE13w33a8O1LTuRZeccWcR1XUyTkOHAUOk928FpS36wOqnO

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

_(... 125 more line(s))_

Output:

```
from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from uuid import UUID

@dataclass
class Actor:
    actor_id: int
    first_name: str
    last_name: str
    last_update: datetime

@dataclass
class Staff:
    staff_id: int
    first_name: str
    last_name: str
```

_(... 32 more line(s))_

## Pipeline

- `run_mog`: Remove pg_dump banners, SET block, restrict directives and object comments.
- `replace_regex`: Drop CREATE SEQUENCE statements (multi-line, to the first ';').
- `replace_regex`: Drop ALTER SEQUENCE statements.
- `replace_regex`: Drop ALTER TABLE statements (constraints, defaults, foreign keys).
- `replace_regex`: Drop CREATE INDEX statements.
- `replace_regex`: Drop CREATE TRIGGER statements.
- `replace_regex`: Drop CREATE VIEW / MATERIALIZED VIEW statements.
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE public.<tbl> (' into a @dataclass decorator + class header (no closing brace in Python).
- `replace_regex`: Drop each table-closing ');' entirely (Python has no closing brace).
- `replace_regex`: Drop DEFAULT clauses from column lines (nextval, now(), literals).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag Postgres enum columns (public.<type>): mapped to str.
- `flag_matching`: Flag Postgres array columns: mapped to a Python list.
- `flag_matching`: Flag Postgres tsvector columns: mapped to str.
- `replace_regex`: character varying(n) -> str.
- `replace_regex`: character varying / varchar -> str.
- `replace_regex`: character(n) / char(n) / bpchar(n) -> str.
- `replace_regex`: bare character / char / bpchar -> str.
- `replace_regex`: timestamp with/without time zone (and bare timestamp) -> datetime.
- `replace_regex`: timestamptz -> datetime.
- `replace_regex`: time with/without time zone / timetz / time -> time (datetime.time).
- `replace_regex`: double precision -> float.
- `replace_regex`: numeric(p,s) / decimal(p,s) -> Decimal.
- `replace_regex`: bare numeric / decimal -> Decimal.
- `replace_regex`: bigserial / bigint -> int.
- `replace_regex`: smallserial / smallint -> int.
- `replace_regex`: serial / integer / int -> int.
- `replace_regex`: boolean / bool -> bool.
- `replace_regex`: real -> float.
- `replace_regex`: bare double -> float.
- `replace_regex`: Postgres enum reference (public.<type>) -> str (flagged above).
- `replace_regex`: text[] array -> list[str] (flagged above).
- `replace_regex`: tsvector -> str (flagged above).
- `replace_regex`: text -> str.
- `replace_regex`: uuid -> UUID.
- `replace_regex`: bytea -> bytes.
- `replace_regex`: date -> date (datetime.date; identity map, kept explicit).
- `replace_regex`: jsonb / json -> dict (assumes object payloads; use str for raw JSON).
- `replace_regex`: Rewrite nullable columns into a dataclass field with a 'type | None' annotation.
- `replace_regex`: Rewrite non-nullable columns into a dataclass field with a plain annotation.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier); field names stay snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.
- `replace_regex`: Prepend the fixed stdlib import header (last, so the type-map regexes never touch 'date'/'time'/'decimal' in the imports).

## Tags

`sql` `postgres` `python` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
