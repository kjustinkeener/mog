# pg_dump (Postgres) to ClickHouse

Convert pg_dump --schema-only output to a runnable ClickHouse MergeTree table

Turn real 'pg_dump --schema-only' table DDL into an executable ClickHouse CREATE TABLE for analytics ingestion. Strips the pg_dump plumbing, drops the public. qualifier, and maps every Postgres column type to its ClickHouse equivalent (character varying/text -> String, integer -> Int32, bigint -> Int64, smallint -> Int16, numeric(p,s) -> Decimal(p,s), double precision -> Float64, real -> Float32, timestamp with time zone -> DateTime64(6, 'UTC'), timestamp without time zone -> DateTime64(6), date -> Date, boolean -> Bool, uuid -> UUID, jsonb/json -> String, bytea -> String). Applies ClickHouse's NOT-NULL-by-default rule (wraps nullable columns in Nullable(...)) and synthesizes 'ENGINE = MergeTree' plus 'ORDER BY (<pk cols>)' from the PRIMARY KEY (falling back to ORDER BY tuple() when there is no PK). Non-portable constructs are dropped and stamped with -- TODO(mog): IDENTITY/sequences, the consumed PRIMARY KEY, UNIQUE/FOREIGN KEY/CHECK constraints (ClickHouse does not enforce them), and secondary indexes. Minimal-diff regex rewrite, NOT a semantic transpiler. Validated by executing the result in a real ClickHouse engine.

## Run

```
mog -m postgres-tables-to-clickhouse <file>
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
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
```

_(... 72 more line(s))_

Output:

```
CREATE TABLE pg2ch_orders (
    id Int64,
    order_ref UUID DEFAULT generateUUIDv4(),
    customer_code String,
    description Nullable(String),
    quantity Int32,
    unit_price Decimal(12,2),
    discount Nullable(Float64),
    weight_kg Nullable(Float32),
    is_paid Bool DEFAULT false,
    status_code Nullable(Int16),
    created_at DateTime64(6, 'UTC') DEFAULT now(),
    updated_at Nullable(DateTime64(6)),
    order_date Nullable(Date),
    metadata Nullable(String),
    payload Nullable(String)
)
ENGINE = MergeTree
```

_(... 10 more line(s))_

## Steps

- `run_mog`: Remove pg_dump plumbing (banners, \restrict, SET block, object banners), keep CREATE/ALTER. Normalizes to LF.
- `replace`: Drop the public. schema qualifier; ClickHouse tables live in a database, not a schema.
- `replace_regex`: character varying(n) -> String (ClickHouse String is unbounded; drop the length).
- `replace_regex`: varchar(n) -> String (in case the dump kept the short spelling).
- `replace_regex`: character(n) / char(n) -> String (fixed-width becomes String).
- `replace_regex`: text -> String.
- `replace_regex`: timestamp without time zone -> DateTime64(6). Run before the bare-timestamp and with-time-zone maps so it is not re-matched.
- `replace_regex`: timestamp with time zone / timestamptz -> DateTime64(6, 'UTC').
- `replace_regex`: bare timestamp -> DateTime64(6) (fallback; the produced DateTime64 does not contain the word 'timestamp').
- `replace_regex`: bigint -> Int64.
- `replace_regex`: smallint -> Int16.
- `replace_regex`: integer / int -> Int32.
- `replace_regex`: numeric(p,s) -> Decimal(p,s).
- `replace_regex`: decimal(p,s) -> Decimal(p,s) (canonical casing).
- `replace_regex`: double precision -> Float64.
- `replace_regex`: real -> Float32.
- `replace_regex`: boolean -> Bool.
- `replace_regex`: uuid -> UUID.
- `replace_regex`: jsonb -> String (ClickHouse JSON is experimental; String is portable). Run before json so jsonb is not left as 'Stringb'.
- `replace_regex`: json -> String.
- `replace_regex`: bytea -> String (ClickHouse stores binary in String).
- `replace_regex`: date -> Date. Case-sensitive so it never bites the 'Date' inside the generated 'DateTime64'.
- `replace`: gen_random_uuid() -> generateUUIDv4() (ClickHouse UUID default).
- `replace_regex`: ClickHouse is NOT NULL by default: wrap the type of every column WITHOUT a NOT NULL marker in Nullable(...). Keyed on the absence of NOT NULL (negative lookahead), so it must run before NOT NULL is stripped. The DateTime64/Decimal parenthesized forms are listed first so they are not shadowed by the bare Date/Int alternatives.
- `replace`: Strip the Postgres NOT NULL marker; NOT NULL is the ClickHouse default and non-nullable columns are left bare.
- `replace_regex`: Core transform: close the CREATE TABLE with a MergeTree engine and derive ORDER BY from the PRIMARY KEY, which pg_dump emits as a separate ALTER further down (captured via lookahead, not consumed here). Anchored on CREATE TABLE and lazy to the first ');' so it fires exactly once.
- `replace_regex`: Fallback for a table with no PRIMARY KEY: close it with ORDER BY tuple(). The tempered body (no 'ENGINE =' before the close) means this is a no-op once the PK step has already added an engine.
- `replace_regex`: Drop the Postgres IDENTITY / sequence ALTER and flag it: ClickHouse has no server-side auto-increment.
- `replace_regex`: Drop and flag a UNIQUE constraint: ClickHouse does not enforce uniqueness.
- `replace_regex`: Drop the separate PRIMARY KEY ALTER (already consumed into ORDER BY) and leave a breadcrumb.
- `replace_regex`: Drop and flag a FOREIGN KEY constraint: ClickHouse has no referential integrity.
- `replace_regex`: Drop and flag a CHECK constraint: ClickHouse has no CHECK constraints.
- `replace_regex`: Drop and flag a secondary index; ClickHouse indexing is the ORDER BY key plus optional data-skipping indexes.
- `replace_regex`: Trim leading blank lines left by the strip pass.
- `squeeze_blank_lines`: Collapse runs of blank lines to a single blank line.
- `trim_whitespace_right`: Remove trailing whitespace on every line.

## Tags

`sql` `postgres` `clickhouse` `convert` `migration` `analytics` `warehouse`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
