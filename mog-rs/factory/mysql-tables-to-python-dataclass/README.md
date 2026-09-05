# MySQL SHOW CREATE tables to Python dataclasses

Generate Python dataclasses from MySQL SHOW CREATE TABLE output

Turn real 'SHOW CREATE TABLE' / mysqldump DDL into idiomatic Python dataclasses, one '@dataclass' per table. Strips the MySQL plumbing (ENGINE/CHARSET/COLLATE, backtick quoting, AUTO_INCREMENT, unsigned/zerofill), drops key and constraint lines, then rewrites each CREATE TABLE into a dataclass with one 4-space-indented annotated attribute per column. Attribute names are kept EXACTLY as in the database (snake_case, PEP 8, lossless); only the class name is PascalCased. MySQL types map to Python types (tinyint(1)->bool, integer family->int, year->int, decimal/numeric->Decimal, float/double/real->float, char/varchar/text->str, datetime/timestamp->datetime, date->date, time->time, blob/binary->bytes, json->dict). Nullable columns (no NOT NULL) get a 'T | None' union. enum/set map to a best-effort str and are flagged inline with a # TODO(mog): comment. Emits a ready-to-import module with the needed 'from __future__ import annotations' + dataclass/datetime/decimal imports at the top. The Python sibling of mysql-tables-to-typescript. Assumes LF.

## Run

```
mog -m mysql-tables-to-python-dataclass <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `actor` (
  `actor_id` int unsigned NOT NULL AUTO_INCREMENT,
  `first_name` varchar(45) NOT NULL,
  `last_name` varchar(45) NOT NULL,
  `last_update` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`actor_id`),
  KEY `idx_actor_last_name` (`last_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3

CREATE TABLE `film` (
  `film_id` int unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(255) NOT NULL,
  `description` text,
  `release_year` year DEFAULT NULL,
  `language_id` int unsigned NOT NULL,
  `original_language_id` int unsigned DEFAULT NULL,
  `rental_duration` tinyint unsigned NOT NULL DEFAULT '3',
  `rental_rate` decimal(4,2) NOT NULL DEFAULT '4.99',
```

_(... 49 more line(s))_

Output:

```
from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal

@dataclass
class Actor:
    actor_id: int
    first_name: str
    last_name: str
    last_update: datetime

@dataclass
class Film:
    film_id: int
    title: str
    description: str | None
    release_year: int | None
```

_(... 35 more line(s))_

## Pipeline

- `run_mog`: Strip mysqldump / SHOW CREATE noise (ENGINE, backtick quoting, AUTO_INCREMENT, unsigned).
- `replace_regex`: Remove any leftover mog TODO markers from the strip fragment.
- `replace_regex`: Drop column-level CHARACTER SET clauses.
- `replace_regex`: Drop column-level COLLATE clauses.
- `replace_regex`: Drop key / constraint lines.
- `replace_regex`: Remove AUTO_INCREMENT.
- `replace_regex`: Remove unsigned / zerofill numeric modifiers.
- `replace_regex`: Remove a bare NULL marker (keep NOT NULL).
- `replace_regex`: Rewrite each 'CREATE TABLE <tbl> (' into a @dataclass header.
- `replace_regex`: Remove each table-closing ')'.
- `replace_regex`: Drop DEFAULT clauses from column lines.
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (2-space indent, NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag MySQL enum columns.
- `flag_matching`: Flag MySQL set columns.
- `replace_regex`: tinyint(1) -> bool.
- `replace_regex`: integer family -> int.
- `replace_regex`: year -> int.
- `replace_regex`: decimal/numeric(p,s) -> Decimal.
- `replace_regex`: bare decimal/numeric -> Decimal.
- `replace_regex`: double / double precision -> float.
- `replace_regex`: float -> float.
- `replace_regex`: real -> float.
- `replace_regex`: bit -> int.
- `replace_regex`: varchar/char(n) -> str.
- `replace_regex`: bare varchar/char -> str.
- `replace_regex`: text family -> str.
- `replace_regex`: datetime -> datetime.
- `replace_regex`: timestamp -> datetime.
- `replace_regex`: date -> date.
- `replace_regex`: time -> time.
- `replace_regex`: enum(...) -> str (flagged above).
- `replace_regex`: set(...) -> str (flagged above).
- `replace_regex`: varbinary/binary -> bytes.
- `replace_regex`: blob family -> bytes.
- `replace_regex`: json -> dict.
- `replace_regex`: Rewrite nullable columns into an annotated attribute with a '| None' union (4-space indent).
- `replace_regex`: Rewrite non-nullable columns into an annotated attribute (4-space indent).
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier); attributes stay snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.
- `replace_regex`: Prepend the future-annotations pragma, dataclass, and datetime/decimal imports.

## Tags

`sql` `mysql` `python` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
