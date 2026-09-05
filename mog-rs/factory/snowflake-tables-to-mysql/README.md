# Snowflake GET_DDL to MySQL

Convert a Snowflake GET_DDL table export to MySQL DDL

Convert Snowflake GET_DDL('TABLE', ...) output to MySQL DDL. Strips the Snowflake-only renderings (cluster by, comments, terminator), rewrites autoincrement and the create verb, and maps types (NUMBER(38,0)->BIGINT, NUMBER(p,s)->DECIMAL(p,s), VARIANT->JSON, TIMESTAMP_NTZ->DATETIME, TIMESTAMP_TZ/LTZ->TIMESTAMP). Not a semantic transpiler; leaves TODO(mog) markers where a construct needs manual review.

## Run

```
mog -m snowflake-tables-to-mysql <file>
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
	ID BIGINT NOT NULL AUTO_INCREMENT,
	NAME VARCHAR(200),
	EMAIL VARCHAR(255),
	BALANCE DECIMAL(18,2),
	ATTRS JSON,
	CREATED_AT DATETIME,
	UPDATED_AT TIMESTAMP,
	REGION VARCHAR(50),
	constraint PK_CUSTOMERS primary key (ID)
)
-- TODO(mog): dropped Snowflake CLUSTER BY (REGION) (set target clustering/partitioning manually)
```

## Pipeline

- `run_mog`
- `replace_regex`: Snowflake autoincrement -> MySQL identity.
- `replace_regex`: NUMBER(38,0) -> BIGINT.
- `replace_regex`: NUMBER(p,s) -> DECIMAL(p,s).
- `replace_regex`: Bare NUMBER -> BIGINT.
- `replace_regex`: VARIANT -> JSON.
- `replace_regex`: TIMESTAMP_NTZ(n) -> DATETIME.
- `replace_regex`: TIMESTAMP_TZ / TIMESTAMP_LTZ -> TIMESTAMP.
- `replace`: Normalize the create verb for MySQL.

## Tags

`sql` `snowflake` `mysql` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
