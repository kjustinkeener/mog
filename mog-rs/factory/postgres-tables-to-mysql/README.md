# pg_dump (Postgres) to MySQL

Convert pg_dump --schema-only output to MySQL DDL

Convert pg_dump --schema-only output to MySQL DDL: strips pg_dump plumbing, drops the public. qualifier and ALTER TABLE ONLY, maps types (text[]->JSON flagged, text->TEXT, numeric->DECIMAL, timestamp with/without time zone->DATETIME), and rewrites now()->CURRENT_TIMESTAMP. Not a semantic transpiler; unmapped cases are flagged inline as TODO(mog).

## Run

```
mog -m postgres-tables-to-mysql <file>
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
    total DECIMAL(18,2),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    note text,
    tags JSON -- TODO(mog): Postgres array mapped to JSON; MySQL has no array type
);

ALTER TABLE orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);
```

## Pipeline

- `run_mog`
- `replace`
- `replace`
- `flag_matching`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`

## Tags

`sql` `postgres` `mysql` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
