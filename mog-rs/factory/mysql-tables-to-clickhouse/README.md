# MySQL SHOW CREATE TABLE to ClickHouse

Convert a MySQL table export into runnable ClickHouse MergeTree DDL

Convert MySQL 'SHOW CREATE TABLE' output to ClickHouse for analytics ingestion. Strips MySQL storage noise (ENGINE/CHARSET/COLLATE, backticks, DEFAULT NULL, ON UPDATE), maps every MySQL type to a ClickHouse type (varchar/text->String, int->Int32, bigint->Int64, tinyint(1)->UInt8, tinyint->Int8, decimal(p,s)->Decimal(p,s), double->Float64, datetime/timestamp->DateTime64(3), date->Date, enum(...)->String), and applies the two ClickHouse-specific structural transforms: nullable columns are wrapped as Nullable(<type>) (columns are NOT NULL by default) and the MySQL PRIMARY KEY becomes 'ENGINE = MergeTree' + 'ORDER BY (<pk cols>)'. Non-portable constructs are flagged with -- TODO(mog): AUTO_INCREMENT, ENUM (constraint lost), ON UPDATE, and UNIQUE/secondary indexes (ClickHouse does not enforce uniqueness). Minimal-diff. Validated by a live round-trip against a real ClickHouse engine.

## Run

```
mog -m mysql-tables-to-clickhouse <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `my2ch_orders` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `order_no` varchar(40) NOT NULL,
  `customer_name` varchar(200) DEFAULT NULL,
  `notes` text,
  `amount` decimal(18,2) NOT NULL DEFAULT '0.00',
  `is_paid` tinyint(1) NOT NULL DEFAULT '0',
  `priority` tinyint DEFAULT '5',
  `quantity` int NOT NULL DEFAULT '1',
  `discount_pct` double DEFAULT NULL,
  `status` enum('pending','shipped','cancelled') NOT NULL DEFAULT 'pending',
  `order_date` date DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_order_no` (`order_no`),
  KEY `idx_customer` (`customer_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
```

Output:

```
CREATE TABLE my2ch_orders (
  id Int64, -- TODO(mog): MySQL AUTO_INCREMENT has no ClickHouse equivalent (use a generated UUID, a sequence table, or an application-supplied id)
  order_no String,
  customer_name Nullable(String),
  notes Nullable(String),
  amount Decimal(18,2) DEFAULT '0.00',
  is_paid UInt8 DEFAULT '0',
  priority Nullable(Int8) DEFAULT '5',
  quantity Int32 DEFAULT '1',
  discount_pct Nullable(Float64),
  status String DEFAULT 'pending', -- TODO(mog): MySQL ENUM mapped to String; the allowed-value constraint was lost (use a ClickHouse Enum8/Enum16 or a CHECK)
  order_date Nullable(Date),
  created_at DateTime64(3) DEFAULT CURRENT_TIMESTAMP,
  updated_at Nullable(DateTime64(3)) DEFAULT CURRENT_TIMESTAMP -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  -- TODO(mog): MySQL UNIQUE KEY uq_order_no (order_no) dropped; ClickHouse does not enforce uniqueness (dedupe with ReplacingMergeTree or enforce upstream)
  -- TODO(mog): MySQL secondary index idx_customer (customer_name) dropped; ClickHouse uses the ORDER BY key and data-skipping indexes instead
) ENGINE = MergeTree
ORDER BY (id)
```

## Steps

- `run_mog`: Remove MySQL SHOW CREATE rendering: backticks, ENGINE/CHARSET/COLLATE, DEFAULT NULL, and flag+drop ON UPDATE.
- `flag_matching`: Flag AUTO_INCREMENT before dropping it: ClickHouse has no autoincrement.
- `replace_regex`: Drop the AUTO_INCREMENT column marker (only the column occurrence, which precedes the comma, so the TODO note keeps the word).
- `flag_matching`: Flag ENUM (mapped to String; the allowed-value constraint is lost).
- `replace_regex`: enum(...) -> String.
- `replace_regex`: timestamp (instant) -> DateTime64(3). Runs before datetime; both map to the same token, whole-word so it never re-bites.
- `replace_regex`: datetime (wall-clock) -> DateTime64(3).
- `replace_regex`: date -> Date (after datetime/timestamp so it does not touch them).
- `replace_regex`: tinyint(1) is MySQL's boolean -> UInt8. Runs before the bare tinyint map.
- `replace_regex`: tinyint -> Int8.
- `replace_regex`: smallint -> Int16.
- `replace_regex`: mediumint -> Int32.
- `replace_regex`: bigint -> Int64. Runs before the bare int map.
- `replace_regex`: int -> Int32 (whole word so it never bites bigint/tinyint or an Int64 token).
- `replace_regex`: decimal(p,s) -> Decimal(p,s) (ClickHouse spelling; precision/scale preserved).
- `replace_regex`: numeric(p,s) -> Decimal(p,s).
- `replace_regex`: double / double precision -> Float64.
- `replace_regex`: float -> Float32.
- `replace_regex`: varchar(n)/char(n) -> String (ClickHouse String is unbounded).
- `replace_regex`: text family (text/tinytext/mediumtext/longtext) -> String.
- `replace_regex`: blob/varbinary/binary family -> String (ClickHouse stores bytes as String).
- `replace_regex`: json -> String (ClickHouse String is the portable, always-runnable choice).
- `replace_regex`: Wrap every nullable column's type in Nullable(...). A column line WITHOUT 'NOT NULL' is nullable in MySQL; ClickHouse needs the explicit Nullable() wrapper. NOT NULL columns are left bare (next steps drop the keyword).
- `replace_regex`: Drop the NOT NULL keyword (ClickHouse columns are non-null by default; NOT NULL columns are already bare).
- `replace_regex`: Drop the redundant explicit NULL modifier left by MySQL (the type is already Nullable()).
- `replace_regex`: UNIQUE KEY has no ClickHouse equivalent (uniqueness is not enforced): turn it into a TODO note.
- `replace_regex`: Secondary index KEY has no direct ClickHouse equivalent: turn it into a TODO note (ClickHouse uses the ORDER BY key and data-skipping indexes).
- `replace_regex`: The main structural transform: PRIMARY KEY (cols) -> a required MergeTree engine + ORDER BY (cols). Consumes the last column's trailing comma, preserves any TODO notes, and appends the engine clause after the closing paren. If a table has no PRIMARY KEY, add 'ENGINE = MergeTree ORDER BY tuple()' by hand.

## Tags

`sql` `mysql` `clickhouse` `convert` `migration` `analytics`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
