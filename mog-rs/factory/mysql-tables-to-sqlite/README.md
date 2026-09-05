# MySQL SHOW CREATE TABLE to SQLite

Convert a MySQL table export to SQLite DDL

Convert MySQL SHOW CREATE TABLE output to SQLite: strips MySQL storage noise, drops AUTO_INCREMENT (SQLite uses INTEGER PRIMARY KEY AUTOINCREMENT; flagged), and maps enum(...)->TEXT and json->TEXT (SQLite has flexible typing, so most types pass through as affinities). Validated by executing the result in a real SQLite engine.

## Run

```
mog -m mysql-tables-to-sqlite <file>
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
-- TODO(mog): MySQL dropped; SQLite needs INTEGER PRIMARY KEY AUTOINCREMENT
  id bigint NOT NULL,
  sku varchar(40) NOT NULL,
  name varchar(200),
  description text,
  price decimal(18,2) DEFAULT '0.00',
  in_stock INTEGER DEFAULT '1',
  attrs TEXT,
  status TEXT DEFAULT 'active',
  created_at datetime DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamp NULL DEFAULT CURRENT_TIMESTAMP, -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  PRIMARY KEY (id)
)
```

## Pipeline

- `run_mog`
- `flag_matching`
- `replace`
- `replace_regex`
- `replace_regex`
- `replace_regex`

## Tags

`sql` `mysql` `sqlite` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
