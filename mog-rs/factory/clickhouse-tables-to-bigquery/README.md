# ClickHouse SHOW CREATE TABLE to BigQuery

Convert a ClickHouse table export to BigQuery DDL

Convert the output of ClickHouse 'SHOW CREATE TABLE' to runnable BigQuery DDL. (1) Rewrites CREATE TABLE to CREATE OR REPLACE TABLE, keeping the ClickHouse database qualifier as the BigQuery dataset, (2) strips the engine/physical clauses (ENGINE, SETTINGS, CODEC) and flags the ones carrying intent (PARTITION BY, PRIMARY KEY, ORDER BY, SAMPLE BY, TTL) as TODO(mog), suggesting BigQuery CLUSTER BY for an ORDER BY, (3) models nullability: bare columns become NOT NULL placed ahead of any DEFAULT, Nullable(T) unwraps to nullable T, LowCardinality(T) unwraps to T, and ARRAY columns lose the constraint BigQuery rejects, (4) maps scalar types (every integer width to INT64 except UInt64 to NUMERIC(20, 0) and the 128/256-bit widths to BIGNUMERIC, Float to FLOAT64, Decimal to NUMERIC, String/FixedString/UUID/IP to STRING, Date to DATE, DateTime/DateTime64 to DATETIME or TIMESTAMP when zone-aware), and (5) preserves Array(T) as a typed ARRAY<T> while collapsing Map/Tuple/Nested to JSON and Enum to STRING, flagging each. Not a semantic transpiler. Assumes one table's SHOW CREATE TABLE output.

## Run

```
mog -m clickhouse-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE default.ch2bq_orders
(
    `order_id` Int64,
    `customer_id` Int32,
    `store_id` Int16,
    `status_code` UInt8,
    `quantity` UInt64,
    `order_ref` String,
    `region_code` FixedString(4),
    `currency` LowCardinality(String),
    `total_amount` Decimal(18, 2),
    `discount_rate` Float32,
    `weight_kg` Float64,
    `created_at` DateTime64(3),
    `updated_at` DateTime,
    `order_date` Date,
    `order_uuid` UUID,
    `notes` Nullable(String),
```

_(... 10 more line(s))_

Output:

```
CREATE OR REPLACE TABLE default.ch2bq_orders
(
    `order_id` INT64 NOT NULL,
    `customer_id` INT64 NOT NULL,
    `store_id` INT64 NOT NULL,
    `status_code` INT64 NOT NULL,
    `quantity` NUMERIC(20, 0) NOT NULL,
    `order_ref` STRING NOT NULL,
    `region_code` STRING NOT NULL,
    `currency` STRING NOT NULL,
    `total_amount` NUMERIC(18, 2) NOT NULL,
    `discount_rate` FLOAT64 NOT NULL,
    `weight_kg` FLOAT64 NOT NULL,
    `created_at` DATETIME NOT NULL,
    `updated_at` DATETIME NOT NULL,
    `order_date` DATE NOT NULL,
    `order_uuid` STRING NOT NULL,
    `notes` STRING,
```

_(... 9 more line(s))_

## Pipeline

- `eol_lf`: Normalize to LF so every anchored regex below is reliable; SQL scripts stay LF.
- `replace`: BigQuery export idiom: CREATE TABLE -> CREATE OR REPLACE TABLE (idempotent reload).
- `replace_regex`: BigQuery does support partitioning, but only on a DATE/TIMESTAMP/DATETIME column or an integer range, and the ClickHouse expression syntax does not carry over. Flag it with the BigQuery shape to write by hand.
- `replace_regex`: ClickHouse PRIMARY KEY is a sparse sort index, not a uniqueness constraint, and BigQuery does not enforce one either. Flag it.
- `replace_regex`: ORDER BY is the ClickHouse sort key. BigQuery CLUSTER BY is the closest analog (up to four columns), so suggest it.
- `replace_regex`: SAMPLE BY (sampling expression) has no BigQuery equivalent. Flag it.
- `replace_regex`: Table-level TTL (data expiry) maps to a BigQuery partition-expiration option rather than a DDL clause. Flag it.
- `replace_regex`: SETTINGS is pure engine tuning (index_granularity, etc.). Flag-and-drop as internals.
- `replace_regex`: Drop the ENGINE = <engine>(...) line entirely (MergeTree family / Log / etc. are storage internals BigQuery has no counterpart for).
- `replace_regex`: Drop per-column CODEC(...) compression hints (ClickHouse storage internals). Safe no-op when absent.
- `replace_regex`: Model ClickHouse nullability: append NOT NULL to every column-definition line while the backtick markers still identify them. Nullable and ARRAY columns get it removed below.
- `replace_regex`: Undo NOT NULL on Nullable(...) columns: Nullable means the column is nullable, which is the BigQuery default.
- `replace_regex`: BigQuery column order is `type [NOT NULL] [DEFAULT ...]`, so move NOT NULL ahead of a DEFAULT clause the previous step appended past.
- `replace_regex`: Unwrap Nullable(T) -> T (handles one nested-parens level, e.g. Nullable(Decimal(18, 2))). BigQuery columns are nullable unless marked NOT NULL.
- `flag_matching`: Flag Enum columns before they are mapped away: BigQuery has no enum type, so the allowed-value constraint is lost.
- `replace_regex`: Unwrap LowCardinality(T) -> T (a ClickHouse storage optimization with no BigQuery analog).
- `replace_regex`: Map(K, V) -> BigQuery JSON. BigQuery has no map type; the faithful alternative is ARRAY<STRUCT<k, v>>, which needs a hand-written shape.
- `replace_regex`: Tuple(...) -> BigQuery JSON (positional structure flattened; a STRUCT<...> equivalent needs named fields).
- `replace_regex`: Nested(...) -> BigQuery JSON (nested repeated structure flattened).
- `replace_regex`: Enum8(...) / Enum16(...) -> STRING (already flagged above).
- `replace_regex`: FixedString(n) -> STRING. BigQuery STRING takes no length parameter, so the fixed width is not enforced.
- `replace_regex`: Array(T) -> ARRAY<T>. BigQuery arrays are typed, so the element type is preserved and mapped by the scalar step below.
- `replace_regex`: DateTime64(n, 'tz') -> TIMESTAMP (absolute instant). Precision is dropped; BigQuery TIMESTAMP is microsecond. Run before the bare DateTime64.
- `replace_regex`: DateTime64(n) -> DATETIME (wall-clock, no zone). Precision dropped.
- `replace_regex`: DateTime('tz') -> TIMESTAMP. Run before the bare DateTime.
- `replace`: bare DateTime -> DATETIME (whole word, so it never bites a DateTime64 already mapped or an identifier).
- `replace`: Date -> DATE (whole word). Runs after every DateTime* map so it never truncates one.
- `replace_regex`: Decimal(p, s) -> NUMERIC(p, s) (BigQuery's fixed-point type, good to 38 digits; ClickHouse keeps the same precision/scale).
- `replace_map`: Map ClickHouse scalar types to BigQuery. BigQuery has a single 64-bit integer type, so every signed width and every unsigned width that fits collapses to INT64; UInt64 overflows it and becomes NUMERIC(20, 0), and the 128/256-bit widths become BIGNUMERIC. Float32/64 both become FLOAT64. String/UUID/IP types become STRING (BigQuery has no length-parameterized string and no UUID or IP type).
- `replace_regex`: BigQuery rejects NOT NULL on an ARRAY column (an array is never NULL, only empty), so drop the constraint the nullability step added.
- `flag_matching`: Flag JSON columns: a ClickHouse Map/Tuple/Nested was flattened into semi-structured JSON.
- `trim_whitespace_right`: Remove any trailing whitespace left on a line.
- `squeeze_blank_lines`: Collapse runs of blank lines to a single blank line.

## Tags

`sql` `clickhouse` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
