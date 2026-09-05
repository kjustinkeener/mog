# SQL Server (SSMS) to Snowflake table DDL

Convert SQL Server table DDL to Snowflake

Convert a SQL Server SSMS 'Generate Scripts' / mssql-scripter table export to runnable Snowflake DDL. Turns sp_addextendedproperty 'MS_Description' into COMMENT ON TABLE / COLUMN, strips the SSMS server/session/DROP scaffolding, drops T-SQL [bracket] quoting and the dbo. qualifier, and keeps IDENTITY(seed,step) and PRIMARY KEY as real DDL. Function map: newid/newsequentialid->UUID_STRING, sysdatetime/getdate->CURRENT_TIMESTAMP, *utc*->SYSDATE. Type map: NVARCHAR(MAX)/VARCHAR(MAX)->VARCHAR, NVARCHAR(n)->VARCHAR(n), BIT->BOOLEAN, MONEY->NUMBER(19,4), DECIMAL/NUMERIC->NUMBER, UNIQUEIDENTIFIER->VARCHAR(36), VARBINARY/BINARY/IMAGE->BINARY, DATETIME2(n)/DATETIME/SMALLDATETIME->TIMESTAMP_NTZ, DATETIMEOFFSET(n)->TIMESTAMP_TZ. Flags constructs with no runnable Snowflake form as -- TODO(mog): PERSISTED computed columns, separate DEFAULT constraints, CHECK constraints, UNIQUE via ALTER, and secondary CREATE INDEX. Not a semantic transpiler.

## Run

```
mog -m sqlserver-tables-to-snowflake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [lab]
GO
/****** Object:  Table [dbo].[ms2sf_orders]    Script Date: Sat 8 29 2026  12:24:56 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[ms2sf_orders](
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

CREATE OR REPLACE TABLE ms2sf_orders(
	order_id bigint IDENTITY(1,1) NOT NULL,
	order_uid VARCHAR(36) NOT NULL,
	customer_code VARCHAR(50) NOT NULL,
	description VARCHAR NULL,
	status_flag BOOLEAN NOT NULL,
	quantity int NOT NULL,
	unit_price NUMBER(19,4) NOT NULL,
	discount_pct NUMBER(5, 2) NOT NULL,
	-- TODO(mog): computed column line_total = (quantity*unit_price) [PERSISTED] has no Snowflake column form; recreate it as a view or a downstream derived column.
	notes VARCHAR NULL,
	payload BINARY NULL,
	created_at TIMESTAMP_NTZ(7) NOT NULL,
	updated_at TIMESTAMP_TZ(7) NULL,
	legacy_ts TIMESTAMP_NTZ NULL
);
```

_(... 26 more line(s))_

## Pipeline

- `eol_lf`: Normalize to LF first so every later newline-sensitive regex is reliable. Output stays LF (standard for SQL scripts).
- `replace_regex`: sp_addextendedproperty 'MS_Description' on a COLUMN -> Snowflake COMMENT ON COLUMN table.column. Runs BEFORE the strip fragment (which would otherwise delete the extended-property calls) and before the TABLE rule so a column property is not mis-read as a table property. Leaves the trailing GO for the finalize pass to turn into ';'.
- `replace_regex`: sp_addextendedproperty 'MS_Description' on a TABLE or VIEW -> Snowflake COMMENT ON TABLE object.
- `run_mog`: Remove SSMS server/session/DROP scaffolding (object banners, SET options, CREATE/ALTER DATABASE, the guarded DROP phase, ALTER TABLE CHECK CONSTRAINT re-enables). Shared fragment; runs on the raw SSMS text while GO batches are intact. The extended-property step above already rewrote MS_Description, so nothing is lost here.
- `replace_regex`: Snowflake has no secondary indexes: replace the whole CREATE [NON]CLUSTERED INDEX ... GO batch with a TODO. Runs before bracket-stripping so it can consume the full multi-line INCLUDE/WITH/ON PRIMARY batch in one match.
- `replace`: Strip T-SQL square-bracket quoting: opening bracket.
- `replace`: Strip T-SQL square-bracket quoting: closing bracket.
- `replace`: Drop the dbo. schema qualifier so objects resolve in the active Snowflake schema.
- `replace_regex`: USE <db> + its GO -> a documenting comment (Snowflake selects a database/schema on the session, not with USE <db> in a DDL script).
- `replace_map`: Map T-SQL builtins to Snowflake. Runs before the type maps so a sys*datetime* function name is never corrupted, and before the DEFAULT-constraint TODO so the flagged default text is already Snowflake-flavored. None of these keys is a substring of another (the trailing () differ), so replace_map is order-safe.
- `replace_regex`: datetimeoffset(n) -> TIMESTAMP_TZ(n) (keeps fractional-second precision). Run before datetime2/datetime.
- `replace`: bare datetimeoffset -> TIMESTAMP_TZ.
- `replace_regex`: datetime2(n) -> TIMESTAMP_NTZ(n).
- `replace`: bare datetime2 -> TIMESTAMP_NTZ.
- `replace`: smalldatetime -> TIMESTAMP_NTZ (run before datetime; it contains 'datetime').
- `replace`: datetime (wall-clock) -> TIMESTAMP_NTZ (whole word, so it never bites an identifier or a residual function name). Produces TIMESTAMP_NTZ, which has no bare 'datetime' left to re-match.
- `replace`: uniqueidentifier -> VARCHAR(36) (Snowflake has no native GUID type; store the canonical 36-char string).
- `replace`: bit -> BOOLEAN (whole word). Any 0/1 default is carried in the DEFAULT-constraint TODO below.
- `replace`: nvarchar(max) -> VARCHAR (run before varchar(max), which it contains).
- `replace`: varchar(max) -> VARCHAR (Snowflake VARCHAR defaults to the 16 MB max).
- `replace`: nvarchar(n) -> VARCHAR(n) (Snowflake VARCHAR is already Unicode; no national type).
- `replace`: nchar -> CHAR.
- `replace`: ntext -> VARCHAR (LOB text type).
- `replace`: text -> VARCHAR.
- `replace`: smallmoney -> NUMBER(10,4) (run before money).
- `replace`: money -> NUMBER(19,4).
- `replace`: decimal -> NUMBER (Snowflake's canonical fixed-point type; precision/scale preserved).
- `replace`: numeric -> NUMBER.
- `replace`: varbinary(max) -> BINARY (run before varbinary(n)/binary).
- `replace_regex`: varbinary(n) / bare varbinary -> BINARY.
- `replace_regex`: binary(n) / bare binary -> BINARY.
- `replace`: image -> BINARY (legacy LOB).
- `replace_map`: Remaining scalar/spatial/semi-structured type renames: float/real -> FLOAT, xml/hierarchyid -> VARCHAR (no native type; flag for review), sql_variant -> VARIANT, geography/geometry -> GEOGRAPHY/GEOMETRY.
- `replace_regex`: A T-SQL computed/PERSISTED column has no runnable Snowflake column form: drop the column and leave a TODO in its place (the trailing comma goes with it; the preceding column keeps its own comma).
- `replace_regex`: Drop any index/constraint storage-option clause: WITH (PAD_INDEX = ...).
- `replace`: Drop the CREATE TABLE filegroup placement 'ON PRIMARY TEXTIMAGE_ON PRIMARY'.
- `replace`: Drop remaining ' ON PRIMARY' filegroup placement.
- `replace`: Drop ' ASC' from inline key-column lists (Snowflake default; invalid inside a PRIMARY KEY column list).
- `replace`: Drop ' NONCLUSTERED' (no Snowflake equivalent).
- `replace`: Drop ' CLUSTERED' (Snowflake has no clustered-index syntax on a constraint).
- `replace_regex`: UNIQUE constraints added via ALTER TABLE are metadata-only in Snowflake and are not accepted by every target; flag the whole batch (fold UNIQUE inline into the CREATE TABLE if you want the metadata constraint). Runs after the storage/CLUSTERED cleanup so the batch is already tidy.
- `replace_regex`: CHECK constraints are unsupported in Snowflake: flag the ALTER ... WITH CHECK ADD CONSTRAINT ... CHECK batch (enforce the rule in ELT or a view instead).
- `replace_regex`: SSMS scripts column defaults as separate ALTER TABLE ADD CONSTRAINT DEFAULT batches. Snowflake ALTER COLUMN SET DEFAULT accepts only a sequence, so flag each with the (already Snowflake-flavored) default expression to fold inline into the CREATE TABLE.
- `replace`: Snowflake export idiom: CREATE TABLE -> CREATE OR REPLACE TABLE (idempotent redeploys).
- `replace_regex`: Turn each remaining GO batch separator into a statement terminator plus a blank line (CREATE TABLE, the PRIMARY KEY ALTER, and the COMMENT ON statements).
- `run_mog`: Canonical whitespace finalize via the shared fragment: collapse the double spaces left by dropped markers, tidy spaces before ';', strip trailing whitespace, and collapse blank-line runs.

## Tags

`sql` `mssql` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
