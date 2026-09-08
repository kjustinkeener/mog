# SQL Server views to PostgreSQL

Convert a real SSMS-scripted SQL Server view to PostgreSQL

Convert a SQL Server SSMS-scripted view (T-SQL CREATE VIEW) to PostgreSQL. Removes the SSMS scaffolding (USE banner, SET ANSI_NULLS/QUOTED_IDENTIFIER), strips square-bracket quoting and the dbo. prefix, drops WITH SCHEMABINDING / VIEW_METADATA, moves a leading TOP (n) to a trailing LIMIT n, and rewrites SELECT-body functions (GETDATE()->now(), GETUTCDATE()/SYSUTCDATETIME()->utc now(), SYSDATETIME()->current_timestamp, ISNULL->COALESCE, LEN->LENGTH) plus common cast type names. Flags constructs it cannot safely rewrite as -- TODO(mog): CROSS/OUTER APPLY, PIVOT/UNPIVOT, part-aware DATEDIFF, TOP ... PERCENT / WITH TIES, '+' string concatenation, CONVERT(...) casts, and schema-qualified scalar UDF calls. Not a SQL parser. Set schema_prefix (or --define schema_prefix=...) to control schema qualification.

## Run

```
mog -m sqlserver-views-to-postgres <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [lab]
GO
/****** Object:  View [dbo].[v_viewdemo]    Script Date: Sat 8 29 2026  2:09:17 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE VIEW [dbo].[v_viewdemo]
WITH SCHEMABINDING
AS
SELECT TOP 100
    [order_id],
    [customer_name],
    ISNULL([region], 'UNKNOWN') AS region_clean,
    [quantity] * [unit_price] AS gross_amount,
    [quantity] * [unit_price] * (1 - ISNULL([discount], 0)) AS net_amount,
    LEN([customer_name]) AS name_len,
    GETDATE() AS snapshot_at
```

_(... 4 more line(s))_

Output:

```
-- Database: lab

CREATE VIEW v_viewdemo
AS
SELECT
    order_id,
    customer_name,
    COALESCE(region, 'UNKNOWN') AS region_clean,
    quantity * unit_price AS gross_amount,
    quantity * unit_price * (1 - COALESCE(discount, 0)) AS net_amount,
    LENGTH(customer_name) AS name_len,
    now() AS snapshot_at
FROM viewdemo_orders
WHERE status_code = 1 AND quantity > 0
ORDER BY order_date DESC
LIMIT 100;
```

## Steps

- `eol_lf`: Normalize to LF first so every later CRLF-sensitive regex (GO-batch anchors) is reliable. Output stays LF (standard for SQL scripts; psql reads it fine).
- `run_mog`: Strip SQL Server server/session scaffolding first (object banner comment, SET ANSI_NULLS/QUOTED_IDENTIFIER batches, any CREATE/DROP DATABASE or DROP phase). Runs on the raw SSMS text while GO batches are intact. Reusable across object-kind converters.
- `flag_matching`: Flag schema-qualified scalar UDF calls ([dbo].[fn_x]( ... )) BEFORE the bracket/dbo strip, while the schema is still visible. A trailing '(' distinguishes a function call from a plain table reference in FROM. Postgres has no dbo schema and the function must be ported separately.
- `flag_matching`: Flag CROSS APPLY / OUTER APPLY: no direct Postgres equivalent (rewrite as a LATERAL join).
- `flag_matching`: Flag PIVOT / UNPIVOT: not supported in Postgres (rewrite with conditional aggregation, or the tablefunc crosstab).
- `flag_matching`: Flag part-aware DATEDIFF: Postgres has no DATEDIFF (for day diffs use end::date - start::date, else EXTRACT / AGE).
- `flag_matching`: Flag CONVERT(...) casts: the optional style argument is not portable; rewrite as CAST(expr AS type) (and to_char/to_date for styled datetime conversions).
- `flag_matching`: Flag TOP ... PERCENT / WITH TIES: cannot be moved to a plain LIMIT (needs a window function / percentage of rows).
- `flag_matching`: Flag '+' string concatenation (a string literal adjacent to '+'): Postgres uses || and treats NULL differently.
- `replace`: Strip T-SQL square-bracket quoting: opening bracket.
- `replace`: Strip T-SQL square-bracket quoting: closing bracket.
- `replace`: Schema qualifier: rewrite the SQL Server 'dbo.' prefix. Default schema_prefix is empty, so objects become unqualified and resolve via search_path (public) -- the idiomatic Postgres result. Set schema_prefix to 'public.' or keep 'dbo.' as needed.
- `replace_regex`: USE <db> + its GO -> a documenting comment (Postgres has no USE; you connect to a database instead).
- `replace_regex`: Drop the view option clause 'WITH SCHEMABINDING' / 'WITH VIEW_METADATA' (and combinations): Postgres has no per-view equivalent. Removes the whole clause line so CREATE VIEW <name> flows straight into AS.
- `replace_regex`: Move a leading TOP (n) / TOP n on the view's SELECT to a trailing LIMIT n (Postgres has no TOP). Non-greedy to the batch-terminating GO; the negative lookahead leaves TOP ... PERCENT / WITH TIES for the flag above. Runs while GO is still the terminator.
- `replace`: Function map: getutcdate() -> now() in UTC (run before getdate()).
- `replace`: Function map: sysutcdatetime() -> now() in UTC.
- `replace`: Function map: sysdatetime() -> current_timestamp.
- `replace`: Function map: getdate() -> now().
- `replace_regex`: Function map: ISNULL(a, b) -> COALESCE(a, b) (whole-word so it never bites into another identifier).
- `replace_regex`: Function map: LEN(x) -> LENGTH(x). Note: LEN ignores trailing spaces in T-SQL while LENGTH counts them; identical for the common case.
- `replace`: Type name in a CAST: nvarchar(max) -> text (run before varchar(max), which it contains).
- `replace`: Type name in a CAST: varchar(max) -> text.
- `replace`: Type name in a CAST: nvarchar -> varchar (Postgres varchar is already Unicode).
- `replace_regex`: Type name in a CAST: datetime2(n) -> timestamp (run before bare datetime).
- `replace`: Type name in a CAST: datetime -> timestamp (whole word).
- `replace`: Type name in a CAST: uniqueidentifier -> uuid.
- `replace`: Type name in a CAST: bit -> boolean (whole word).
- `replace_regex`: Turn the view's terminating GO batch separator into a statement terminator, plus a blank line after.
- `run_mog`: Canonical whitespace finalize via the shared fragment (tidy_chars picks the punctuation to de-space).

## Tags

`sql` `mssql` `postgres` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
