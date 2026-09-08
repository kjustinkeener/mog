# BigQuery tables to C# classes

Generate C# POCO classes from a BigQuery DDL export

Generate idiomatic C# POCO classes from a BigQuery DDL export (INFORMATION_SCHEMA.TABLES.ddl or 'bq show'), one class per table. Discards every non-table statement (CREATE SCHEMA, views, ALTERs) and BigQuery's physical-layout trailers (PARTITION BY, CLUSTER BY, table and column OPTIONS), then rewrites each CREATE TABLE into a 'public class' with one auto-property per column. BigQuery types map to C# types (INT64 -> long, FLOAT64 -> double, NUMERIC/BIGNUMERIC -> decimal, BOOL -> bool, STRING -> string, BYTES -> byte[], DATE -> DateOnly, TIME -> TimeOnly, DATETIME/TIMESTAMP -> DateTime, JSON -> string, STRUCT<...> -> Dictionary<string,object>, ARRAY<T> -> List<T>). BigQuery columns are NULLABLE by default -- only an explicit NOT NULL makes a column required -- so the nullable '?' suffix is driven by the ABSENCE of NOT NULL; ARRAY columns are never null. Table and column identifiers are PascalCased. Semi-structured and lossy types (JSON, STRUCT, BIGNUMERIC) are flagged with a // TODO(mog): comment. Assumes LF.

## Run

```
mog -m bigquery-tables-to-csharp <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
-- BigQuery DDL export
-- Source: SELECT ddl FROM `mog-demo.analytics.INFORMATION_SCHEMA.TABLES`

CREATE SCHEMA IF NOT EXISTS `mog-demo.analytics`;

DROP TABLE IF EXISTS `mog-demo.analytics.bqcs_scratch`;

-- Table: bqcs_actor

CREATE OR REPLACE TABLE `mog-demo.analytics.bqcs_actor`
(
    `actor_id` INT64 NOT NULL,
    `first_name` STRING NOT NULL,
    `last_name` STRING NOT NULL,
    `last_update` TIMESTAMP NOT NULL
);

-- Table: bqcs_staff
```

_(... 55 more line(s))_

Output:

```
using System;
using System.Collections.Generic;

public class BqcsActor
{
    public long ActorId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime LastUpdate { get; set; }
}

public class BqcsStaff
{
    public long StaffId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public long AddressId { get; set; }
    public string? Email { get; set; }
```

_(... 30 more line(s))_

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
- `flag_matching`: Flag BigQuery JSON columns: mapped to string.
- `flag_matching`: Flag BigQuery STRUCT columns: mapped to a dictionary.
- `flag_matching`: Flag BigQuery ARRAY columns: mapped to a C# list.
- `flag_matching`: Flag BigQuery BIGNUMERIC columns: mapped to decimal.
- `replace_regex`: STRUCT<...> -> Dictionary<string,object> (flagged above).
- `replace_regex`: BIGNUMERIC(p,s) -> decimal (flagged above).
- `replace_regex`: bare BIGNUMERIC -> decimal (flagged above).
- `replace_regex`: NUMERIC(p,s) / DECIMAL(p,s) -> decimal.
- `replace_regex`: bare NUMERIC / DECIMAL -> decimal.
- `replace_regex`: INT64 / INT / INTEGER -> long.
- `replace_regex`: FLOAT64 / FLOAT -> double.
- `replace_regex`: BOOL / BOOLEAN -> bool.
- `replace_regex`: STRING(n) -> string.
- `replace_regex`: bare STRING -> string.
- `replace_regex`: BYTES(n) / BYTES -> byte[].
- `replace_regex`: TIMESTAMP -> DateTime.
- `replace_regex`: DATETIME -> DateTime.
- `replace_regex`: DATE -> DateOnly.
- `replace_regex`: TIME -> TimeOnly.
- `replace_regex`: JSON -> string (flagged above).
- `replace_regex`: ARRAY<T> -> List<T> (flagged above).
- `replace_regex`: Rewrite nullable columns into a C# auto-property with a nullable type.
- `replace_regex`: Rewrite non-nullable columns into a C# auto-property.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier).
- `to_pascal`: PascalCase each property name (the column identifier).
- `replace_regex`: Collapse multiple blank lines to a single blank between declarations.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.
- `replace_regex`: Prepend the using directives the generated types need (System for the date and time kinds, System.Collections.Generic for the mapped repeated and record columns). Last, so the type-map regexes never touch the header.

## Tags

`sql` `bigquery` `csharp` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
