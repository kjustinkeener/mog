# ClickHouse SHOW CREATE TABLE to Postgres

Convert a ClickHouse table export to runnable PostgreSQL DDL

Convert the output of ClickHouse 'SHOW CREATE TABLE' to PostgreSQL (e.g. mirroring an analytics table into a serving/OLTP database, or migrating off ClickHouse). Strips the ClickHouse engine/physical noise (ENGINE, PARTITION BY, ORDER BY, PRIMARY KEY, SAMPLE BY, TTL, SETTINGS, backticks, database qualifier), applies ClickHouse nullability (columns are NOT NULL unless wrapped in Nullable(); LowCardinality() is unwrapped), and maps types to Postgres: String->text, FixedString(n)->varchar(n), Int8/16->smallint, Int32->integer, Int64->bigint, UInt8->smallint, UInt16->integer, UInt32->bigint, UInt64->numeric(20,0), Decimal(p,s)->numeric(p,s), Float32->real, Float64->double precision, DateTime64(n,'tz')->timestamptz, DateTime64(n)/DateTime->timestamp, Date->date, UUID->uuid. Non-portable constructs are flagged with -- TODO(mog): Enum8/16->text (value set lost), Array(T)->T[], Map/Tuple/Nested->jsonb, and dropped ORDER BY/PARTITION BY preserved as comments.

## Run

```
mog -m clickhouse-tables-to-postgres <file>
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
    order_id numeric(20, 0) NOT NULL,
    customer_id bigint NOT NULL,
    store_id integer NOT NULL,
    region_code smallint NOT NULL,
    is_paid smallint DEFAULT 0 NOT NULL,
    status text NOT NULL,
    order_ref varchar(12) NOT NULL,
    notes text,
    coupon text,
    quantity integer NOT NULL,
    item_count bigint NOT NULL,
    big_seq numeric(20, 0) NOT NULL,
    amount numeric(18, 2) NOT NULL,
    discount real NOT NULL,
    tax_rate double precision NOT NULL,
    created_at timestamptz NOT NULL,
```

_(... 12 more line(s))_

## Pipeline

- `run_mog`: Remove ClickHouse engine/physical/identifier noise; ORDER BY and PARTITION BY become TODO comments.
- `replace_regex`: Unwrap LowCardinality(Nullable(T)) -> Nullable(T): LowCardinality is a storage encoding with no Postgres analog; keep the nullability.
- `replace_regex`: Unwrap LowCardinality(T) -> T (dictionary-encoding hint dropped).
- `replace_regex`: ClickHouse columns are NOT NULL by default: stamp NOT NULL on every column line that is not wrapped in Nullable(). The negative lookahead skips Nullable columns; structural lines (CREATE TABLE, the parens, TODO comments) do not start with the 4-space column indent.
- `replace_regex`: Unwrap Nullable(inner(...)) with one level of nested parens (e.g. Nullable(FixedString(12)), Nullable(Decimal(18,2))) -> the inner type; the column is left nullable (Postgres default).
- `replace_regex`: Unwrap the remaining simple Nullable(T) -> T.
- `replace_regex`: FixedString(n) -> varchar(n) (fixed-width text; varchar is the closest runnable Postgres type).
- `replace_regex`: String -> text (ClickHouse String is unbounded).
- `replace_regex`: DateTime64(n,'tz') (instant with time zone) -> timestamptz. Runs before the plain DateTime64/DateTime maps so its tz form is not swallowed.
- `replace_regex`: DateTime64(n) (sub-second, no tz) -> timestamp.
- `replace_regex`: DateTime('tz') -> timestamptz.
- `replace_regex`: DateTime (second precision, no tz) -> timestamp.
- `replace_regex`: Date32/Date -> date.
- `replace_regex`: Decimal(p,s) -> numeric(p,s) (identical fixed-point semantics).
- `replace_regex`: UInt64 -> numeric(20,0): an unsigned 64-bit value overflows Postgres bigint, so use exact numeric.
- `replace_regex`: UInt32 -> bigint (unsigned 32-bit exceeds signed integer range).
- `replace_regex`: UInt16 -> integer (unsigned 16-bit exceeds signed smallint range).
- `replace_regex`: UInt8 -> smallint (safe; use boolean by hand if the column is a 0/1 flag).
- `replace_regex`: Int64 -> bigint.
- `replace_regex`: Int32 -> integer.
- `replace_regex`: Int16 -> smallint.
- `replace_regex`: Int8 -> smallint (Postgres has no 1-byte integer).
- `replace_regex`: Float32 -> real.
- `replace_regex`: Float64 -> double precision.
- `replace_regex`: UUID -> uuid.
- `flag_matching`: Flag Enum8/Enum16 before mapping it to text (the allowed-value constraint is lost; use a CHECK or a Postgres enum type).
- `replace_regex`: Enum8/Enum16(...) -> text.
- `flag_matching`: Flag Array columns before mapping to a Postgres array.
- `replace_regex`: Array(T) -> T[] (T already mapped to a Postgres scalar above).
- `flag_matching`: Flag Map columns before mapping to jsonb.
- `replace_regex`: Map(K, V) -> jsonb.
- `flag_matching`: Flag Tuple columns before mapping to jsonb.
- `replace_regex`: Tuple(...) -> jsonb.
- `flag_matching`: Flag Nested columns before mapping to jsonb.
- `replace_regex`: Nested(...) -> jsonb.

## Tags

`sql` `clickhouse` `postgres` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
