# SQL Server (SSMS/mssql-scripter) DDL to MySQL

Convert real SQL Server script output to MySQL DDL

Convert real SQL Server script output (SSMS Generate Scripts / mssql-scripter) to MySQL DDL. Strips the SET/banner/DROP noise, then the USE/GO batch scaffolding, the [bracket] quoting and dbo. schema, the ON [PRIMARY]/TEXTIMAGE_ON filegroup clauses, and the separate ALTER TABLE ADD CONSTRAINT PRIMARY KEY CLUSTERED WITH(...) block. Folds the identity column's primary key inline (IDENTITY(1,1) NOT NULL -> AUTO_INCREMENT PRIMARY KEY, since MySQL requires the auto column to be a key), and maps types (nvarchar(max)->TEXT, nvarchar->varchar, bit->tinyint(1), datetime2(n)->datetime, uniqueidentifier->char(36)).

## Run

```
mog -m sqlserver-tables-to-mysql <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [lab]
GO
/****** Object:  Table [dbo].[Customers]    Script Date: Fri 8 28 2026  7:13:00 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[Customers](
	[CustomerID] [int] IDENTITY(1,1) NOT NULL,
	[Name] [nvarchar](200) NOT NULL,
	[Email] [nvarchar](255) NULL,
	[Notes] [nvarchar](max) NULL,
	[Balance] [decimal](18, 2) NULL,
	[IsActive] [bit] NOT NULL,
	[CreatedAt] [datetime2](7) NOT NULL,
	[RowGuid] [uniqueidentifier] NOT NULL
) ON [PRIMARY] TEXTIMAGE_ON [PRIMARY]
GO
```

_(... 6 more line(s))_

Output:

```


CREATE TABLE Customers(
	CustomerID int AUTO_INCREMENT PRIMARY KEY,
	Name varchar(200) NOT NULL,
	Email varchar(255) NULL,
	Notes TEXT NULL,
	Balance decimal(18, 2) NULL,
	IsActive tinyint(1) NOT NULL,
	CreatedAt datetime NOT NULL,
	RowGuid char(36) NOT NULL
)

-- TODO(mog): primary key folded into CREATE TABLE
```

## Pipeline

- `run_mog`
- `replace_regex`: Drop USE [db] batch.
- `replace_regex`: Drop GO batch separators.
- `replace_regex`: Drop the separate ALTER ... PRIMARY KEY CLUSTERED WITH(...) block (folded inline below).
- `replace`: Drop [ bracket.
- `replace`: Drop ] bracket.
- `replace`: Drop dbo. schema.
- `replace_regex`: IDENTITY(1,1) NOT NULL -> AUTO_INCREMENT PRIMARY KEY (fold PK; MySQL needs the auto col to be a key).
- `replace_regex`: Any remaining IDENTITY(n,n) -> AUTO_INCREMENT.
- `replace_regex`: Drop the ) ON PRIMARY TEXTIMAGE_ON PRIMARY filegroup clause.
- `replace_regex`: nvarchar(max) -> TEXT.
- `replace_regex`: nvarchar/nchar -> varchar/char.
- `replace_regex`: bit -> tinyint(1).
- `replace_regex`: datetime2(n) -> datetime.
- `replace_regex`: uniqueidentifier -> char(36).
- `replace_regex`: Collapse blank lines.

## Tags

`sql` `mssql` `mysql` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
