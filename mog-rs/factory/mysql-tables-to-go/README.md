# MySQL SHOW CREATE tables to Go structs

Generate Go structs (with db tags) from MySQL SHOW CREATE TABLE output

Turn real 'SHOW CREATE TABLE' / mysqldump DDL into idiomatic Go type declarations, one 'type <Table> struct' per table. Strips the MySQL plumbing (ENGINE/CHARSET/COLLATE, backtick quoting, AUTO_INCREMENT, unsigned/zerofill), drops key and constraint lines, then rewrites each CREATE TABLE into a struct with one exported field per column. Field names are PascalCased for export, and the EXACT database column name is preserved losslessly in a `db:"..."` struct tag (compatible with sqlx / pgx). MySQL types map to Go types (tinyint(1)->bool, tinyint->int8, smallint/year->int16, mediumint/int->int32, bigint->int64, float->float32, double->float64, char/varchar/text->string, datetime/timestamp/date/time->time.Time, blob/binary->[]byte, json->json.RawMessage). decimal/numeric map to string (Go has no native decimal) and are flagged. Nullable columns become pointer types (*T); nilable types (slices, json.RawMessage) stay as-is. enum/set map to a best-effort string and are flagged inline with a // TODO(mog): comment. Emits the struct declarations only -- add your own 'package' clause and imports (time, encoding/json), then run gofmt to align fields. The Go sibling of mysql-tables-to-typescript. Assumes LF.

## Run

```
mog -m mysql-tables-to-go <file>
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
type Actor struct {
	ActorId int32 `db:"actor_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	LastUpdate time.Time `db:"last_update"`
}

type Film struct {
	FilmId int32 `db:"film_id"`
	Title string `db:"title"`
	Description *string `db:"description"`
	ReleaseYear *int16 `db:"release_year"`
	LanguageId int32 `db:"language_id"`
	OriginalLanguageId *int32 `db:"original_language_id"`
	RentalDuration int8 `db:"rental_duration"`
	RentalRate string `db:"rental_rate"` // TODO(mog): MySQL exact-precision column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for arithmetic.
	Length *int16 `db:"length"`
	ReplacementCost string `db:"replacement_cost"` // TODO(mog): MySQL exact-precision column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for arithmetic.
```

_(... 30 more line(s))_

## Pipeline

- `run_mog`: Strip mysqldump / SHOW CREATE noise (ENGINE, backtick quoting, AUTO_INCREMENT, unsigned).
- `replace_regex`: Remove any leftover mog TODO markers from the strip fragment.
- `replace_regex`: Drop column-level CHARACTER SET clauses.
- `replace_regex`: Drop column-level COLLATE clauses.
- `replace_regex`: Drop key / constraint lines (PRIMARY KEY, UNIQUE KEY, KEY, FOREIGN KEY, CONSTRAINT).
- `replace_regex`: Remove AUTO_INCREMENT.
- `replace_regex`: Remove unsigned / zerofill numeric modifiers.
- `replace_regex`: Remove a bare NULL marker (keep NOT NULL).
- `replace_regex`: Rewrite each 'CREATE TABLE <tbl> (' into a Go struct header + opening brace.
- `replace_regex`: Rewrite each table-closing ')' into a closing brace.
- `replace_regex`: Drop DEFAULT clauses from column lines.
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (2-space indent, NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag MySQL enum columns.
- `flag_matching`: Flag MySQL set columns.
- `flag_matching`: Flag MySQL decimal/numeric columns.
- `replace_regex`: tinyint(1) -> bool.
- `replace_regex`: bigint -> int64.
- `replace_regex`: smallint -> int16.
- `replace_regex`: mediumint -> int32.
- `replace_regex`: tinyint -> int8.
- `replace_regex`: integer / int -> int32.
- `replace_regex`: year -> int16.
- `replace_regex`: decimal/numeric(p,s) -> string (flagged above).
- `replace_regex`: bare decimal/numeric -> string (flagged above).
- `replace_regex`: double / double precision -> float64.
- `replace_regex`: float -> float32.
- `replace_regex`: real -> float64.
- `replace_regex`: bit -> []byte.
- `replace_regex`: varchar/char(n) -> string.
- `replace_regex`: bare varchar/char -> string.
- `replace_regex`: text family -> string.
- `replace_regex`: datetime -> time.Time (via sentinel).
- `replace_regex`: timestamp -> time.Time (via sentinel).
- `replace_regex`: date -> time.Time (via sentinel).
- `replace_regex`: time -> time.Time (via sentinel).
- `replace_regex`: enum(...) -> string (flagged above).
- `replace_regex`: set(...) -> string (flagged above).
- `replace_regex`: varbinary/binary -> []byte.
- `replace_regex`: blob family -> []byte.
- `replace_regex`: json -> json.RawMessage.
- `replace_regex`: Resolve the time sentinel to time.Time (kept separate to avoid re-matching the 'time' keyword).
- `replace_regex`: Rewrite nullable columns into a pointer-typed Go field carrying a db tag placeholder.
- `replace_regex`: Rewrite non-nullable columns into a Go field carrying a db tag placeholder.
- `replace_regex`: Nilable types (slices, json.RawMessage) don't need a pointer for nullability -- drop it.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each struct name (the table identifier).
- `to_pascal`: PascalCase each field name (the column identifier); the db tag keeps the exact column name.
- `replace_regex`: Resolve the db tag placeholder into a real Go struct tag.
- `replace_regex`: Convert the 2-space column indent to a tab (Go convention).
- `replace_regex`: Collapse multiple blank lines to a single blank between structs.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `mysql` `go` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
