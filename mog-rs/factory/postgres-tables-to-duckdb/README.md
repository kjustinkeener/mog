# pg_dump (Postgres) to DuckDB

Convert pg_dump --schema-only output to DuckDB DDL

Convert real 'pg_dump --schema-only' output to DuckDB DDL. DuckDB is highly Postgres-compatible, so after stripping the pg_dump plumbing the only rewrites needed are dropping the public. schema qualifier and the Postgres-only ALTER TABLE ONLY keyword; types (bigint, numeric, text, text[], timestamptz, now(), and the PK ALTER) pass through and execute in DuckDB as-is. Validated by executing the result in a real DuckDB engine.

## Run

```
mog -m postgres-tables-to-duckdb <file>
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
    created_at timestamp with time zone DEFAULT now(),
    note text,
    tags text[]
);

ALTER TABLE orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);
```

## Steps

- `run_mog`
- `replace`: Drop the public. schema qualifier.
- `replace`: Postgres ALTER TABLE ONLY -> ALTER TABLE (DuckDB has no ONLY).

## Tags

`sql` `postgres` `duckdb` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
