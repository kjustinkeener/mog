# BigQuery tables to Go structs

Generate Go structs (with bigquery tags) from a BigQuery DDL export

Generate idiomatic Go type declarations from a BigQuery DDL export (INFORMATION_SCHEMA.TABLES.ddl or 'bq show'), one 'type <Table> struct' per table. Discards every non-table statement (CREATE SCHEMA, views, ALTERs) and BigQuery's physical-layout trailers (PARTITION BY, CLUSTER BY, table and column OPTIONS), then rewrites each CREATE TABLE into a struct with one exported field per column. Field names are PascalCased for export, with the exact BigQuery column name preserved losslessly in a `bigquery:"..."` struct tag (the tag cloud.google.com/go/bigquery reads). BigQuery types map to Go types (INT64 -> int64, FLOAT64 -> float64, BOOL -> bool, STRING -> string, BYTES -> []byte, DATE/DATETIME/TIME/TIMESTAMP -> time.Time, JSON -> json.RawMessage, STRUCT<...> -> map[string]any, ARRAY<T> -> []T). NUMERIC and BIGNUMERIC map to string (Go has no native fixed-point) and are flagged. BigQuery columns are NULLABLE by default -- only an explicit NOT NULL makes a column required -- so a missing NOT NULL is what produces a pointer type (*T); nilable types (slices, maps, json.RawMessage) stay as-is, and ARRAY columns are never null. Emits the struct declarations only: add your own 'package' clause and imports (time, encoding/json), then run gofmt to align fields. Assumes LF.

## Run

```
mog -m bigquery-tables-to-go <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
-- BigQuery DDL export
-- Source: SELECT ddl FROM `mog-demo.analytics.INFORMATION_SCHEMA.TABLES`

CREATE SCHEMA IF NOT EXISTS `mog-demo.analytics`;

DROP TABLE IF EXISTS `mog-demo.analytics.bqgo_scratch`;

-- Table: bqgo_actor

CREATE OR REPLACE TABLE `mog-demo.analytics.bqgo_actor`
(
    `actor_id` INT64 NOT NULL,
    `first_name` STRING NOT NULL,
    `last_name` STRING NOT NULL,
    `last_update` TIMESTAMP NOT NULL
);

-- Table: bqgo_staff
```

_(... 55 more line(s))_

Output:

```
type BqgoActor struct {
	ActorId int64 `bigquery:"actor_id"`
	FirstName string `bigquery:"first_name"`
	LastName string `bigquery:"last_name"`
	LastUpdate time.Time `bigquery:"last_update"`
}

type BqgoStaff struct {
	StaffId int64 `bigquery:"staff_id"`
	FirstName string `bigquery:"first_name"`
	LastName string `bigquery:"last_name"`
	AddressId int64 `bigquery:"address_id"`
	Email *string `bigquery:"email"`
	StoreId int64 `bigquery:"store_id"`
	Active bool `bigquery:"active"`
	Username string `bigquery:"username"`
	Password *string `bigquery:"password"`
	LastUpdate time.Time `bigquery:"last_update"`
```

_(... 24 more line(s))_

## Steps

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
- `flag_matching`: Flag BigQuery JSON columns: mapped to json.RawMessage.
- `flag_matching`: Flag BigQuery STRUCT columns: mapped to a map.
- `flag_matching`: Flag BigQuery ARRAY columns: mapped to a Go slice.
- `flag_matching`: Flag BigQuery BIGNUMERIC columns: mapped to string.
- `flag_matching`: Flag BigQuery NUMERIC columns: mapped to string.
- `replace_regex`: STRUCT<...> -> map[string]any (flagged above).
- `replace_regex`: BIGNUMERIC(p,s) -> string (flagged above).
- `replace_regex`: bare BIGNUMERIC -> string (flagged above).
- `replace_regex`: NUMERIC(p,s) / DECIMAL(p,s) -> string (flagged above).
- `replace_regex`: bare NUMERIC / DECIMAL -> string (flagged above).
- `replace_regex`: INT64 / INT / INTEGER -> int64.
- `replace_regex`: FLOAT64 / FLOAT -> float64.
- `replace_regex`: BOOL / BOOLEAN -> bool.
- `replace_regex`: STRING(n) -> string.
- `replace_regex`: bare STRING -> string.
- `replace_regex`: BYTES(n) / BYTES -> []byte.
- `replace_regex`: TIMESTAMP / DATETIME / DATE / TIME -> time.Time (via sentinel, so the 'time' keyword is not re-matched).
- `replace_regex`: JSON -> json.RawMessage (flagged above).
- `replace_regex`: ARRAY<T> -> []T (flagged above).
- `replace_regex`: Resolve the time sentinel to time.Time.
- `replace_regex`: Rewrite nullable columns into a pointer-typed Go field carrying a bigquery tag placeholder.
- `replace_regex`: Rewrite non-nullable columns into a Go field carrying a bigquery tag placeholder.
- `replace_regex`: Nilable types (slices, maps, json.RawMessage) don't need a pointer for nullability -- drop it.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each struct name (the table identifier).
- `to_pascal`: PascalCase each field name (the column identifier); the bigquery tag keeps the exact column name.
- `replace_regex`: Resolve the bigquery tag placeholder into a real Go struct tag.
- `replace_regex`: Convert the 4-space column indent to a tab (Go convention).
- `replace_regex`: Collapse multiple blank lines to a single blank between declarations.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `bigquery` `go` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
