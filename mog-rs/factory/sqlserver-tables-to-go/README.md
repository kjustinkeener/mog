# SQL Server SSMS tables to Go structs

Generate Go structs (with db tags) from SSMS-scripted SQL Server tables

Generate idiomatic Go type declarations from a SSMS 'Script Table as CREATE' export, one 'type <Table> struct' per table. Strips the SSMS plumbing, removes bracket quoting, drops USE/GO batch separators, ALTER TABLE and CREATE INDEX statements, then rewrites each CREATE TABLE into a struct with one exported field per column. Field names are PascalCased for export, and the EXACT database column name is preserved losslessly in a `db:"..."` struct tag (compatible with sqlx / pgx). SQL Server types map to Go types (tinyint->int16, smallint->int16, int->int32, bigint->int64, real->float32, float->float64, bit->bool, char/varchar/nchar/nvarchar/text->string, uniqueidentifier->string, date/time/datetime/datetime2/datetimeoffset/smalldatetime->time.Time, binary/varbinary/image->[]byte). decimal/numeric/money/smallmoney map to string (Go has no native decimal) and are flagged; sql_variant maps to any and is flagged. Nullable columns become pointer types (*T); nilable types (slices) stay as-is. Emits the struct declarations only -- add your own 'package' clause and imports (time), then run gofmt to align fields. The Go sibling of sqlserver-tables-to-typescript. Assumes LF.

## Run

```
mog -m sqlserver-tables-to-go <file>
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
type CsgenOrders struct {
	OrderId int64 `db:"order_id"`
	Quantity int32 `db:"quantity"`
	SmallCount int16 `db:"small_count"`
	TinyFlag int16 `db:"tiny_flag"`
	CustomerCode string `db:"customer_code"`
	Description *string `db:"description"`
	LegacyNote *string `db:"legacy_note"`
	FixedCode *string `db:"fixed_code"`
	RegionCode *string `db:"region_code"`
	LongText *string `db:"long_text"`
	IsActive bool `db:"is_active"`
	DiscountPct string `db:"discount_pct"` // TODO(mog): exact-precision or currency column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for math.
	UnitPrice string `db:"unit_price"` // TODO(mog): exact-precision or currency column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for math.
	Ratio float64 `db:"ratio"`
	SmallRatio *float32 `db:"small_ratio"`
	CreatedAt time.Time `db:"created_at"`
	UpdatedAt *time.Time `db:"updated_at"`
```

_(... 7 more line(s))_

## Pipeline

- `run_mog`: Strip SSMS server noise (SET ANSI_NULLS, object banners, ON [PRIMARY]).
- `replace`: Remove SQL Server opening bracket quotes.
- `replace`: Remove SQL Server closing bracket quotes.
- `replace_regex`: Drop USE <db> batch header.
- `replace_regex`: Drop ALTER TABLE batches (constraints, defaults).
- `replace_regex`: Drop CREATE INDEX batches.
- `replace_regex`: Drop remaining GO batch separators.
- `replace_regex`: Normalize the leading tab indent SSMS uses to 4 spaces.
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE <schema>.<tbl>(' into a Go struct header + opening brace.
- `replace_regex`: Rewrite each table-closing ') ON [PRIMARY]...' into a closing brace.
- `replace_regex`: Drop IDENTITY(seed,increment) markers.
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (4-space indent, ending in bare NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag exact-precision / currency columns before they are mapped to string.
- `replace_regex`: date/time/datetime family -> time.Time (via sentinel).
- `replace_regex`: char/varchar/nchar/nvarchar/text -> string.
- `replace_regex`: uniqueidentifier -> string.
- `replace_regex`: decimal/numeric -> string (flagged below).
- `replace_regex`: money/smallmoney -> string (flagged below).
- `replace_regex`: real -> float32.
- `replace_regex`: float -> float64.
- `replace_regex`: bigint -> int64.
- `replace_regex`: smallint / tinyint -> int16.
- `replace_regex`: int -> int32.
- `replace_regex`: bit -> bool.
- `replace_regex`: varbinary -> []byte.
- `replace_regex`: binary -> []byte.
- `replace_regex`: image -> []byte.
- `replace_regex`: sql_variant -> any (flagged below).
- `replace_regex`: Resolve the time sentinel to time.Time (kept separate to avoid re-matching the 'time' keyword).
- `flag_matching`: Flag SQL Server sql_variant columns (mapped to any).
- `replace_regex`: Rewrite nullable columns into a pointer-typed Go field carrying a db tag placeholder.
- `replace_regex`: Rewrite non-nullable columns into a Go field carrying a db tag placeholder.
- `replace_regex`: Nilable types (slices) don't need a pointer for nullability -- drop it.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each struct name (the table identifier).
- `to_pascal`: PascalCase each field name (the column identifier); the db tag keeps the exact column name.
- `replace_regex`: Resolve the db tag placeholder into a real Go struct tag.
- `replace_regex`: Convert the 4-space column indent to a tab (Go convention).
- `replace_regex`: Collapse multiple blank lines to a single blank between structs.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.

## Tags

`sql` `mssql` `go` `codegen` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
