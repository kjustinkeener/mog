# Strip pg_dump schema-only noise

Remove pg_dump plumbing, leaving the real CREATE/ALTER statements

Strip the boilerplate that 'pg_dump --schema-only' wraps around the real schema, so a downstream dialect converter sees clean statements. Removes the dump banners, the psql \restrict / \unrestrict directives (pg_dump 16+), the SET / pg_catalog.set_config session block, the default_tablespace / access_method lines, and the '-- Name: ...; Type: ...; Owner:' object banners, then collapses the blank lines they leave. Keeps CREATE TABLE and ALTER TABLE. Target-agnostic; run it first. Assumes LF.

## Run

```
mog -m strip-pgdump-noise <file>
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


CREATE TABLE public.orders (
    id bigint NOT NULL,
    total numeric(18,2),
    created_at timestamp with time zone DEFAULT now(),
    note text,
    tags text[]
);

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);
```

## Steps

- `eol_lf`: Normalize to LF.
- `replace_regex`: Drop the psql \restrict / \unrestrict directives (pg_dump 16+).
- `replace_regex`: Drop the SET session block.
- `replace_regex`: Drop the pg_catalog.set_config search_path line.
- `replace_regex`: Drop the dump banner and dumped-by comments.
- `replace_regex`: Drop the object-banner comment lines.
- `replace_regex`: Drop the lone '--' separator lines.
- `replace_regex`: Collapse the blank lines left behind.

## Tags

`sql` `postgres` `strip` `migration` `fragment`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
