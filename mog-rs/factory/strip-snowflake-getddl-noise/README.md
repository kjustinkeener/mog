# Strip Snowflake GET_DDL noise

Remove Snowflake-only GET_DDL renderings, keep the CREATE TABLE

Strip the Snowflake-specific rendering that SELECT GET_DDL('TABLE', ...) adds, so a downstream dialect converter sees a clean CREATE TABLE. Drops the trailing semicolon, the inline column COMMENT 'x' and the glued table )COMMENT='x', and turns cluster by (...) into a TODO. Leaves the create-or-replace verb, autoincrement clause, and the NUMBER/VARIANT/TIMESTAMP_* types for the target-specific converter to map. Target-agnostic; run it first. Assumes LF.

## Run

```
mog -m strip-snowflake-getddl-noise <file>
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
create or replace TABLE CUSTOMERS (
	ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	NAME VARCHAR(200),
	EMAIL VARCHAR(255),
	BALANCE NUMBER(18,2),
	ATTRS VARIANT,
	CREATED_AT TIMESTAMP_NTZ(9),
	UPDATED_AT TIMESTAMP_TZ(9),
	REGION VARCHAR(50),
	constraint PK_CUSTOMERS primary key (ID)
)
-- TODO(mog): dropped Snowflake CLUSTER BY (REGION) (set target clustering/partitioning manually)
```

## Pipeline

- `eol_lf`: Normalize to LF.
- `replace`: Drop statement terminators.
- `replace_regex`: cluster by -> TODO (proprietary micro-partition clustering).
- `replace_regex`: Drop inline column COMMENT 'x'.
- `replace_regex`: Drop the glued table )COMMENT='x'.

## Tags

`sql` `snowflake` `mssql` `strip` `migration` `fragment`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
