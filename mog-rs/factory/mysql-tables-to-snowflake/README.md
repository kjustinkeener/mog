# MySQL SHOW CREATE TABLE to Snowflake

Convert a MySQL table export to Snowflake DDL

Convert MySQL 'SHOW CREATE TABLE' output to Snowflake. Strips MySQL storage noise (ENGINE/CHARSET/COLLATE, backticks, DEFAULT NULL, ON UPDATE), then maps MySQL types to Snowflake: tinyint(1)->BOOLEAN, datetime->TIMESTAMP_NTZ, timestamp->TIMESTAMP_LTZ, text->VARCHAR, json->VARIANT, enum(...)->VARCHAR (value set flagged), and AUTO_INCREMENT->AUTOINCREMENT. Minimal-diff, NOT a semantic transpiler. Validated by executing the result in a real Snowflake engine (fakesnow).

## Run

```
mog -m mysql-tables-to-snowflake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `products` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `sku` varchar(40) NOT NULL,
  `name` varchar(200) DEFAULT NULL,
  `description` text,
  `price` decimal(18,2) DEFAULT '0.00',
  `in_stock` tinyint(1) DEFAULT '1',
  `attrs` json DEFAULT NULL,
  `status` enum('active','discontinued') DEFAULT 'active',
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
```

Output:

```
CREATE TABLE products (
  id bigint NOT NULL AUTOINCREMENT,
  sku varchar(40) NOT NULL,
  name varchar(200),
  description VARCHAR,
  price decimal(18,2) DEFAULT '0.00',
  in_stock BOOLEAN DEFAULT '1',
  attrs VARIANT,
  status VARCHAR DEFAULT 'active', -- TODO(mog): MySQL ENUM converted to VARCHAR; the allowed-value constraint was lost (add a CHECK if needed)
  created_at TIMESTAMP_NTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP_LTZ NULL DEFAULT CURRENT_TIMESTAMP, -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  PRIMARY KEY (id)
)
```

## Pipeline

- `run_mog`
- `replace_regex`: tinyint(1) is MySQL's boolean -> BOOLEAN.
- `replace_regex`: datetime (wall-clock) -> TIMESTAMP_NTZ.
- `replace_regex`: timestamp (instant) -> TIMESTAMP_LTZ.
- `replace_regex`: text -> VARCHAR.
- `replace_regex`: json -> VARIANT.
- `flag_matching`: Flag ENUM (Snowflake has none; value constraint is lost).
- `replace_regex`: enum(...) -> VARCHAR.
- `replace`: MySQL AUTO_INCREMENT -> Snowflake AUTOINCREMENT.

## Tags

`sql` `mysql` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
