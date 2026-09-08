# BigQuery tables to Python dataclasses

Generate Python @dataclass classes from a BigQuery DDL export

Generate Python standard-library @dataclass classes from a BigQuery DDL export (INFORMATION_SCHEMA.TABLES.ddl or 'bq show'), one class per table. Discards non-table statements (CREATE SCHEMA, views, ALTERs) and BigQuery's physical-layout trailers (PARTITION BY, CLUSTER BY, table and column OPTIONS). Column names stay snake_case (BigQuery's own convention, and the keys the client returns); only the class name is PascalCased. BigQuery types map to Python types (INT64 -> int, FLOAT64 -> float, NUMERIC/BIGNUMERIC -> Decimal, BOOL -> bool, STRING -> str, BYTES -> bytes, DATE -> date, TIME -> time, DATETIME/TIMESTAMP -> datetime, JSON -> dict[str, Any], STRUCT<...> -> dict[str, Any], ARRAY<T> -> list[T]). BigQuery columns are NULLABLE by default -- only an explicit NOT NULL makes a column required -- so the 'type | None' annotation is driven by the ABSENCE of NOT NULL; ARRAY columns are never null. Emits a fixed stdlib import header with 'from __future__ import annotations' so the module runs on Python 3.7+. Semi-structured and lossy types (JSON, STRUCT, BIGNUMERIC) are flagged with a # TODO(mog): comment. Assumes LF.

## Run

```
mog -m bigquery-tables-to-python-dataclass <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
-- BigQuery DDL export
-- Source: SELECT ddl FROM `mog-demo.analytics.INFORMATION_SCHEMA.TABLES`

CREATE SCHEMA IF NOT EXISTS `mog-demo.analytics`;

DROP TABLE IF EXISTS `mog-demo.analytics.bqpy_scratch`;

-- Table: bqpy_actor

CREATE OR REPLACE TABLE `mog-demo.analytics.bqpy_actor`
(
    `actor_id` INT64 NOT NULL,
    `first_name` STRING NOT NULL,
    `last_name` STRING NOT NULL,
    `last_update` TIMESTAMP NOT NULL
);

-- Table: bqpy_staff
```

_(... 55 more line(s))_

Output:

```
from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from typing import Any

@dataclass
class BqpyActor:
    actor_id: int
    first_name: str
    last_name: str
    last_update: datetime

@dataclass
class BqpyStaff:
    staff_id: int
    first_name: str
    last_name: str
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
- `flag_matching`: Flag BigQuery JSON columns: mapped to dict.
- `flag_matching`: Flag BigQuery STRUCT columns: mapped to dict.
- `flag_matching`: Flag BigQuery ARRAY columns: mapped to a Python list.
- `flag_matching`: Flag BigQuery BIGNUMERIC columns: mapped to Decimal.
- `replace_regex`: STRUCT<...> -> dict (flagged above).
- `replace_regex`: BIGNUMERIC(p,s) -> Decimal (flagged above).
- `replace_regex`: bare BIGNUMERIC -> Decimal (flagged above).
- `replace_regex`: NUMERIC(p,s) / DECIMAL(p,s) -> Decimal.
- `replace_regex`: bare NUMERIC / DECIMAL -> Decimal.
- `replace_regex`: INT64 / INT / INTEGER -> int.
- `replace_regex`: FLOAT64 / FLOAT -> float.
- `replace_regex`: BOOL / BOOLEAN -> bool.
- `replace_regex`: STRING(n) -> str.
- `replace_regex`: bare STRING -> str.
- `replace_regex`: BYTES(n) / BYTES -> bytes.
- `replace_regex`: TIMESTAMP -> datetime.
- `replace_regex`: DATETIME -> datetime.
- `replace_regex`: DATE -> date (datetime.date).
- `replace_regex`: TIME -> time (datetime.time).
- `replace_regex`: JSON -> dict (flagged above).
- `replace_regex`: ARRAY<T> -> list[T] (flagged above).
- `replace_regex`: Rewrite nullable columns into a dataclass field with a 'type | None' annotation.
- `replace_regex`: Rewrite non-nullable columns into a dataclass field with a plain annotation.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier); field names stay snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between declarations.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.
- `replace_regex`: Widen the bare dict annotation to dict[str, Any] so the output passes a strict type checker. Runs after nullability, and is anchored to the annotation so it never touches the word 'dict' inside a TODO comment.
- `replace_regex`: Prepend the fixed stdlib import header (last, so the type-map regexes never touch 'date'/'time'/'Decimal' in the imports).

## Tags

`sql` `bigquery` `python` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
