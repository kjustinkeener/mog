# Snowflake tables to TypeScript interfaces

Generate TypeScript interfaces from a Snowflake GET_DDL export

Generate TypeScript 'export interface' declarations from a Snowflake table export (SELECT GET_DDL('TABLE', ...) or a Snowsight DDL copy), one interface per table, 2-space indented. Strips the Snowflake-only rendering (statement terminators, inline COMMENT 'x', the glued )COMMENT='x', cluster by, autoincrement, DEFAULT and table-level constraint lines), then rewrites each 'create or replace TABLE' into an interface. Snowflake identifiers are UPPER_SNAKE_CASE: the interface name is PascalCased and members are camelCased (alias your driver's column mapping to match). Snowflake types map to TypeScript types (NUMBER/DECIMAL/INT/FLOAT/DOUBLE/REAL -> number, BOOLEAN -> boolean, VARCHAR/STRING/TEXT/CHAR -> string, DATE/TIME/TIMESTAMP_NTZ/TIMESTAMP_TZ/TIMESTAMP_LTZ -> string as ISO-8601 text, BINARY/VARBINARY -> Uint8Array, VARIANT/OBJECT -> unknown, ARRAY -> unknown[]). Nullable columns get a '<type> | null' union rather than an optional '?' member. Lossy mappings are flagged inline with a // TODO(mog): comment: semi-structured VARIANT/OBJECT/ARRAY, NUMBER(p,0) wider than Number.MAX_SAFE_INTEGER, and NUMBER(p,s) fixed-point columns that a binary float cannot hold exactly. Assumes LF.

## Run

```
mog -m snowflake-tables-to-typescript <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
create or replace TABLE WEB_SESSION (
	WEB_SESSION_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	FIRST_NAME VARCHAR(64) NOT NULL,
	LAST_NAME VARCHAR(64) NOT NULL,
	UPDATED_AT TIMESTAMP_NTZ(9) NOT NULL,
	primary key (WEB_SESSION_ID)
);

create or replace TABLE CUSTOMER_PROFILE (
	CUSTOMER_PROFILE_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
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
export interface WebSession {
  webSessionId: number; // TODO(mog): Snowflake column can exceed JavaScript's safe range of 2^53; carry wide values losslessly.
  firstName: string;
  lastName: string;
  updatedAt: string;
}

export interface CustomerProfile {
  customerProfileId: number; // TODO(mog): Snowflake column can exceed JavaScript's safe range of 2^53; carry wide values losslessly.
  accountCode: string;
  email: string | null;
  isActive: boolean;
  signedUpOn: string;
  notes: string | null;
  balance: number; // TODO(mog): Snowflake fixed-point column; JavaScript cannot hold every scale exactly -- use an exact-arithmetic library.
  lastSeenAt: string | null;
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
- `flag_matching`: Flag semi-structured VARIANT / OBJECT columns: mapped to unknown.
- `flag_matching`: Flag ARRAY columns: Snowflake arrays are untyped.
- `flag_matching`: Flag wide integers (NUMBER(p,0) with p >= 16): beyond Number.MAX_SAFE_INTEGER.
- `flag_matching`: Flag fixed-point columns (NUMBER(p,s) with s > 0): number is a binary float.
- `replace_regex`: TIMESTAMP_NTZ / DATETIME (wall-clock, no zone) -> string.
- `replace_regex`: TIMESTAMP_TZ / TIMESTAMP_LTZ (zone-aware) -> string.
- `replace_regex`: Bare TIMESTAMP (the session default, normally NTZ) -> string.
- `replace_regex`: TIME -> string.
- `replace_regex`: DATE -> string (ISO-8601 text).
- `replace_regex`: NUMBER(p,s) / DECIMAL(p,s) with s > 0 -> number (via sentinel).
- `replace_regex`: NUMBER(p,0) with p <= 9 -> number (via sentinel).
- `replace_regex`: NUMBER(p,0) with p between 10 and 18 -> number (via sentinel).
- `replace_regex`: NUMBER(p,0) with p >= 19, including Snowflake's NUMBER(38,0) default -> number (via sentinel).
- `replace_regex`: NUMBER(p) with no scale -> number (via sentinel).
- `replace_regex`: Bare NUMBER / DECIMAL / NUMERIC (defaults to NUMBER(38,0)) -> number (via sentinel).
- `replace_regex`: BIGINT / INT / INTEGER (all aliases of NUMBER(38,0) in Snowflake) -> number (via sentinel).
- `replace_regex`: SMALLINT / TINYINT / BYTEINT (also NUMBER(38,0) in Snowflake) -> number (via sentinel).
- `replace_regex`: FLOAT / FLOAT4 / FLOAT8 / DOUBLE PRECISION / DOUBLE / REAL -> number.
- `replace_regex`: BOOLEAN -> boolean.
- `replace_regex`: VARCHAR(n) / CHAR(n) / CHARACTER(n) -> string.
- `replace_regex`: Bare VARCHAR / STRING / TEXT / CHAR -> string.
- `replace_regex`: VARIANT -> unknown (flagged above).
- `replace_regex`: OBJECT -> unknown (flagged above).
- `replace_regex`: ARRAY -> unknown[] (flagged above).
- `replace_regex`: GEOGRAPHY / GEOMETRY -> string (carry the GeoJSON/WKT text through).
- `replace_regex`: BINARY(n) / VARBINARY -> Uint8Array (last, so it cannot be re-matched by an earlier rule).
- `replace_regex`: Resolve the fixed-point sentinel to number.
- `replace_regex`: Resolve the small-integer sentinel to number.
- `replace_regex`: Resolve the mid-integer sentinel to number.
- `replace_regex`: Resolve the wide-integer sentinel to number.
- `replace_regex`: Rewrite nullable columns into a TypeScript member with a '| null' union (2-space indent).
- `replace_regex`: Rewrite non-nullable columns into a TypeScript member (2-space indent).
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each interface name (the Snowflake table identifier).
- `to_camel`: camelCase each member name; Snowflake returns UPPER_SNAKE_CASE identifiers.
- `replace_regex`: Collapse multiple blank lines to a single blank between interfaces.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `snowflake` `typescript` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
