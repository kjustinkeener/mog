# Snowflake GET_DDL to DuckDB

Convert a Snowflake GET_DDL table export to DuckDB DDL

Convert Snowflake GET_DDL('TABLE', ...) output to DuckDB DDL. Drops the trailing semicolon and the Snowflake-only cluster-by (flagged), table-comment, column-comment, and autoincrement renderings, maps types to DuckDB (NUMBER(38,0)->BIGINT, NUMBER(p,s)->DECIMAL(p,s), VARIANT/OBJECT/ARRAY->JSON, TIMESTAMP_NTZ->TIMESTAMP, TIMESTAMP_TZ/LTZ->TIMESTAMPTZ), and normalizes the create-or-replace verb. Expects one table's GET_DDL output. Not a semantic transpiler; leaves TODO(mog) markers where a construct needs manual review.

## Run

```
mog -m snowflake-tables-to-duckdb <file>
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
	REGION VARCHAR(50)
)COMMENT='customer master table'
cluster by (REGION);
```

Output:

```
CREATE OR REPLACE TABLE CUSTOMERS (
	ID BIGINT NOT NULL,
	NAME VARCHAR(200),
	EMAIL VARCHAR(255),
	BALANCE DECIMAL(18,2),
	ATTRS JSON,
	CREATED_AT TIMESTAMP,
	UPDATED_AT TIMESTAMPTZ,
	REGION VARCHAR(50)
)
-- TODO(mog): dropped Snowflake CLUSTER BY (REGION) (DuckDB has no clustering)
```

## Pipeline

- `eol_lf`: Normalize to LF.
- `replace`: Drop statement terminators (single-statement export).
- `replace_regex`: Snowflake cluster by -> TODO (DuckDB has no clustering).
- `replace_regex`: Drop Snowflake autoincrement rendering (set identity via a DuckDB sequence if needed).
- `replace_regex`: Drop inline column COMMENT 'x' (DuckDB uses COMMENT ON).
- `replace_regex`: Drop the glued table )COMMENT='x'.
- `replace_map`: Snowflake semi-structured types -> DuckDB JSON.
- `replace_regex`: NUMBER(38,0) (Snowflake default integer) -> BIGINT.
- `replace_regex`: NUMBER(p,s) -> DECIMAL(p,s).
- `replace_regex`: Bare NUMBER -> BIGINT.
- `replace_regex`: TIMESTAMP_NTZ(n) -> TIMESTAMP (wall-clock).
- `replace_regex`: TIMESTAMP_TZ / TIMESTAMP_LTZ -> TIMESTAMPTZ (instant).
- `replace`: Normalize the mixed-case verb.

## Tags

`sql` `snowflake` `duckdb` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
