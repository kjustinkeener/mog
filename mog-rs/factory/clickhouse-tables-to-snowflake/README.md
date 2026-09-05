# ClickHouse SHOW CREATE TABLE to Snowflake

Convert a ClickHouse table export to Snowflake DDL

Convert the output of ClickHouse 'SHOW CREATE TABLE' to runnable Snowflake DDL. (1) Rewrites CREATE TABLE to CREATE OR REPLACE TABLE and drops the ClickHouse database qualifier, (2) strips the engine/physical clauses (ENGINE, SETTINGS, CODEC) and flags the ones carrying intent (PARTITION BY, PRIMARY KEY, ORDER BY, SAMPLE BY, TTL) as TODO(mog), leaving an ORDER BY as a CLUSTER BY hint, (3) models nullability: bare columns become NOT NULL, Nullable(T) unwraps to nullable T, LowCardinality(T) unwraps to T, (4) maps scalar types (Int/UInt widths, Float, Decimal->NUMBER, String->VARCHAR, FixedString(n)->VARCHAR(n), UUID->VARCHAR(36), Date->DATE, DateTime/DateTime64->TIMESTAMP_NTZ or TIMESTAMP_TZ with precision dropped), and (5) collapses composite types (Array->ARRAY, Map->OBJECT, Tuple/Nested->VARIANT, Enum->VARCHAR) and flags each. Not a semantic transpiler. Assumes one table's SHOW CREATE TABLE output.

## Run

```
mog -m clickhouse-tables-to-snowflake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE default.ch2sf_orders
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
CREATE OR REPLACE TABLE ch2sf_orders
(
    order_id BIGINT NOT NULL,
    customer_id INTEGER NOT NULL,
    store_id SMALLINT NOT NULL,
    status_code SMALLINT NOT NULL,
    quantity NUMBER(20,0) NOT NULL,
    order_ref VARCHAR NOT NULL,
    region_code VARCHAR(4) NOT NULL,
    currency VARCHAR NOT NULL,
    total_amount NUMBER(18, 2) NOT NULL,
    discount_rate FLOAT NOT NULL,
    weight_kg FLOAT NOT NULL,
    created_at TIMESTAMP_NTZ NOT NULL,
    updated_at TIMESTAMP_NTZ NOT NULL,
    order_date DATE NOT NULL,
    order_uuid VARCHAR(36) NOT NULL,
    notes VARCHAR,
```

_(... 9 more line(s))_

## Pipeline

- `eol_lf`: Normalize to LF so every anchored regex below is reliable; SQL scripts stay LF.
- `replace`: Snowflake export idiom: CREATE TABLE -> CREATE OR REPLACE TABLE (idempotent reload).
- `replace_regex`: Drop the ClickHouse database qualifier (e.g. default.tbl -> tbl); the table lands in the current Snowflake schema. A migration sets the target schema separately.
- `replace_regex`: PARTITION BY carries physical layout intent but has no Snowflake equivalent (Snowflake manages micro-partitions). Flag it.
- `replace_regex`: ClickHouse PRIMARY KEY is a sparse sort index, not a uniqueness constraint. Flag it rather than emit a Snowflake PRIMARY KEY (which would change semantics).
- `replace_regex`: ORDER BY is the ClickHouse sort/primary key. Flag it and suggest a Snowflake CLUSTER BY on the same columns as a starting point.
- `replace_regex`: SAMPLE BY (sampling expression) has no Snowflake equivalent. Flag it.
- `replace_regex`: Table-level TTL (data expiry) is a ClickHouse retention feature. Flag it; model retention with a Snowflake task or lifecycle policy.
- `replace_regex`: SETTINGS is pure engine tuning (index_granularity, etc.). Flag-and-drop as internals.
- `replace_regex`: Drop the ENGINE = <engine>(...) line entirely (MergeTree family / Log / etc. are storage internals with no Snowflake counterpart).
- `replace_regex`: Drop per-column CODEC(...) compression hints (ClickHouse storage internals). Safe no-op when absent.
- `replace_regex`: Model ClickHouse nullability: append NOT NULL to every column-definition line while the backtick markers still identify them. Nullable columns get it removed in the next step. The comma (if any) is preserved after NOT NULL.
- `replace_regex`: Undo NOT NULL on Nullable(...) columns: Nullable means the column is nullable, which is Snowflake's default.
- `replace_regex`: Unwrap Nullable(T) -> T (handles one nested-parens level, e.g. Nullable(Decimal(18, 2))). Snowflake columns are nullable unless marked NOT NULL.
- `replace`: Unquote ClickHouse backtick identifiers (they are simple names; Snowflake folds unquoted names to upper-case, which is the idiomatic result).
- `flag_matching`: Flag Enum columns before they are mapped away: Snowflake has no enum type, so the allowed-value constraint is lost.
- `replace_regex`: Unwrap LowCardinality(T) -> T (a ClickHouse storage optimization with no Snowflake analog).
- `replace_regex`: FixedString(n) -> VARCHAR(n) (Snowflake has no fixed-width string; VARCHAR(n) caps the length).
- `replace_regex`: Array(T) -> Snowflake ARRAY (untyped semi-structured; element type dropped).
- `replace_regex`: Map(K, V) -> Snowflake OBJECT (untyped semi-structured; key/value types dropped).
- `replace_regex`: Tuple(...) -> Snowflake VARIANT (positional structure flattened into semi-structured).
- `replace_regex`: Nested(...) -> Snowflake VARIANT (nested repeated structure flattened).
- `replace_regex`: Enum8(...) / Enum16(...) -> VARCHAR (already flagged above).
- `replace_regex`: DateTime64(n, 'tz') -> TIMESTAMP_TZ (timezone-aware). Precision is dropped (Snowflake caps at 9; fakesnow drops it on *TZ anyway). Run before the bare DateTime64.
- `replace_regex`: DateTime64(n) -> TIMESTAMP_NTZ (wall-clock, no zone). Precision dropped.
- `replace_regex`: DateTime('tz') -> TIMESTAMP_TZ. Run before the bare DateTime.
- `replace`: bare DateTime -> TIMESTAMP_NTZ (whole word, so it never bites a DateTime64 already mapped or an identifier).
- `replace`: Date -> DATE (whole word). Runs after every DateTime* map so it never truncates one.
- `replace_regex`: Decimal(p, s) -> NUMBER(p, s) (Snowflake's fixed-point type; ClickHouse keeps the same precision/scale).
- `replace_map`: Map ClickHouse scalar types to Snowflake. Widths: Int64->BIGINT, Int32->INTEGER, Int8/16->SMALLINT; unsigned promotes a width where the range needs it (UInt64->NUMBER(20,0)); wide 128/256 ints -> NUMBER(38,0). Float32/64 both -> FLOAT (Snowflake FLOAT is 64-bit). String->VARCHAR, UUID->VARCHAR(36) (Snowflake has no UUID type), Bool->BOOLEAN, IPv4/IPv6->VARCHAR.
- `flag_matching`: Flag ARRAY columns: Snowflake ARRAY is untyped, so the ClickHouse element type was dropped.
- `flag_matching`: Flag OBJECT columns: Snowflake OBJECT is untyped, so the ClickHouse key/value types were dropped.
- `flag_matching`: Flag VARIANT columns: a ClickHouse Tuple/Nested was flattened into semi-structured VARIANT.
- `trim_whitespace_right`: Remove any trailing whitespace left on a line.
- `squeeze_blank_lines`: Collapse runs of blank lines to a single blank line.

## Tags

`sql` `clickhouse` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
