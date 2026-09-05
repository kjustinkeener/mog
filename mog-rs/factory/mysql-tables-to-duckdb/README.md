# MySQL SHOW CREATE TABLE to DuckDB

Convert a MySQL table export to DuckDB DDL

Convert MySQL 'SHOW CREATE TABLE' output to DuckDB. Strips MySQL storage noise (ENGINE/CHARSET/COLLATE, backticks, DEFAULT NULL, ON UPDATE), maps types (tinyint(1)->BOOLEAN, datetime->TIMESTAMP, timestamp->TIMESTAMPTZ, json->JSON, enum(...)->VARCHAR flagged), and drops AUTO_INCREMENT (DuckDB uses a sequence; flagged). Minimal-diff. Validated by executing the result in a real DuckDB engine.

## Run

```
mog -m mysql-tables-to-duckdb <file>
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
-- TODO(mog): MySQL dropped; create a DuckDB SEQUENCE and DEFAULT nextval(...) if needed
  id bigint NOT NULL,
  sku varchar(40) NOT NULL,
  name varchar(200),
  description text,
  price decimal(18,2) DEFAULT '0.00',
  in_stock BOOLEAN DEFAULT '1',
  attrs JSON,
  status VARCHAR DEFAULT 'active', -- TODO(mog): MySQL ENUM mapped to VARCHAR; the allowed-value constraint was lost
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NULL DEFAULT CURRENT_TIMESTAMP, -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  PRIMARY KEY (id)
)
```

## Pipeline

- `run_mog`
- `replace_regex`: tinyint(1) -> BOOLEAN.
- `replace_regex`: timestamp (instant) -> TIMESTAMPTZ. Before datetime so it does not re-match.
- `replace_regex`: datetime (wall-clock) -> TIMESTAMP.
- `replace_regex`: json -> JSON.
- `flag_matching`: Flag ENUM (mapped to VARCHAR; value constraint lost).
- `replace_regex`: enum(...) -> VARCHAR.
- `flag_matching`: Flag AUTO_INCREMENT (DuckDB uses a sequence).
- `replace`: Drop AUTO_INCREMENT (no DuckDB keyword).

## Tags

`sql` `mysql` `duckdb` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
