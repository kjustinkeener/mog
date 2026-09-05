# BigQuery tables to TypeScript interfaces

Generate TypeScript interfaces from a BigQuery DDL export

Generate TypeScript 'export interface' declarations from a BigQuery DDL export (INFORMATION_SCHEMA.TABLES.ddl or 'bq show'), one interface per table, 2-space indented. Discards non-table statements (CREATE SCHEMA, views, ALTERs) and BigQuery's physical-layout trailers (PARTITION BY, CLUSTER BY, table and column OPTIONS). Column/member names are kept exactly as in the table (snake_case, matching the keys the BigQuery client returns); only the interface name is PascalCased. BigQuery types map to TypeScript types (INT64/NUMERIC/BIGNUMERIC/FLOAT64 -> number, BOOL -> boolean, STRING/BYTES-as-base64 aside, DATE/DATETIME/TIME/TIMESTAMP -> string as ISO-8601 text, BYTES -> Uint8Array, JSON -> unknown, STRUCT<...> -> Record<string,unknown>, ARRAY<T> -> T[]). BigQuery columns are NULLABLE by default -- only an explicit NOT NULL makes a column required -- so the '<type> | null' union is driven by the ABSENCE of NOT NULL. ARRAY columns are never null and stay a plain array. Semi-structured and lossy types (JSON, STRUCT, BIGNUMERIC) are flagged inline with a // TODO(mog): comment. Assumes LF.

## Run

```
mog -m bigquery-tables-to-typescript <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
-- BigQuery DDL export
-- Source: SELECT ddl FROM `mog-demo.analytics.INFORMATION_SCHEMA.TABLES`

CREATE SCHEMA IF NOT EXISTS `mog-demo.analytics`;

DROP TABLE IF EXISTS `mog-demo.analytics.bqts_scratch`;

-- Table: bqts_actor

CREATE OR REPLACE TABLE `mog-demo.analytics.bqts_actor`
(
    `actor_id` INT64 NOT NULL,
    `first_name` STRING NOT NULL,
    `last_name` STRING NOT NULL,
    `last_update` TIMESTAMP NOT NULL
);

-- Table: bqts_staff
```

_(... 55 more line(s))_

Output:

```
export interface BqtsActor {
  actor_id: number;
  first_name: string;
  last_name: string;
  last_update: string;
}

export interface BqtsStaff {
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

_(... 24 more line(s))_

## Pipeline

- `replace_regex`: Drop the export's leading '--' comment banners and per-table headers.
- `replace_regex`: Drop CREATE SCHEMA / CREATE DATASET statements.
- `replace_regex`: Drop DROP TABLE / DROP VIEW statements.
- `replace_regex`: Drop CREATE VIEW / MATERIALIZED VIEW statements (multi-line, to the first ';').
- `replace_regex`: Drop ALTER TABLE statements (SET OPTIONS, ADD COLUMN, drops).
- `replace_regex`: Drop the table-level OPTIONS(...) trailer (description, labels, expiration).
- `replace_regex`: Drop PARTITION BY trailers (BigQuery physical layout, not part of the row type).
- `replace_regex`: Drop CLUSTER BY trailers (BigQuery physical layout, not part of the row type).
- `replace_regex`: Strip the backtick quoting BigQuery puts around table and column identifiers.
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE OR REPLACE TABLE <project>.<dataset>.<tbl>' plus its own-line '(' into the type header.
- `replace_regex`: Rewrite each table-closing ')' (with or without a trailing ';') into the closing form.
- `replace_regex`: Drop per-column OPTIONS(...) clauses (column descriptions and policy tags).
- `replace_regex`: Drop DEFAULT clauses from column lines (literals, CURRENT_TIMESTAMP(), GENERATE_UUID()).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: BigQuery ARRAY columns are never NULL (an absent value is the empty array), so mark them NOT NULL before the nullability pass.
- `replace_regex`: Mark nullable columns with a sentinel: in BigQuery a column is NULLABLE unless it explicitly says NOT NULL, so nullability is driven by the ABSENCE of the constraint.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag BigQuery JSON columns: mapped to unknown.
- `flag_matching`: Flag BigQuery STRUCT columns: mapped to an index signature.
- `flag_matching`: Flag BigQuery ARRAY columns: mapped to a TypeScript array.
- `flag_matching`: Flag BigQuery BIGNUMERIC columns: mapped to number.
- `replace_regex`: STRUCT<...> -> Record<string,unknown> (flagged above).
- `replace_regex`: BIGNUMERIC(p,s) -> number (flagged above).
- `replace_regex`: bare BIGNUMERIC -> number (flagged above).
- `replace_regex`: NUMERIC(p,s) / DECIMAL(p,s) -> number.
- `replace_regex`: bare NUMERIC / DECIMAL -> number.
- `replace_regex`: INT64 / INT / INTEGER -> number.
- `replace_regex`: FLOAT64 / FLOAT -> number.
- `replace_regex`: BOOL / BOOLEAN -> boolean.
- `replace_regex`: STRING(n) -> string.
- `replace_regex`: bare STRING -> string.
- `replace_regex`: BYTES(n) / BYTES -> Uint8Array.
- `replace_regex`: TIMESTAMP / DATETIME / DATE / TIME -> string (ISO-8601 text).
- `replace_regex`: JSON -> unknown (flagged above).
- `replace_regex`: ARRAY<T> -> T[] (flagged above).
- `replace_regex`: Rewrite nullable columns into a TypeScript member with a '| null' union (2-space indent).
- `replace_regex`: Rewrite non-nullable columns into a TypeScript member (2-space indent).
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each interface name (the table identifier); column members stay snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between declarations.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `bigquery` `typescript` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
