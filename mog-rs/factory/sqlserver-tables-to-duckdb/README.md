# SQL Server (SSMS) table DDL to DuckDB

Convert SQL Server table DDL to DuckDB

Convert a SQL Server SSMS 'Generate Scripts' (mssql-scripter) table export to runnable DuckDB DDL. Strips all SSMS server/session/DROP scaffolding and GO batches; rewrites [bracket] quoting and the dbo. schema prefix; maps every T-SQL type (NVARCHAR/VARCHAR(MAX)->VARCHAR, NVARCHAR(n)/NCHAR/NTEXT/TEXT->VARCHAR, DATETIME2/DATETIME/SMALLDATETIME->TIMESTAMP, DATETIMEOFFSET->TIMESTAMPTZ, BIT->BOOLEAN, UNIQUEIDENTIFIER->UUID, VARBINARY/BINARY/IMAGE->BLOB, MONEY->DECIMAL(19,4), SMALLMONEY->DECIMAL(10,4), TINYINT->SMALLINT, FLOAT->DOUBLE); maps functions (newsequentialid()/newid()->uuid(), sysdatetime()/getdate() family->now()); rewrites DEFAULT constraints to ALTER COLUMN ... SET DEFAULT (unwrapping SSMS doubled parens); keeps PRIMARY KEY as an ALTER TABLE ADD CONSTRAINT; converts a UNIQUE constraint to CREATE UNIQUE INDEX (DuckDB cannot ADD UNIQUE via ALTER); maps a PERSISTED computed column to a DuckDB generated column; and flags the non-portable bits with -- TODO(mog): IDENTITY (no DuckDB auto-increment), covering-index INCLUDE columns, PERSISTED->VIRTUAL semantics, and CHECK constraints (DuckDB has no ALTER ADD CHECK). Not a full SQL parser.

## Run

```
mog -m sqlserver-tables-to-duckdb <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [lab]
GO
/****** Object:  Table [dbo].[ms2dd_orders]    Script Date: Sat 8 29 2026  12:25:25 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[ms2dd_orders](
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
-- Database: lab

CREATE TABLE ms2dd_orders(
-- TODO(mog): SQL Server IDENTITY dropped; DuckDB has no auto-increment column. Create a SEQUENCE and set this column's DEFAULT to nextval('seq') to reproduce it.
	order_id bigint NOT NULL,
	order_uid UUID NOT NULL,
	customer_code VARCHAR(50) NOT NULL,
	description VARCHAR NULL,
	status_flag BOOLEAN NOT NULL,
	quantity int NOT NULL,
	unit_price DECIMAL(19,4) NOT NULL,
	discount_pct decimal(5, 2) NOT NULL,
	line_total AS (quantity*unit_price), -- TODO(mog): SQL Server stored computed column (PERSISTED) mapped to a DuckDB VIRTUAL generated column; it is recomputed on read, not materialized.
	notes VARCHAR NULL,
	payload BLOB NULL,
	created_at TIMESTAMP NOT NULL,
	updated_at TIMESTAMPTZ NULL,
	legacy_ts TIMESTAMP NULL
```

_(... 32 more line(s))_

## Pipeline

- `eol_lf`: Normalize to LF first so every later CRLF-sensitive and GO-batch regex is reliable. Output stays LF (standard for SQL scripts).
- `replace_regex`: sp_addextendedproperty 'MS_Description' on a COLUMN -> DuckDB COMMENT ON COLUMN table.column. Runs BEFORE the strip fragment (which would otherwise delete the extended-property calls) and before the TABLE rule so a column property is not mis-read as a table property. No trailing ';' -- the GO pass adds it.
- `replace_regex`: sp_addextendedproperty 'MS_Description' on a TABLE or VIEW -> DuckDB COMMENT ON TABLE object. No trailing ';' -- the GO pass adds it.
- `run_mog`: Strip SQL Server server/session/DROP scaffolding (object banners, SET options, CREATE/DROP DATABASE, ALTER DATABASE settings, the DROP phase, constraint re-enables, any remaining extended properties). Runs on the raw SSMS text while GO batches are intact. Reusable across MSSQL->target converters.
- `replace`: Strip T-SQL square-bracket quoting: opening bracket.
- `replace`: Strip T-SQL square-bracket quoting: closing bracket.
- `replace`: Drop the SQL Server 'dbo.' schema qualifier so objects are unqualified in DuckDB's default schema (main).
- `replace_regex`: USE <db> + its GO -> a documenting comment (DuckDB has no USE-for-database; you ATTACH/connect to a database file instead).
- `replace`: Function map: newsequentialid() -> uuid() (DuckDB has no sequential-GUID; uuid() is the portable default).
- `replace`: Function map: newid() -> uuid().
- `replace`: Function map: sysdatetimeoffset() -> now(). Must run before the datetimeoffset/datetime type maps, which would otherwise corrupt the name.
- `replace`: Function map: sysutcdatetime() -> now(). Before the datetime type map.
- `replace`: Function map: sysdatetime() -> now(). Before the datetime type map.
- `replace`: Function map: getutcdate() -> now().
- `replace`: Function map: getdate() -> now().
- `replace_regex`: Type map: datetimeoffset(n) -> TIMESTAMPTZ (DuckDB TIMESTAMPTZ has no scripted precision). Run before datetime.
- `replace`: Type map: bare datetimeoffset -> TIMESTAMPTZ.
- `replace_regex`: Type map: datetime2(n) -> TIMESTAMP. Run before datetime.
- `replace`: Type map: bare datetime2 -> TIMESTAMP.
- `replace`: Type map: smalldatetime -> TIMESTAMP (run before datetime; it contains 'datetime').
- `replace`: Type map: datetime -> TIMESTAMP (whole word, so it never bites into an identifier or a sys*datetime* function that slipped through).
- `replace`: Type map: tinyint -> SMALLINT (DuckDB TINYINT is signed 1-byte; T-SQL tinyint is 0-255, so SMALLINT is the safe portable choice).
- `replace`: Type map: uniqueidentifier -> UUID.
- `replace`: Type map: bit -> BOOLEAN (whole word). DuckDB accepts a 0/1 literal DEFAULT on a BOOLEAN column, so bit DEFAULTs port cleanly.
- `replace`: Type map: nvarchar(max) -> VARCHAR (DuckDB VARCHAR is unbounded). Run before varchar(max), which it contains.
- `replace`: Type map: varchar(max) -> VARCHAR.
- `replace`: Type map: nvarchar(n) -> VARCHAR(n) (DuckDB VARCHAR is already Unicode; no separate national type). The (n) length is preserved and ignored by DuckDB.
- `replace`: Type map: nchar -> VARCHAR.
- `replace`: Type map: ntext -> VARCHAR (deprecated T-SQL LOB).
- `replace`: Type map: text -> VARCHAR (deprecated T-SQL LOB; whole word so it never bites a column comment).
- `replace`: Type map: smallmoney -> DECIMAL(10,4) (run before money).
- `replace`: Type map: money -> DECIMAL(19,4).
- `replace`: Type map: float -> DOUBLE.
- `replace`: Type map: varbinary(max) -> BLOB (run before varbinary(n)/binary).
- `replace_regex`: Type map: varbinary(n) / bare varbinary -> BLOB (DuckDB BLOB is unsized).
- `replace_regex`: Type map: binary(n) / bare binary -> BLOB.
- `replace`: Type map: image -> BLOB.
- `flag_matching`: Flag IDENTITY: DuckDB has no auto-increment column type. Stamp a TODO on the line before it.
- `replace_regex`: Drop the IDENTITY(seed,step) clause (handled by the TODO above); the column keeps its integer type + NOT NULL.
- `flag_matching`: Flag a PERSISTED computed column: DuckDB has no STORED generated column, so it becomes a VIRTUAL one (recomputed on read).
- `replace`: Drop the PERSISTED keyword; 'col AS (expr)' is a valid DuckDB (virtual) generated column.
- `replace_regex`: Constraint ALTER: '  WITH CHECK ADD  ' -> ' ADD ' (DuckDB has no WITH CHECK ADD).
- `replace_regex`: Drop any index/constraint storage-option clause: WITH (PAD_INDEX = ...). One regex covers PK/UNIQUE constraints and standalone CREATE INDEX. A trailing ON PRIMARY is removed below.
- `flag_matching`: Flag covering-index INCLUDE columns: DuckDB indexes do not support INCLUDE.
- `replace_regex`: Drop the INCLUDE (...) covering-column list (handled by the TODO above).
- `replace`: Drop ' CLUSTERED' (DuckDB has no clustered-index syntax).
- `replace`: Drop ' NONCLUSTERED' (no DuckDB equivalent; CREATE NONCLUSTERED INDEX -> CREATE INDEX).
- `replace`: Drop ' ASC' from key/index column lists (DuckDB default; invalid inside these inline column lists).
- `replace`: Drop filegroup placement 'ON PRIMARY TEXTIMAGE_ON PRIMARY'.
- `replace`: Drop remaining ' ON PRIMARY' filegroup placement.
- `replace_regex`: UNIQUE constraint -> CREATE UNIQUE INDEX. DuckDB cannot ADD a UNIQUE constraint via ALTER TABLE, but a UNIQUE INDEX enforces the same rule and executes. Runs after the storage/CLUSTERED/ASC cleanup so the column list is bare.
- `replace_regex`: CHECK constraint ALTER -> a commented-out TODO. DuckDB has no ALTER TABLE ADD CHECK, so the constraint is preserved as a comment for you to enforce in your pipeline.
- `replace_regex`: Rewrite DEFAULT constraints to DuckDB ALTER TABLE ... ALTER COLUMN ... SET DEFAULT, unwrapping the SSMS captured expression (doubled parens like ((0)) become (0), which DuckDB accepts). No trailing ';' -- the GO pass adds it.
- `replace_regex`: Turn each GO batch separator into a statement terminator, plus a blank line between statements.
- `run_mog`: Canonical whitespace finalize via the shared fragment (collapse double spaces left by dropped markers, tidy spaces before ';', trim trailing whitespace, squeeze blank lines).

## Tags

`sql` `mssql` `duckdb` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
