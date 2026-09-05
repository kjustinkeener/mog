# Strip ClickHouse SHOW CREATE TABLE noise

Remove ClickHouse engine, physical, and identifier noise from SHOW CREATE

Strip the ClickHouse-specific rendering that SHOW CREATE TABLE adds, so a downstream dialect converter sees a clean CREATE TABLE column list. Normalizes to LF, drops backtick identifier quoting, unqualifies the CREATE TABLE database prefix (default.table -> table), turns the closing engine paren into a statement terminator, and removes the physical/storage clauses that have no portable target-DDL equivalent: ENGINE=..., PRIMARY KEY (ClickHouse sparse index), SAMPLE BY, TTL, SETTINGS, and any table-level COMMENT. The layout-encoding ORDER BY (sort key) and PARTITION BY clauses are preserved as -- TODO(mog): comments rather than silently dropped. Target-agnostic; run it first. Leaves ClickHouse column TYPES intact for the downstream type map.

## Run

```
mog -m strip-clickhouse-showcreate-noise <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE default.ch2pg_orders
(
    `order_id` UInt64,
    `customer_id` Int64,
    `store_id` Int32,
    `region_code` Int16,
    `is_paid` UInt8 DEFAULT 0,
    `status` LowCardinality(String),
    `order_ref` FixedString(12),
    `notes` Nullable(String),
    `coupon` LowCardinality(Nullable(String)),
    `quantity` UInt16,
    `item_count` UInt32,
    `big_seq` UInt64,
    `amount` Decimal(18, 2),
    `discount` Float32,
    `tax_rate` Float64,
    `created_at` DateTime64(3, 'UTC'),
```

_(... 12 more line(s))_

Output:

```
CREATE TABLE ch2pg_orders
(
    order_id UInt64,
    customer_id Int64,
    store_id Int32,
    region_code Int16,
    is_paid UInt8 DEFAULT 0,
    status LowCardinality(String),
    order_ref FixedString(12),
    notes Nullable(String),
    coupon LowCardinality(Nullable(String)),
    quantity UInt16,
    item_count UInt32,
    big_seq UInt64,
    amount Decimal(18, 2),
    discount Float32,
    tax_rate Float64,
    created_at DateTime64(3, 'UTC'),
```

_(... 9 more line(s))_

## Pipeline

- `eol_lf`: Normalize to LF so every later regex is reliable.
- `replace`: Drop backtick identifier quoting (ClickHouse quotes every column name).
- `replace_regex`: Unqualify the CREATE TABLE database prefix (e.g. default.orders -> orders); resolves via search_path in the target.
- `replace_regex`: Preserve the ClickHouse ORDER BY (sort / sparse-primary key) as a TODO comment; it encodes physical layout with no Postgres table-DDL equivalent.
- `replace_regex`: Preserve the ClickHouse PARTITION BY expression as a TODO comment (physical partitioning is not part of a Postgres CREATE TABLE).
- `replace_regex`: Drop the ENGINE = ... clause (MergeTree/ReplacingMergeTree/Log/etc); no Postgres equivalent.
- `replace_regex`: Drop the ClickHouse table-level PRIMARY KEY (a sparse index, not a uniqueness constraint).
- `replace_regex`: Drop SAMPLE BY (sampling expression; no Postgres equivalent).
- `replace_regex`: Drop table-level TTL (data-expiry policy; use a Postgres job/partition drop instead).
- `replace_regex`: Drop the SETTINGS clause (index_granularity, etc; engine tuning with no Postgres analog).
- `replace_regex`: Drop a trailing table-level COMMENT clause (kept runnable; re-add via COMMENT ON TABLE by hand if desired).
- `replace_regex`: Turn the closing engine paren into a statement terminator.
- `replace_regex`: Collapse blank-line runs left by removed clauses to a single blank line.
- `trim_whitespace_right`: Tidy any trailing spaces the removals left.

## Tags

`sql` `clickhouse` `strip` `migration` `fragment`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
