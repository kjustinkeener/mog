# SQL Server (SSMS) to BigQuery table DDL

Convert SQL Server table DDL to BigQuery

Convert a SQL Server SSMS 'Generate Scripts' / mssql-scripter table export to runnable BigQuery standard-SQL DDL. It (1) strips the SSMS server/session/DROP scaffolding (banners, SET options, CREATE/ALTER DATABASE, the DROP phase, constraint re-enables, and the sp_addextendedproperty MS_Description calls -- BigQuery column descriptions live in OPTIONS(), so the descriptions are dropped, not converted), (2) drops T-SQL [bracket] quoting and the dbo. schema qualifier, (3) maps T-SQL functions (newid/newsequentialid->GENERATE_UUID, sysdatetime/getdate->CURRENT_DATETIME, *utc*/sysdatetimeoffset->CURRENT_TIMESTAMP) and every T-SQL type to BigQuery (bigint/int/smallint/tinyint->INT64, bit->BOOL, decimal/numeric/money->NUMERIC, float/real->FLOAT64, nvarchar/varchar/nchar/char/text->STRING, uniqueidentifier/xml/hierarchyid/sql_variant->STRING, varbinary/binary/image->BYTES, datetime2->DATETIME, datetimeoffset->TIMESTAMP, datetime/smalldatetime->DATETIME, geography/geometry->GEOGRAPHY), (4) drops IDENTITY(seed,step) (BigQuery has no auto-increment) and flags each such column, and (5) flags every T-SQL construct with no runnable BigQuery form via -- TODO(mog): the PERSISTED computed column, the PRIMARY KEY (BigQuery only takes PRIMARY KEY ... NOT ENFORCED inside CREATE TABLE), separate DEFAULT constraints, CHECK and UNIQUE constraints, and secondary CREATE INDEX (BigQuery has none; use partitioning/clustering). Not a semantic transpiler. Assumes LF text.

## Run

```
mog -m sqlserver-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [lab]
GO
/****** Object:  Table [dbo].[ms2bq_orders]    Script Date: Sat 8 29 2026  12:24:56 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[ms2bq_orders](
	[order_id] [bigint] IDENTITY(1,1) NOT NULL,
	[order_uid] [uniqueidentifier] NOT NULL,
	[customer_code] [nvarchar](50) NOT NULL,
	[description] [nvarchar](max) NULL,
	[status_flag] [bit] NOT NULL,
	[quantity] [int] NOT NULL,
	[unit_price] [money] NOT NULL,
	[discount_pct] [decimal](5, 2) NOT NULL,
	[line_total]  AS ([quantity]*[unit_price]) PERSISTED,
	[notes] [varchar](max) NULL,
```

_(... 47 more line(s))_

Output:

```
-- Dataset: lab

CREATE OR REPLACE TABLE ms2bq_orders(
	order_id INT64 NOT NULL, -- TODO(mog): BigQuery has no auto-increment; assign this key with GENERATE_UUID() or from the application
	order_uid STRING NOT NULL,
	customer_code STRING(50) NOT NULL,
	description STRING NULL,
	status_flag BOOL NOT NULL,
	quantity INT64 NOT NULL,
	unit_price NUMERIC NOT NULL,
	discount_pct NUMERIC(5, 2) NOT NULL,
	-- TODO(mog): computed column line_total = (quantity*unit_price) [PERSISTED] has no BigQuery column form; recreate it as a view or a downstream derived column.
	notes STRING NULL,
	payload BYTES NULL,
	created_at DATETIME NOT NULL,
	updated_at TIMESTAMP NULL,
	legacy_ts DATETIME NULL
);
```

_(... 19 more line(s))_

## Steps

- `eol_lf`: Normalize to LF first so every later newline-sensitive regex is reliable.
- `run_mog`: Remove SSMS server/session/DROP scaffolding (object banners, SET options, CREATE/ALTER DATABASE, the guarded DROP phase, ALTER TABLE CHECK CONSTRAINT re-enables, and the sp_addextendedproperty description calls). Shared fragment; runs on the raw SSMS text while GO batches are intact.
- `replace_regex`: BigQuery has no secondary indexes: replace the whole CREATE [NON]CLUSTERED INDEX ... GO batch with a TODO. Runs before bracket-stripping so it can consume the full multi-line INCLUDE/WITH/ON PRIMARY batch in one match.
- `replace`: Strip T-SQL square-bracket quoting: opening bracket.
- `replace`: Strip T-SQL square-bracket quoting: closing bracket.
- `replace`: Drop the dbo. schema qualifier so objects resolve in the active BigQuery dataset.
- `replace_regex`: USE <db> + its GO -> a documenting comment (BigQuery selects a project/dataset on the job, not with USE <db> in DDL).
- `replace_map`: Map T-SQL builtins to BigQuery. Runs before the type maps so a sys*datetime* function name is never corrupted. None of these keys is a substring of another (the trailing () differ), so replace_map is order-safe.
- `flag_matching`: BigQuery has no auto-increment: flag every IDENTITY column, then drop the IDENTITY(seed,step) token. Flag first so the marker survives the strip.
- `replace_regex`: Drop the IDENTITY(seed,step) token now that the column is flagged.
- `replace_regex`: datetimeoffset(n) / bare -> TIMESTAMP (BigQuery TIMESTAMP has no precision argument).
- `replace_regex`: datetime2(n) / bare -> DATETIME (wall-clock; BigQuery DATETIME has no precision argument).
- `replace`: smalldatetime -> DATETIME (run before datetime; it contains 'datetime').
- `replace`: datetime (wall-clock) -> DATETIME (whole word, so it never bites an identifier or a residual function name).
- `replace`: uniqueidentifier -> STRING (BigQuery has no native GUID type).
- `replace`: bit -> BOOL (whole word). Any 0/1 default is carried in the DEFAULT-constraint TODO below.
- `replace`: nvarchar(max) -> STRING (run before varchar(max), which it contains).
- `replace`: varchar(max) -> STRING (BigQuery STRING is unbounded by default).
- `replace`: nvarchar(n) -> STRING(n) (BigQuery STRING is Unicode; the length limit is valid parameterized-STRING syntax).
- `replace`: varchar(n) -> STRING(n).
- `replace`: nchar -> STRING.
- `replace`: ntext -> STRING (LOB text type).
- `replace`: text -> STRING.
- `replace`: char(n) / bare char -> STRING.
- `replace`: smallmoney -> NUMERIC (run before money).
- `replace`: money -> NUMERIC (BigQuery fixed-point; 38,9).
- `replace`: decimal -> NUMERIC (precision/scale preserved and valid in BigQuery).
- `replace`: numeric -> NUMERIC (canonicalize the keyword spelling/case).
- `replace`: varbinary(max) -> BYTES (run before varbinary(n)/binary).
- `replace_regex`: varbinary(n) / bare varbinary -> BYTES.
- `replace_regex`: binary(n) / bare binary -> BYTES.
- `replace`: image -> BYTES (legacy LOB).
- `replace_map`: Remaining scalar type renames to BigQuery: integer family -> INT64, float/real -> FLOAT64, date/time keep, xml/hierarchyid/sql_variant -> STRING (no native type; review), geography/geometry -> GEOGRAPHY.
- `replace_regex`: A T-SQL computed/PERSISTED column has no runnable BigQuery column form: drop the column and leave a TODO in its place (the trailing comma goes with it; the preceding column keeps its own comma).
- `replace_regex`: Drop any index/constraint storage-option clause: WITH (PAD_INDEX = ...).
- `replace`: Drop the CREATE TABLE filegroup placement 'ON PRIMARY TEXTIMAGE_ON PRIMARY'.
- `replace`: Drop remaining ' ON PRIMARY' filegroup placement.
- `replace`: Drop ' ASC' from inline key-column lists.
- `replace`: Drop ' NONCLUSTERED' (no BigQuery equivalent).
- `replace`: Drop ' CLUSTERED' (BigQuery clustering is a table option, not a constraint keyword).
- `replace_regex`: BigQuery only accepts PRIMARY KEY ... NOT ENFORCED inside a CREATE TABLE, not via ALTER ADD CONSTRAINT: flag the whole PK batch (fold it into the CREATE TABLE with NOT ENFORCED).
- `replace_regex`: BigQuery has no UNIQUE constraint: flag the ALTER TABLE ADD CONSTRAINT ... UNIQUE batch.
- `replace_regex`: CHECK constraints are unsupported in BigQuery: flag the ALTER ... WITH CHECK ADD CONSTRAINT ... CHECK batch.
- `replace_regex`: SSMS scripts column defaults as separate ALTER TABLE ADD CONSTRAINT DEFAULT batches. Flag each with the (already BigQuery-flavored) default expression to fold inline into the CREATE TABLE column as DEFAULT.
- `replace`: BigQuery redeploy idiom: CREATE TABLE -> CREATE OR REPLACE TABLE (idempotent).
- `replace_regex`: Turn each remaining GO batch separator into a statement terminator plus a blank line.
- `run_mog`: Canonical whitespace finalize via the shared fragment: collapse double spaces left by dropped markers, tidy spaces before ';', strip trailing whitespace, and collapse blank-line runs.

## Tags

`sql` `mssql` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
