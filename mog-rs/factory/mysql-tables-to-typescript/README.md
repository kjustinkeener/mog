# MySQL SHOW CREATE TABLE to TypeScript interfaces

Generate TypeScript interfaces from MySQL SHOW CREATE TABLE output

Turn real MySQL 'SHOW CREATE TABLE' output into idiomatic TypeScript 'export interface' declarations, one interface per table, 2-space indented. Strips the MySQL storage rendering (backticks, ENGINE/CHARSET/COLLATE, DEFAULT NULL, ON UPDATE), drops the inline index and constraint lines (PRIMARY KEY, KEY, UNIQUE KEY, FULLTEXT KEY, CONSTRAINT, FOREIGN KEY), then rewrites each CREATE TABLE into an 'export interface' with one member per column. Member names keep the DB's snake_case exactly; only the interface name is PascalCased. MySQL types map to TypeScript types (tinyint(1)->boolean before the generic tinyint rule; tinyint/smallint/mediumint/int/integer/bigint with or without ' unsigned'->number; year->number; decimal/numeric/float/double/real->number; bit->boolean; char/varchar/tinytext/text/mediumtext/longtext->string; date/datetime/timestamp/time->string as ISO-8601 strings; binary/varbinary/tinyblob/blob/mediumblob/longblob->Uint8Array; json->unknown). Nullability is inferred: MySQL columns are nullable unless declared NOT NULL, so any column line lacking NOT NULL gets a '<type> | null' union (no optional '?'). The ' unsigned', CHARACTER SET, COLLATE, AUTO_INCREMENT and DEFAULT clauses are dropped. enum(...) and set(...) map to string and are flagged with a // TODO(mog): comment because their value set is lost. The TypeScript sibling of mysql-tables-to-csharp. Assumes LF.

## Run

```
mog -m mysql-tables-to-typescript <file>
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
export interface Actor {
  actor_id: number;
  first_name: string;
  last_name: string;
  last_update: string;
}

export interface Film {
  film_id: number;
  title: string;
  description: string | null;
  release_year: number | null;
  language_id: number;
  original_language_id: number | null;
  rental_duration: number;
  rental_rate: number;
  length: number | null;
  replacement_cost: number;
```

_(... 30 more line(s))_

## Pipeline

- `run_mog`: Remove MySQL storage noise: backticks, ENGINE/CHARSET/COLLATE, DEFAULT NULL, ON UPDATE.
- `replace_regex`: Drop the SQL-comment TODO the strip fragment stamped for ON UPDATE (not valid TS).
- `replace_regex`: Drop per-column CHARACTER SET clause.
- `replace_regex`: Drop per-column COLLATE clause.
- `replace_regex`: Drop the inline PRIMARY KEY / UNIQUE KEY / FULLTEXT KEY / KEY / CONSTRAINT / FOREIGN KEY lines (whole line).
- `replace_regex`: Drop the MySQL AUTO_INCREMENT flag.
- `replace_regex`: Drop UNSIGNED / ZEROFILL numeric modifiers.
- `replace_regex`: Drop an explicit NULL nullability marker (leave NOT NULL intact).
- `replace_regex`: Rewrite each 'CREATE TABLE <tbl> (' into a TypeScript interface header + opening brace.
- `replace_regex`: Rewrite each table-closing ')' into a closing brace.
- `replace_regex`: Drop DEFAULT clauses from column lines (literals, CURRENT_TIMESTAMP).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag MySQL enum columns (mapped to string; value set lost).
- `flag_matching`: Flag MySQL set columns (mapped to string; multi-value set lost).
- `replace_regex`: tinyint(1) is MySQL's boolean -> boolean.
- `replace_regex`: integer family (tinyint/smallint/mediumint/int/integer/bigint) -> number.
- `replace_regex`: year -> number.
- `replace_regex`: decimal(p,s) / numeric(p,s) -> number.
- `replace_regex`: bare decimal / numeric -> number.
- `replace_regex`: double / double precision -> number.
- `replace_regex`: float -> number.
- `replace_regex`: real -> number.
- `replace_regex`: bit -> boolean.
- `replace_regex`: varchar(n) / char(n) -> string.
- `replace_regex`: bare varchar / char -> string.
- `replace_regex`: text family (tinytext/mediumtext/longtext/text) -> string.
- `replace_regex`: datetime -> string (ISO-8601).
- `replace_regex`: timestamp -> string (ISO-8601).
- `replace_regex`: date -> string (ISO-8601, runs after datetime).
- `replace_regex`: time -> string (ISO-8601, runs after datetime/timestamp).
- `replace_regex`: enum(...) -> string (flagged above).
- `replace_regex`: set(...) -> string (flagged above).
- `replace_regex`: varbinary(n) / binary(n) / bare -> Uint8Array.
- `replace_regex`: blob family (tinyblob/mediumblob/longblob/blob) -> Uint8Array.
- `replace_regex`: json -> unknown.
- `replace_regex`: Rewrite nullable columns into a TypeScript member with a '| null' union.
- `replace_regex`: Rewrite non-nullable columns into a TypeScript member.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each interface name (the table identifier); members keep snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between interfaces.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `mysql` `typescript` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
