# Strip MySQL SHOW CREATE TABLE noise

Remove MySQL storage clauses, backticks, and MySQL-only column bits

Strip the MySQL-specific rendering that SHOW CREATE TABLE adds, so a downstream dialect converter sees a clean CREATE TABLE. Removes the trailing ENGINE=... DEFAULT CHARSET=... COLLATE=... storage clause, the backtick identifier quoting, the redundant DEFAULT NULL, and the MySQL-only ON UPDATE CURRENT_TIMESTAMP auto-refresh (flagged, since it has no portable equivalent). Target-agnostic; run it first. Assumes LF.

## Run

```
mog -m strip-mysql-ddl-noise <file>
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
  id bigint NOT NULL AUTO_INCREMENT,
  sku varchar(40) NOT NULL,
  name varchar(200),
  description text,
  price decimal(18,2) DEFAULT '0.00',
  in_stock tinyint(1) DEFAULT '1',
  attrs json,
  status enum('active','discontinued') DEFAULT 'active',
  created_at datetime DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamp NULL DEFAULT CURRENT_TIMESTAMP, -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  PRIMARY KEY (id)
)
```

## Pipeline

- `eol_lf`: Normalize to LF.
- `flag_matching`: Flag the MySQL-only ON UPDATE auto-refresh before removing it.
- `replace_regex`: Remove the ON UPDATE CURRENT_TIMESTAMP clause.
- `replace_regex`: Drop the trailing ENGINE / CHARSET / COLLATE storage clause.
- `replace`: Drop backtick identifier quoting.
- `replace_regex`: Drop redundant DEFAULT NULL (columns are nullable by default).

## Tags

`sql` `mysql` `strip` `migration` `fragment`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
