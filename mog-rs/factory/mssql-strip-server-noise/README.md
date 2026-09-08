# Strip SQL Server script noise

Strip SSMS server and DROP scaffolding from T-SQL

Remove the server/session/DROP scaffolding that SSMS 'Generate Scripts' emits around real object DDL: object banners, SET options, CREATE/DROP DATABASE, ALTER DATABASE settings, the guarded DROP phase, ALTER TABLE DROP/CHECK CONSTRAINT lines, and extended properties. Leaves CREATE/ALTER-ADD DDL untouched. Composable: run this first (via run_mog) from any MSSQL->target converter. Operates on the raw SSMS text (brackets intact); each rule eats a whole GO batch, so it must run before GO is rewritten to ';'.

## Run

```
mog -m mssql-strip-server-noise <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [MogConvertTest]
GO
EXEC sys.sp_dropextendedproperty @name=N'MS_Description' , @level0type=N'SCHEMA',@level0name=N'dbo', @level1type=N'TABLE',@level1name=N'Widget'
GO
IF  EXISTS (SELECT * FROM sys.objects WHERE object_id = OBJECT_ID(N'[dbo].[Widget]') AND type in (N'U'))
DROP TABLE [dbo].[Widget]
GO
DROP INDEX [IX_Widget_Name] ON [dbo].[Widget]
GO
ALTER TABLE [dbo].[Widget] DROP CONSTRAINT [DF_Widget_Active]
GO
/****** Object:  Database [MogConvertTest]    Script Date: 1/2/2024 3:04:05 PM ******/
DROP DATABASE [MogConvertTest]
GO
CREATE DATABASE [MogConvertTest]
 CONTAINMENT = NONE
 ON  PRIMARY
( NAME = N'MogConvertTest', FILENAME = N'C:\data\MogConvertTest.mdf' )
```

_(... 37 more line(s))_

Output:

```
USE [MogConvertTest]
GO
CREATE TABLE [dbo].[Widget](
	[WidgetID] [int] IDENTITY(1,1) NOT NULL,
	[Name] [nvarchar](100) NOT NULL,
	[IsActive] [bit] NOT NULL,
 CONSTRAINT [PK_Widget] PRIMARY KEY CLUSTERED
(
	[WidgetID] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, IGNORE_DUP_KEY = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON, OPTIMIZE_FOR_SEQUENTIAL_KEY = OFF) ON [PRIMARY]
) ON [PRIMARY]
GO
CREATE NONCLUSTERED INDEX [IX_Widget_Name] ON [dbo].[Widget]
(
	[Name] ASC
)WITH (PAD_INDEX = OFF) ON [PRIMARY]
GO
ALTER TABLE [dbo].[Widget]  WITH CHECK ADD  CONSTRAINT [CK_Widget_Name] CHECK  ((len([Name])>(0)))
```

_(... 1 more line(s))_

## Steps

- `eol_lf`: Normalize to LF so every GO-batch anchor below is reliable.
- `replace_regex`: Drop SSMS object banner comment lines: /****** Object: ... Script Date: ... ******/.
- `replace_regex`: Remove SET ANSI_NULLS batches.
- `replace_regex`: Remove SET QUOTED_IDENTIFIER batches.
- `replace_regex`: Remove SET ANSI_PADDING batches (SSMS emits these around indexes).
- `replace_regex`: Remove the CREATE DATABASE batch (no Postgres equivalent; you create/connect separately).
- `replace_regex`: Remove every ALTER DATABASE ... SET ... batch (compat level, ANSI options, query store, filestream, ...).
- `replace_regex`: Remove EXEC sys.sp_db_vardecimal_storage_format.
- `replace_regex`: Remove guarded drops: IF EXISTS (SELECT ... sys.objects ...) DROP <object>. SSMS emits 'IF  EXISTS' with variable spacing.
- `replace_regex`: Remove plain DROP <object> batches (database/table/view/proc/function/sequence/synonym/schema/index).
- `replace_regex`: Remove ALTER TABLE ... DROP CONSTRAINT batches (drop-phase constraint teardown).
- `replace_regex`: Remove SSMS constraint re-enable lines: ALTER TABLE x CHECK CONSTRAINT y (no Postgres equivalent). Note the single space around CHECK, so this never matches 'WITH CHECK ADD CONSTRAINT'.
- `replace_regex`: Remove extended-property calls (sp_addextendedproperty / sp_dropextendedproperty).

## Tags

`fragment` `sql` `mssql` `cleanup` `strip`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
