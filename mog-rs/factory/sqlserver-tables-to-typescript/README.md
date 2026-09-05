# SQL Server tables to TypeScript interfaces

Generate TypeScript interfaces from SSMS-scripted SQL Server table DDL

Generate idiomatic TypeScript 'export interface' declarations from SQL Server SSMS 'Generate Scripts' / mssql-scripter table DDL, one interface per table. Strips the SSMS scaffolding, unquotes [bracketed] identifiers, drops GO separators, and discards non-table statements (USE, ALTER TABLE constraints/DEFAULTs, CREATE INDEX). Each column becomes a two-space-indented member. Type map: bigint/int/smallint/tinyint, decimal(p,s)/numeric/money/smallmoney/float/real -> number; bit -> boolean; char/nchar/varchar/nvarchar((n)/(max))/text/ntext/uniqueidentifier -> string; binary/varbinary((max))/image -> Uint8Array; date/time(n)/datetime/datetime2(n)/datetimeoffset(n)/smalldatetime -> string (ISO-8601; no native TS date type). Nullable columns get a ' | null' union; NOT NULL columns get the plain type. Interface names are PascalCased; member names keep the DB snake_case. sql_variant maps to 'unknown' with a // TODO(mog) comment. Sibling of sqlserver-tables-to-csharp and postgres-tables-to-typescript. Not a full T-SQL parser. Assumes LF.

## Run

```
mog -m sqlserver-tables-to-typescript <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [lab]
GO
/****** Object:  Table [dbo].[csgen_orders]    Script Date: Sat 8 29 2026  9:18:25 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[csgen_orders](
	[order_id] [bigint] IDENTITY(1,1) NOT NULL,
	[quantity] [int] NOT NULL,
	[small_count] [smallint] NOT NULL,
	[tiny_flag] [tinyint] NOT NULL,
	[customer_code] [nvarchar](50) NOT NULL,
	[description] [nvarchar](max) NULL,
	[legacy_note] [varchar](200) NULL,
	[fixed_code] [char](10) NULL,
	[region_code] [nchar](2) NULL,
	[long_text] [text] NULL,
```

_(... 25 more line(s))_

Output:

```
export interface CsgenOrders {
  order_id: number;
  quantity: number;
  small_count: number;
  tiny_flag: number;
  customer_code: string;
  description: string | null;
  legacy_note: string | null;
  fixed_code: string | null;
  region_code: string | null;
  long_text: string | null;
  is_active: boolean;
  discount_pct: number;
  unit_price: number;
  ratio: number;
  small_ratio: number | null;
  created_at: string;
  updated_at: string | null;
```

_(... 7 more line(s))_

## Pipeline

- `run_mog`: Remove SSMS server/session/DROP scaffolding (object banners, SET batches, CREATE/DROP DATABASE, the guarded DROP phase, extended properties). Runs while GO batches are intact.
- `replace`: Strip T-SQL square-bracket quoting: opening bracket.
- `replace`: Strip T-SQL square-bracket quoting: closing bracket.
- `replace_regex`: Drop the leading USE <db> batch (TypeScript has no database-switch statement).
- `replace_regex`: Drop every ALTER TABLE batch (PRIMARY KEY / UNIQUE / FOREIGN KEY / CHECK / DEFAULT constraints), each spanning up to its GO.
- `replace_regex`: Drop standalone CREATE INDEX batches (clustered/nonclustered/unique) with their WITH(...) options, each spanning up to its GO.
- `replace_regex`: Remove the remaining GO batch terminators (the one after each CREATE TABLE).
- `replace_regex`: Normalize the SSMS leading tab on each column line to four spaces (matches the member regexes below).
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE dbo.<tbl>(' into a TypeScript interface header + opening brace.
- `replace_regex`: Rewrite each table-closing line ') ON PRIMARY ...' (or a bare ')') into a closing brace.
- `replace_regex`: Drop the IDENTITY(seed,step) column marker (TypeScript has no auto-increment).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (a column line ending in a bare NULL, not NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `replace_regex`: date / time(n) / datetime / datetime2(n) / datetimeoffset(n) / smalldatetime -> string (ISO-8601; no native TS date type).
- `replace_regex`: nvarchar / varchar / nchar / char / ntext / text (with or without (n) / (max)) -> string.
- `replace_regex`: uniqueidentifier -> string.
- `replace_regex`: decimal(p,s) / numeric(p,s) / bare decimal / numeric -> number.
- `replace_regex`: smallmoney / money -> number.
- `replace_regex`: float(n) / bare float / real -> number.
- `replace_regex`: bigint / smallint / tinyint / int -> number.
- `replace_regex`: bit -> boolean.
- `replace_regex`: varbinary(n) / varbinary(max) / bare varbinary -> Uint8Array (run before binary).
- `replace_regex`: binary(n) / bare binary -> Uint8Array.
- `replace_regex`: image -> Uint8Array.
- `replace_regex`: sql_variant -> unknown.
- `flag_matching`: Flag the mapped sql_variant columns (now 'unknown'): runtime-typed. Flagged after the type map so the literal 'sql_variant' in the comment is not itself rewritten.
- `replace_regex`: Rewrite nullable columns into a TypeScript member with a ' | null' union.
- `replace_regex`: Rewrite non-nullable columns into a TypeScript member.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each interface name (the table identifier); member names keep their snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between interfaces.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `mssql` `typescript` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
