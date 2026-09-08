# Snowflake GET_DDL to Trino

Convert a Snowflake GET_DDL table export to Trino DDL

Convert Snowflake GET_DDL('TABLE', ...) output to Trino DDL. Strips the Snowflake-only renderings (cluster by, comments, terminator), drops autoincrement and PRIMARY KEY (Trino has neither; flagged), rewrites the create verb, and maps types (NUMBER(38,0)->BIGINT, NUMBER(p,s)->DECIMAL(p,s), VARIANT->JSON, TIMESTAMP_NTZ->TIMESTAMP, TIMESTAMP_TZ/LTZ->TIMESTAMP WITH TIME ZONE). Not a semantic transpiler; leaves TODO(mog) markers where a construct needs manual review.

## Run

```
mog -m snowflake-tables-to-trino <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
create or replace TABLE CUSTOMERS (
	ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	NAME VARCHAR(200),
	EMAIL VARCHAR(255) COMMENT 'primary contact email',
	BALANCE NUMBER(18,2),
	ATTRS VARIANT,
	CREATED_AT TIMESTAMP_NTZ(9),
	UPDATED_AT TIMESTAMP_TZ(9),
	REGION VARCHAR(50),
	constraint PK_CUSTOMERS primary key (ID)
)COMMENT='customer master table'
cluster by (REGION);
```

Output:

```
CREATE TABLE CUSTOMERS (
	ID BIGINT NOT NULL,
	NAME VARCHAR(200),
	EMAIL VARCHAR(255),
	BALANCE DECIMAL(18,2),
	ATTRS JSON,
	CREATED_AT TIMESTAMP,
	UPDATED_AT TIMESTAMP WITH TIME ZONE,
	REGION VARCHAR(50)
	-- TODO(mog): dropped PRIMARY KEY (Trino has no table constraints)
)
-- TODO(mog): dropped Snowflake CLUSTER BY (REGION) (set target clustering/partitioning manually)
```

## Steps

- `run_mog`
- `replace_regex`: Drop autoincrement (Trino has no identity columns).
- `replace_regex`: Drop the PRIMARY KEY constraint (Trino CREATE TABLE has no constraints; flagged).
- `replace_regex`: NUMBER(38,0) -> BIGINT.
- `replace_regex`: NUMBER(p,s) -> DECIMAL(p,s).
- `replace_regex`: Bare NUMBER -> BIGINT.
- `replace_regex`: VARIANT -> JSON.
- `replace_regex`: TIMESTAMP_NTZ(n) -> TIMESTAMP.
- `replace_regex`: TIMESTAMP_TZ / TIMESTAMP_LTZ -> TIMESTAMP WITH TIME ZONE.
- `replace`: Normalize the create verb for Trino.

## Tags

`sql` `snowflake` `trino` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
