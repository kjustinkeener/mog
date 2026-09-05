# Snowflake tables to Go structs

Generate Go structs (with db tags) from a Snowflake GET_DDL export

Generate idiomatic Go type declarations from a Snowflake table export (SELECT GET_DDL('TABLE', ...) or a Snowsight DDL copy), one 'type <Table> struct' per table. Strips the Snowflake-only rendering (statement terminators, inline COMMENT 'x', the glued )COMMENT='x', cluster by, autoincrement, DEFAULT and table-level constraint lines), then rewrites each 'create or replace TABLE' into a struct with one exported field per column. Field names are PascalCased for export, with the exact UPPER_SNAKE_CASE Snowflake column name preserved losslessly in a `db:"..."` struct tag (compatible with sqlx / gosnowflake). Snowflake types map to Go types (NUMBER(p,0) p<=9 -> int32, wider -> int64, NUMBER(p,s) s>0 -> string, FLOAT/DOUBLE/REAL -> float64, BOOLEAN -> bool, VARCHAR/STRING/TEXT/CHAR -> string, DATE/TIME/TIMESTAMP_NTZ/TIMESTAMP_TZ/TIMESTAMP_LTZ -> time.Time, BINARY/VARBINARY -> []byte, VARIANT/OBJECT/ARRAY -> json.RawMessage). Nullable columns (no NOT NULL) become pointer types (*T); nilable types (slices, json.RawMessage) stay as-is. Lossy mappings are flagged inline with a // TODO(mog): comment: semi-structured columns, NUMBER(p,s) fixed-point (Go has no native decimal), and NUMBER(p,0) wide enough to overflow int64. Emits the struct declarations only: add your own 'package' clause and imports (time, encoding/json), then run gofmt to align fields. Assumes LF.

## Run

```
mog -m snowflake-tables-to-go <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
create or replace TABLE SHIPMENT (
	SHIPMENT_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	FIRST_NAME VARCHAR(64) NOT NULL,
	LAST_NAME VARCHAR(64) NOT NULL,
	UPDATED_AT TIMESTAMP_NTZ(9) NOT NULL,
	primary key (SHIPMENT_ID)
);

create or replace TABLE SUPPLIER_ACCOUNT (
	SUPPLIER_ACCOUNT_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	ACCOUNT_CODE VARCHAR(32) NOT NULL,
	EMAIL VARCHAR(255) COMMENT 'primary contact email',
	IS_ACTIVE BOOLEAN NOT NULL DEFAULT TRUE,
	SIGNED_UP_ON DATE NOT NULL,
	NOTES STRING,
	BALANCE NUMBER(18,2) NOT NULL,
	LAST_SEEN_AT TIMESTAMP_LTZ(9)
)COMMENT='account master table'
```

_(... 20 more line(s))_

Output:

```
type Shipment struct {
	ShipmentId int64 `db:"SHIPMENT_ID"` // TODO(mog): Snowflake column can exceed the signed 64-bit range; use math/big if values can pass 9.2e18.
	FirstName string `db:"FIRST_NAME"`
	LastName string `db:"LAST_NAME"`
	UpdatedAt time.Time `db:"UPDATED_AT"`
}

type SupplierAccount struct {
	SupplierAccountId int64 `db:"SUPPLIER_ACCOUNT_ID"` // TODO(mog): Snowflake column can exceed the signed 64-bit range; use math/big if values can pass 9.2e18.
	AccountCode string `db:"ACCOUNT_CODE"`
	Email *string `db:"EMAIL"`
	IsActive bool `db:"IS_ACTIVE"`
	SignedUpOn time.Time `db:"SIGNED_UP_ON"`
	Notes *string `db:"NOTES"`
	Balance string `db:"BALANCE"` // TODO(mog): Snowflake fixed-point column; Go has no native exact-scale kind -- parse with an exact-arithmetic library.
	LastSeenAt *time.Time `db:"LAST_SEEN_AT"`
}

```

_(... 18 more line(s))_

## Pipeline

- `run_mog`: Remove the Snowflake GET_DDL rendering: statement terminators, inline column COMMENT 'x', the glued table )COMMENT='x', and the cluster by clause.
- `replace_regex`: Drop the SQL comment lines the strip fragment leaves behind (the CLUSTER BY note).
- `replace_regex`: Drop table-level constraint lines (primary key, foreign key, unique, constraint).
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Convert the tab indent GET_DDL emits to four spaces, so column lines have a fixed shape.
- `replace_regex`: Rewrite each 'create or replace TABLE <tbl> (' header.
- `replace_regex`: Rewrite each table-closing ')' into a closing brace.
- `replace_regex`: Drop the Snowflake autoincrement / identity clause.
- `replace_regex`: Drop COLLATE clauses.
- `replace_regex`: Drop DEFAULT clauses (sequences, CURRENT_TIMESTAMP(), literals).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag semi-structured VARIANT / OBJECT columns: mapped to json.RawMessage.
- `flag_matching`: Flag ARRAY columns: Snowflake arrays are untyped.
- `flag_matching`: Flag fixed-point columns (NUMBER(p,s) with s > 0): Go has no native decimal.
- `flag_matching`: Flag wide integers (NUMBER(p,0) with p >= 19): they can overflow int64.
- `replace_regex`: TIMESTAMP_NTZ / DATETIME (wall-clock, no zone) -> @@GODT@@.
- `replace_regex`: TIMESTAMP_TZ / TIMESTAMP_LTZ (zone-aware) -> @@GODT@@.
- `replace_regex`: Bare TIMESTAMP (the session default, normally NTZ) -> @@GODT@@.
- `replace_regex`: TIME -> @@GODT@@.
- `replace_regex`: DATE -> @@GODT@@.
- `replace_regex`: NUMBER(p,s) / DECIMAL(p,s) with s > 0 -> string (via sentinel).
- `replace_regex`: NUMBER(p,0) with p <= 9 -> int32 (via sentinel).
- `replace_regex`: NUMBER(p,0) with p between 10 and 18 -> int64 (via sentinel).
- `replace_regex`: NUMBER(p,0) with p >= 19, including Snowflake's NUMBER(38,0) default -> int64 (via sentinel).
- `replace_regex`: NUMBER(p) with no scale -> int64 (via sentinel).
- `replace_regex`: Bare NUMBER / DECIMAL / NUMERIC (defaults to NUMBER(38,0)) -> int64 (via sentinel).
- `replace_regex`: BIGINT / INT / INTEGER (all aliases of NUMBER(38,0) in Snowflake) -> int64 (via sentinel).
- `replace_regex`: SMALLINT / TINYINT / BYTEINT (also NUMBER(38,0) in Snowflake) -> int64 (via sentinel).
- `replace_regex`: FLOAT / FLOAT4 / FLOAT8 / DOUBLE PRECISION / DOUBLE / REAL -> float64.
- `replace_regex`: BOOLEAN -> bool.
- `replace_regex`: VARCHAR(n) / CHAR(n) / CHARACTER(n) -> string.
- `replace_regex`: Bare VARCHAR / STRING / TEXT / CHAR -> string.
- `replace_regex`: VARIANT -> json.RawMessage (flagged above).
- `replace_regex`: OBJECT -> json.RawMessage (flagged above).
- `replace_regex`: ARRAY -> json.RawMessage (flagged above).
- `replace_regex`: GEOGRAPHY / GEOMETRY -> string (carry the GeoJSON/WKT text through).
- `replace_regex`: BINARY(n) / VARBINARY -> []byte (last, so it cannot be re-matched by an earlier rule).
- `replace_regex`: Resolve the fixed-point sentinel to string.
- `replace_regex`: Resolve the small-integer sentinel to int32.
- `replace_regex`: Resolve the mid-integer sentinel to int64.
- `replace_regex`: Resolve the wide-integer sentinel to int64.
- `replace_regex`: Resolve the time sentinel to time.Time (kept separate so the 'TIME' keyword rule cannot re-match it).
- `replace_regex`: Rewrite nullable columns into a pointer-typed Go field carrying a db tag placeholder.
- `replace_regex`: Rewrite non-nullable columns into a Go field carrying a db tag placeholder.
- `replace_regex`: Nilable types (slices, json.RawMessage) don't need a pointer for nullability -- drop it.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each struct name (the Snowflake table identifier).
- `to_pascal`: PascalCase each field name so it is exported; the db tag keeps the exact Snowflake column name.
- `replace_regex`: Resolve the db tag placeholder into a real Go struct tag.
- `replace_regex`: Convert the 4-space column indent to a tab (Go convention).
- `replace_regex`: Collapse multiple blank lines to a single blank between structs.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `snowflake` `go` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
