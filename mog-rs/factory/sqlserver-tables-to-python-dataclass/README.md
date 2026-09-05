# SQL Server SSMS tables to Python dataclasses

Generate Python dataclasses from SSMS-scripted SQL Server tables

Generate idiomatic Python dataclasses from a SSMS 'Script Table as CREATE' export, one '@dataclass' per table. Strips the SSMS plumbing, removes bracket quoting, drops USE/GO batch separators, ALTER TABLE and CREATE INDEX statements, then rewrites each CREATE TABLE into a dataclass with one 4-space-indented annotated attribute per column. Attribute names are kept EXACTLY as in the database (snake_case, PEP 8, lossless); only the class name is PascalCased. SQL Server types map to Python types (int family->int, bit->bool, real/float->float, decimal/numeric/money->Decimal, char/varchar/nchar/nvarchar/text->str, uniqueidentifier->UUID, date->date, time->time, datetime/datetime2/datetimeoffset/smalldatetime->datetime, binary/varbinary/image->bytes). Nullable columns get a 'T | None' union. sql_variant maps to typing.Any and is flagged inline with a # TODO(mog): comment. Emits a ready-to-import module with the needed imports at the top. The Python sibling of sqlserver-tables-to-typescript. Assumes LF.

## Run

```
mog -m sqlserver-tables-to-python-dataclass <file>
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
from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from typing import Any
from uuid import UUID

@dataclass
class CsgenOrders:
    order_id: int
    quantity: int
    small_count: int
    tiny_flag: int
    customer_code: str
    description: str | None
    legacy_note: str | None
    fixed_code: str | None
    region_code: str | None
```

_(... 14 more line(s))_

## Pipeline

- `run_mog`: Strip SSMS server noise (SET ANSI_NULLS, object banners, ON [PRIMARY]).
- `replace`: Remove SQL Server opening bracket quotes.
- `replace`: Remove SQL Server closing bracket quotes.
- `replace_regex`: Drop USE <db> batch header.
- `replace_regex`: Drop ALTER TABLE batches.
- `replace_regex`: Drop CREATE INDEX batches.
- `replace_regex`: Drop remaining GO batch separators.
- `replace_regex`: Normalize the leading tab indent SSMS uses to 4 spaces.
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Rewrite each 'CREATE TABLE <schema>.<tbl>(' into a @dataclass header.
- `replace_regex`: Remove each table-closing ') ON [PRIMARY]...' line.
- `replace_regex`: Drop IDENTITY(seed,increment) markers.
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (4-space indent, ending in bare NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `replace_regex`: datetime family -> datetime.
- `replace_regex`: date -> date.
- `replace_regex`: time -> time.
- `replace_regex`: char/varchar/nchar/nvarchar/text -> str.
- `replace_regex`: uniqueidentifier -> UUID.
- `replace_regex`: decimal/numeric -> Decimal.
- `replace_regex`: money/smallmoney -> Decimal.
- `replace_regex`: real / float -> float.
- `replace_regex`: int family -> int.
- `replace_regex`: bit -> bool.
- `replace_regex`: varbinary -> bytes.
- `replace_regex`: binary -> bytes.
- `replace_regex`: image -> bytes.
- `replace_regex`: sql_variant -> Any (flagged below).
- `flag_matching`: Flag SQL Server sql_variant columns (mapped to Any).
- `replace_regex`: Rewrite nullable columns into an annotated attribute with a '| None' union (4-space indent).
- `replace_regex`: Rewrite non-nullable columns into an annotated attribute (4-space indent).
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the table identifier); attributes stay snake_case.
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.
- `replace_regex`: Prepend the future-annotations pragma and dataclass/datetime/decimal/uuid/typing imports.

## Tags

`sql` `mssql` `python` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
