# Snowflake tables to C# classes

Generate C# POCO classes from a Snowflake GET_DDL export

Generate idiomatic C# POCO classes from a Snowflake table export (SELECT GET_DDL('TABLE', ...) or a Snowsight DDL copy), one class per table with an auto-property per column. Strips the Snowflake-only rendering (statement terminators, inline COMMENT 'x', the glued )COMMENT='x', cluster by, autoincrement, DEFAULT and table-level constraint lines), then rewrites each 'create or replace TABLE' into a 'public class'. Snowflake types map to C# types (NUMBER(p,0) p<=9 -> int, wider -> long, NUMBER(p,s) s>0 -> decimal, FLOAT/DOUBLE/REAL -> double, BOOLEAN -> bool, VARCHAR/STRING/TEXT/CHAR -> string, DATE -> DateOnly, TIME -> TimeOnly, TIMESTAMP_NTZ -> DateTime, TIMESTAMP_TZ/TIMESTAMP_LTZ -> DateTimeOffset, BINARY/VARBINARY -> byte[], VARIANT/OBJECT/ARRAY -> string holding raw JSON). Nullable columns (no NOT NULL) get a nullable type ('?'). Table and column identifiers are PascalCased from Snowflake's UPPER_SNAKE_CASE. Lossy mappings are flagged with a // TODO(mog): comment: semi-structured columns and NUMBER(p,0) wide enough to overflow long. Assumes LF.

## Run

```
mog -m snowflake-tables-to-csharp <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
create or replace TABLE INVOICE (
	INVOICE_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	FIRST_NAME VARCHAR(64) NOT NULL,
	LAST_NAME VARCHAR(64) NOT NULL,
	UPDATED_AT TIMESTAMP_NTZ(9) NOT NULL,
	primary key (INVOICE_ID)
);

create or replace TABLE POLICY_HOLDER (
	POLICY_HOLDER_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
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
using System;

public class Invoice
{
    public long InvoiceId { get; set; } // TODO(mog): Snowflake column can exceed the signed 64-bit range; use System.Numerics.BigInteger if values can pass 9.2e18.
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime UpdatedAt { get; set; }
}

public class PolicyHolder
{
    public long PolicyHolderId { get; set; } // TODO(mog): Snowflake column can exceed the signed 64-bit range; use System.Numerics.BigInteger if values can pass 9.2e18.
    public string AccountCode { get; set; }
    public string? Email { get; set; }
    public bool IsActive { get; set; }
    public DateOnly SignedUpOn { get; set; }
    public string? Notes { get; set; }
```

_(... 23 more line(s))_

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
- `flag_matching`: Flag semi-structured VARIANT / OBJECT columns: mapped to string.
- `flag_matching`: Flag ARRAY columns: Snowflake arrays are untyped.
- `flag_matching`: Flag wide integers (NUMBER(p,0) with p >= 19): they can overflow long.
- `replace_regex`: TIMESTAMP_NTZ / DATETIME (wall-clock, no zone) -> DateTime.
- `replace_regex`: TIMESTAMP_TZ / TIMESTAMP_LTZ (zone-aware) -> DateTimeOffset.
- `replace_regex`: Bare TIMESTAMP (the session default, normally NTZ) -> DateTime.
- `replace_regex`: TIME -> TimeOnly.
- `replace_regex`: DATE -> DateOnly.
- `replace_regex`: NUMBER(p,s) / DECIMAL(p,s) with s > 0 -> decimal (via sentinel).
- `replace_regex`: NUMBER(p,0) with p <= 9 -> int (via sentinel).
- `replace_regex`: NUMBER(p,0) with p between 10 and 18 -> long (via sentinel).
- `replace_regex`: NUMBER(p,0) with p >= 19, including Snowflake's NUMBER(38,0) default -> long (via sentinel).
- `replace_regex`: NUMBER(p) with no scale -> long (via sentinel).
- `replace_regex`: Bare NUMBER / DECIMAL / NUMERIC (defaults to NUMBER(38,0)) -> long (via sentinel).
- `replace_regex`: BIGINT / INT / INTEGER (all aliases of NUMBER(38,0) in Snowflake) -> long (via sentinel).
- `replace_regex`: SMALLINT / TINYINT / BYTEINT (also NUMBER(38,0) in Snowflake) -> long (via sentinel).
- `replace_regex`: FLOAT / FLOAT4 / FLOAT8 / DOUBLE PRECISION / DOUBLE / REAL -> double.
- `replace_regex`: BOOLEAN -> bool.
- `replace_regex`: VARCHAR(n) / CHAR(n) / CHARACTER(n) -> string.
- `replace_regex`: Bare VARCHAR / STRING / TEXT / CHAR -> string.
- `replace_regex`: VARIANT -> string (flagged above).
- `replace_regex`: OBJECT -> string (flagged above).
- `replace_regex`: ARRAY -> string (flagged above).
- `replace_regex`: GEOGRAPHY / GEOMETRY -> string (carry the GeoJSON/WKT text through).
- `replace_regex`: BINARY(n) / VARBINARY -> byte[] (last, so it cannot be re-matched by an earlier rule).
- `replace_regex`: Resolve the fixed-point sentinel to decimal.
- `replace_regex`: Resolve the small-integer sentinel to int.
- `replace_regex`: Resolve the mid-integer sentinel to long.
- `replace_regex`: Resolve the wide-integer sentinel to long.
- `replace_regex`: Rewrite nullable columns into a nullable C# auto-property.
- `replace_regex`: Rewrite non-nullable columns into a C# auto-property.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the Snowflake table identifier).
- `to_pascal`: PascalCase each property name; Snowflake returns UPPER_SNAKE_CASE identifiers.
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.
- `replace_regex`: Prepend the using directives the generated types need (System for DateTime, DateOnly and DateTimeOffset). Last, so the type-map regexes never touch the header.

## Tags

`sql` `snowflake` `csharp` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
