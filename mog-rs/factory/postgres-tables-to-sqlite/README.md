# pg_dump (Postgres) to SQLite

Convert pg_dump --schema-only output to SQLite DDL

Convert pg_dump --schema-only output to SQLite DDL: strips the pg_dump plumbing, drops the public. qualifier, comments out standalone ALTER TABLE ADD CONSTRAINT PRIMARY KEY (SQLite cannot ALTER-add a PK; flagged), and maps text[]->TEXT (flagged). SQLite's flexible typing accepts the remaining types as affinities. Unmapped cases are flagged inline as TODO(mog). Validated by executing the result in a real SQLite engine.

## Run

```
mog -m postgres-tables-to-sqlite <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
--
-- PostgreSQL database dump
--

\restrict FIXEDTOKENabc123FIXEDTOKENabc123FIXEDTOKENabc123FIXED

-- Dumped from database version 16.15 (Debian 16.15-1.pgdg13+2)
-- Dumped by pg_dump version 16.15 (Debian 16.15-1.pgdg13+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET client_min_messages = warning;
SET row_security = off;

```

_(... 30 more line(s))_

Output:

```


CREATE TABLE orders (
    id bigint NOT NULL,
    total numeric(18,2),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    note text,
    tags TEXT -- TODO(mog): Postgres array mapped to TEXT; SQLite has no array type
);

-- TODO(mog): SQLite cannot ALTER-add a PRIMARY KEY; fold it into the CREATE TABLE
```

## Steps

- `run_mog`
- `replace`
- `replace_regex`
- `flag_matching`
- `replace_regex`
- `replace_regex`

## Tags

`sql` `postgres` `sqlite` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
