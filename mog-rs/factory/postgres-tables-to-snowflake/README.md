# pg_dump (Postgres) to Snowflake

Convert pg_dump --schema-only output to Snowflake DDL

Convert pg_dump --schema-only output to Snowflake DDL: strips the pg_dump plumbing, drops the public. schema qualifier and the Postgres-only ALTER TABLE ONLY keyword, and maps types (timestamp with time zone -> TIMESTAMP_TZ, text -> VARCHAR, text[] -> ARRAY, numeric(p,s) -> NUMBER(p,s), now() -> CURRENT_TIMESTAMP()). Not a semantic transpiler; unmapped cases are flagged inline as TODO(mog). Validated by executing the result in a real Snowflake engine (fakesnow).

## Run

```
mog -m postgres-tables-to-snowflake <file>
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
    total NUMBER(18,2),
    created_at TIMESTAMP_TZ DEFAULT CURRENT_TIMESTAMP(),
    note VARCHAR,
    tags ARRAY
);

ALTER TABLE orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);
```

## Steps

- `run_mog`: Remove pg_dump plumbing, keep CREATE/ALTER.
- `replace`: Drop the public. schema qualifier.
- `replace`: Postgres ALTER TABLE ONLY -> ALTER TABLE (Snowflake has no ONLY).
- `replace_regex`: timestamp with time zone -> TIMESTAMP_TZ.
- `replace_regex`: Postgres text[] array -> Snowflake ARRAY.
- `replace_regex`: text -> VARCHAR.
- `replace_regex`: numeric(p,s) -> NUMBER(p,s).
- `replace_regex`: now() -> CURRENT_TIMESTAMP().

## Tags

`sql` `postgres` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
