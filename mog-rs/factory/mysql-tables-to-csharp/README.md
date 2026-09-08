# MySQL SHOW CREATE TABLE to C# classes

Generate C# POCO classes from MySQL SHOW CREATE TABLE output

Turn real MySQL 'SHOW CREATE TABLE' output into idiomatic C# POCO classes, one class per table. Strips the MySQL storage rendering (backticks, ENGINE/CHARSET/COLLATE, DEFAULT NULL, ON UPDATE), drops the inline index and constraint lines (PRIMARY KEY, KEY, UNIQUE KEY, CONSTRAINT, FOREIGN KEY), re-indents the column body from MySQL's two-space to four-space, then rewrites each CREATE TABLE into a 'public class' with one auto-property per column. MySQL types map to C# types (bigint->long, int/integer/mediumint->int, smallint->short, tinyint(1)->bool, tinyint->sbyte, bit->bool, varchar/char/text family->string, decimal/numeric->decimal, double/float->double, datetime/timestamp->DateTime, date->DateOnly, time->TimeOnly, year->int, blob/binary/varbinary family->byte[], json->string). Nullability is inferred: MySQL columns are nullable unless declared NOT NULL, so non-NOT-NULL columns get a C# nullable type ('?'); AUTO_INCREMENT, UNSIGNED, ZEROFILL, DEFAULT clauses and per-column CHARACTER SET/COLLATE are dropped. enum(...) and set(...) map to string and are flagged with a // TODO(mog): comment because their value set is lost. Table and column identifiers are PascalCased. The MySQL-source sibling of postgres-tables-to-csharp. Assumes LF.

## Run

```
mog -m mysql-tables-to-csharp <file>
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
public class Actor
{
    public int ActorId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime LastUpdate { get; set; }
}

public class Film
{
    public int FilmId { get; set; }
    public string Title { get; set; }
    public string? Description { get; set; }
    public int? ReleaseYear { get; set; }
    public int LanguageId { get; set; }
    public int? OriginalLanguageId { get; set; }
    public sbyte RentalDuration { get; set; }
    public decimal RentalRate { get; set; }
```

_(... 34 more line(s))_

## Steps

- `run_mog`: Remove MySQL storage noise: backticks, ENGINE/CHARSET/COLLATE, DEFAULT NULL, ON UPDATE.
- `replace_regex`: Drop the SQL-comment TODO the strip fragment stamped for ON UPDATE (not valid C#).
- `replace_regex`: Drop per-column CHARACTER SET clause.
- `replace_regex`: Drop per-column COLLATE clause.
- `replace_regex`: Drop the inline PRIMARY KEY / UNIQUE KEY / FULLTEXT KEY / KEY / CONSTRAINT / FOREIGN KEY lines (whole line).
- `replace_regex`: Drop the MySQL AUTO_INCREMENT flag (no C# equivalent on a POCO).
- `replace_regex`: Drop UNSIGNED / ZEROFILL numeric modifiers.
- `replace_regex`: Drop an explicit NULL nullability marker (leave NOT NULL intact).
- `replace_regex`: Re-indent the column body from MySQL's two-space to the four-space the codegen expects.
- `replace_regex`: Rewrite each 'CREATE TABLE <tbl> (' into a C# class header + opening brace.
- `replace_regex`: Rewrite each table-closing ')' into a closing brace.
- `replace_regex`: Drop DEFAULT clauses from column lines (literals, CURRENT_TIMESTAMP).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag MySQL enum columns (mapped to string; value set lost).
- `flag_matching`: Flag MySQL set columns (mapped to string; multi-value set lost).
- `replace_regex`: tinyint(1) is MySQL's boolean -> bool.
- `replace_regex`: tinyint(n) -> sbyte.
- `replace_regex`: smallint -> short.
- `replace_regex`: mediumint -> int.
- `replace_regex`: bigint -> long.
- `replace_regex`: integer / int -> int.
- `replace_regex`: decimal(p,s) / numeric(p,s) -> decimal.
- `replace_regex`: bare decimal / numeric -> decimal.
- `replace_regex`: double / double precision -> double.
- `replace_regex`: float -> double.
- `replace_regex`: real -> double.
- `replace_regex`: datetime -> DateTime.
- `replace_regex`: timestamp -> DateTime.
- `replace_regex`: year -> int.
- `replace_regex`: date -> DateOnly (runs after datetime).
- `replace_regex`: time -> TimeOnly (runs after datetime/timestamp).
- `replace_regex`: enum(...) -> string (flagged above).
- `replace_regex`: set(...) -> string (flagged above).
- `replace_regex`: varchar(n) / char(n) -> string.
- `replace_regex`: bare varchar / char -> string.
- `replace_regex`: text family (tinytext/mediumtext/longtext/text) -> string.
- `replace_regex`: blob family (tinyblob/mediumblob/longblob/blob) -> byte[].
- `replace_regex`: varbinary(n) / binary(n) / bare -> byte[].
- `replace_regex`: bit -> bool.
- `replace_regex`: json -> string.
- `replace_regex`: Rewrite nullable columns into a C# auto-property with a nullable type.
- `replace_regex`: Rewrite non-nullable columns into a C# auto-property.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier).
- `to_pascal`: PascalCase each property name (the column identifier).
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `mysql` `csharp` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
