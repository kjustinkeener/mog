# SQL Server tables to C# classes

Generate C# POCO classes from SSMS-scripted SQL Server table DDL

Generate idiomatic C# POCO classes from SQL Server SSMS 'Generate Scripts' / mssql-scripter table DDL, one class per table. Strips the SSMS scaffolding, unquotes the [bracketed] identifiers, discards every non-table statement (USE, the ALTER TABLE constraint/DEFAULT phase, CREATE INDEX), then rewrites each CREATE TABLE into a 'public class' with one auto-property per column. T-SQL types map to C# types (bigint->long, int->int, smallint->short, tinyint->byte, bit->bool, nvarchar/varchar/nchar/char/text/ntext->string, decimal/numeric(p,s)->decimal, money/smallmoney->decimal, float->double, real->float, datetime/datetime2/smalldatetime->DateTime, datetimeoffset->DateTimeOffset, date->DateOnly, time->TimeOnly, uniqueidentifier->Guid, varbinary/binary/image->byte[], xml->string). IDENTITY markers are dropped. Nullable columns (declared NULL, not NOT NULL) get a C# nullable type ('?'). Table and column identifiers are PascalCased (order_id -> OrderId). Exotic types with no clean C# equivalent (sql_variant, hierarchyid, geography, geometry) map to string and are flagged with a // TODO(mog): comment. The SQL-Server-source sibling of postgres-tables-to-csharp. Not a full T-SQL parser. Assumes LF.

## Run

```
mog -m sqlserver-tables-to-csharp <file>
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
public class CsgenOrders
{
    public long OrderId { get; set; }
    public int Quantity { get; set; }
    public short SmallCount { get; set; }
    public byte TinyFlag { get; set; }
    public string CustomerCode { get; set; }
    public string? Description { get; set; }
    public string? LegacyNote { get; set; }
    public string? FixedCode { get; set; }
    public string? RegionCode { get; set; }
    public string? LongText { get; set; }
    public bool IsActive { get; set; }
    public decimal DiscountPct { get; set; }
    public decimal UnitPrice { get; set; }
    public double Ratio { get; set; }
    public float? SmallRatio { get; set; }
    public DateTime CreatedAt { get; set; }
```

_(... 8 more line(s))_

## Pipeline

- `run_mog`: Remove SSMS server/session/DROP scaffolding (object banners, SET batches, CREATE/DROP DATABASE, the guarded DROP phase, extended properties). Runs while GO batches are intact.
- `replace`: Strip T-SQL square-bracket quoting: opening bracket.
- `replace`: Strip T-SQL square-bracket quoting: closing bracket.
- `replace_regex`: Drop the leading USE <db> batch (Postgres/C# have no database-switch statement).
- `replace_regex`: Drop every ALTER TABLE batch (PRIMARY KEY / UNIQUE / FOREIGN KEY / CHECK / DEFAULT constraints), each spanning up to its GO.
- `replace_regex`: Drop standalone CREATE INDEX batches (clustered/nonclustered/unique), each spanning up to its GO.
- `replace_regex`: Remove the remaining GO batch terminators (the one after each CREATE TABLE).
- `replace_regex`: Normalize the SSMS leading tab on each column line to four spaces (matches the property regexes below).
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE dbo.<tbl>(' into a C# class header + opening brace.
- `replace_regex`: Rewrite each table-closing line ') ON PRIMARY ...' (or a bare ')') into a closing brace.
- `replace_regex`: Drop the IDENTITY(seed,step) column marker (C# does not carry auto-increment on the POCO).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (a column line ending in a bare NULL, not NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag sql_variant columns: mapped to string.
- `flag_matching`: Flag hierarchyid columns: mapped to string.
- `flag_matching`: Flag geography columns: mapped to string.
- `flag_matching`: Flag geometry columns: mapped to string.
- `replace_regex`: datetimeoffset(n) / bare datetimeoffset -> DateTimeOffset (run before datetime).
- `replace_regex`: datetime2(n) / bare datetime2 -> DateTime (run before datetime).
- `replace_regex`: smalldatetime -> DateTime (run before datetime; it contains 'datetime').
- `replace_regex`: datetime -> DateTime.
- `replace_regex`: date -> DateOnly (runs after the datetime family, which it does not bite into: no word boundary inside 'DateTime').
- `replace_regex`: time(n) / bare time -> TimeOnly.
- `replace_regex`: nvarchar / varchar / nchar / char / ntext / text (with or without (n) / (max)) -> string.
- `replace_regex`: decimal(p,s) / numeric(p,s) -> decimal.
- `replace_regex`: bare decimal / numeric -> decimal.
- `replace_regex`: smallmoney -> decimal (run before money).
- `replace_regex`: money -> decimal.
- `replace_regex`: bigint -> long (run before int).
- `replace_regex`: smallint -> short (run before int).
- `replace_regex`: tinyint -> byte (run before int).
- `replace_regex`: int -> int.
- `replace_regex`: bit -> bool.
- `replace_regex`: float(n) / bare float -> double.
- `replace_regex`: real -> float.
- `replace_regex`: uniqueidentifier -> Guid.
- `replace_regex`: varbinary(n) / varbinary(max) / bare varbinary -> byte[] (run before binary).
- `replace_regex`: binary(n) / bare binary -> byte[].
- `replace_regex`: image -> byte[].
- `replace_regex`: xml -> string.
- `replace_regex`: sql_variant -> string (flagged above).
- `replace_regex`: hierarchyid -> string (flagged above).
- `replace_regex`: geography -> string (flagged above).
- `replace_regex`: geometry -> string (flagged above).
- `replace_regex`: Rewrite nullable columns into a C# auto-property with a nullable type.
- `replace_regex`: Rewrite non-nullable columns into a C# auto-property.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier).
- `to_pascal`: PascalCase each property name (the column identifier).
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `mssql` `csharp` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
