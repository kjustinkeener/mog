# SQL Server procs to PostgreSQL

Convert a real SSMS-scripted SQL Server stored procedure to PostgreSQL PL/pgSQL

Convert a SQL Server SSMS 'Generate Scripts' stored procedure (T-SQL) to PostgreSQL PL/pgSQL (the procs, not tables, object type). Rewrites the CREATE PROCEDURE wrapper (strips [brackets] and dbo., turns the un-parenthesized @param list into a parenthesized Postgres signature, drops the @ sigil on parameters and their body uses, converts '@p TYPE = default' to 'TYPE DEFAULT default', maps T-SQL param/column types via the shared DDL type maps, and wraps the body BEGIN...END as 'LANGUAGE plpgsql AS $$ ... END; $$;'), drops SET NOCOUNT, and translates the simple, regex-tractable body pieces (GETDATE()->NOW(), ISNULL->COALESCE, newid()->gen_random_uuid(), PRINT expr -> RAISE NOTICE '%%', expr). Everything procedural that cannot be safely converted is COMMENTED OUT and stamped with -- TODO(mog) so the result still COMPILES: T-SQL cursors, DECLARE @tbl TABLE table variables, #temp tables, BEGIN TRY/CATCH, MERGE, dynamic EXEC/sp_executesql, and @@system variables. Assumes the common SSMS shape 'CREATE PROCEDURE name @params AS BEGIN ... END'; a proc with no BEGIN/END body, parameters already in parentheses, or logic beyond the flagged set needs hand review. NOTE: string concatenation with + is left as-is (it parses in Postgres but resolves at runtime; Postgres uses ||). Not a T-SQL parser. Set schema_prefix (or --define schema_prefix=...) to control schema qualification.

## Run

```
mog -m sqlserver-procs-to-postgres <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
USE [lab]
GO
/****** Object:  StoredProcedure [dbo].[p_procdemo]    Script Date: 8/29/2026 12:00:00 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE PROCEDURE [dbo].[p_procdemo]
    @customer NVARCHAR(50),
    @amount MONEY,
    @note NVARCHAR(200) = NULL
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @cur CURSOR;

```

_(... 15 more line(s))_

Output:

```
-- Database: lab

CREATE OR REPLACE PROCEDURE p_procdemo(
    customer varchar(50),
    amount numeric(19,4),
    note varchar(200) DEFAULT NULL
)
LANGUAGE plpgsql
AS $$
BEGIN
    -- TODO(mog): T-SQL cursor not portable to PL/pgSQL; rewrite as a FOR loop or refcursor: DECLARE cur CURSOR;

    INSERT INTO procdemo_orders (customer, amount, created)
    VALUES (customer, COALESCE(amount, 0), NOW());

    UPDATE procdemo_orders
    SET amount = amount
    WHERE customer = customer;
```

_(... 8 more line(s))_

## Pipeline

- `eol_lf`: Normalize to LF first so every later CRLF-sensitive regex is reliable. Output stays LF (standard for SQL scripts; psql reads it fine).
- `run_mog`: Strip SQL Server server/session/DROP scaffolding first (object banners, SET ANSI_NULLS/QUOTED_IDENTIFIER, CREATE/DROP DATABASE, ALTER DATABASE, the DROP phase, extended properties). Runs on the raw SSMS text while GO batches are intact. Shared with the tables converter.
- `replace`: Strip T-SQL square-bracket quoting: opening bracket.
- `replace`: Strip T-SQL square-bracket quoting: closing bracket.
- `replace`: Schema qualifier: rewrite the SQL Server 'dbo.' prefix. Default schema_prefix is empty, so objects become unqualified and resolve via search_path (public). Set schema_prefix to 'public.' to force qualification.
- `replace_regex`: USE <db> + its GO -> a documenting comment (Postgres has no USE; you connect to a database instead).
- `replace_regex`: CREATE/ALTER PROC[EDURE] -> CREATE OR REPLACE PROCEDURE (idempotent redeploy, the Postgres idiom).
- `replace_regex`: T-SQL cursor declaration has no portable PL/pgSQL one-liner: comment the line out and flag it (rewrite as a FOR loop or a refcursor by hand).
- `replace_regex`: Table variable (DECLARE @t TABLE (...)) has no PL/pgSQL equivalent: flag the opening line (rewrite as a TEMP TABLE or an array).
- `replace_regex`: Temp tables (#name) are session objects with different semantics: flag any statement line that references one.
- `replace_regex`: Structured error handling (BEGIN TRY / END TRY / BEGIN CATCH / END CATCH) maps to a PL/pgSQL BEGIN ... EXCEPTION block: flag each marker line for a manual rewrite.
- `replace_regex`: MERGE, dynamic EXEC(...) / EXECUTE(...) / sp_executesql, and @@system variables have no safe regex translation: flag the line.
- `replace_regex`: Parameter default: '@p TYPE = default' -> '@p TYPE DEFAULT default'. Anchored to a line beginning with @name so body assignments (SET x = y, WHERE a = b) are never touched. Runs before the @ sigil is stripped.
- `replace_regex`: Drop the @ sigil on parameters and every body reference (Postgres uses bare names). Leaves @@ system variables (already flagged above) and commented lines readable.
- `replace_regex`: Open the Postgres parameter list: put a '(' right after the procedure name (SSMS scripts the T-SQL list without parentheses).
- `replace_regex`: Close the parameter list and open the PL/pgSQL body: the 'AS' before the body BEGIN becomes ') LANGUAGE plpgsql AS $$'. Matches the SSMS 'AS\nBEGIN' shape (a proc with no BEGIN/END body needs manual wrapping).
- `replace`: Type map: nvarchar(max) -> text (run before varchar(max), which it contains).
- `replace`: Type map: varchar(max) -> text.
- `replace`: Type map: nvarchar -> varchar (Postgres varchar is already Unicode).
- `replace`: Type map: nchar -> char.
- `replace`: Type map: ntext -> text.
- `replace_regex`: Type map: datetimeoffset(n)/bare -> timestamptz (run before datetime).
- `replace_regex`: Type map: datetime2(n)/bare -> timestamp (run before datetime).
- `replace`: Type map: smalldatetime -> timestamp (run before datetime; it contains 'datetime').
- `replace`: Type map: datetime -> timestamp (whole word).
- `replace`: Type map: smallmoney -> numeric(10,4) (run before money).
- `replace`: Type map: money -> numeric(19,4).
- `replace`: Type map: tinyint -> smallint.
- `replace`: Type map: bit -> boolean (whole word).
- `replace`: Type map: uniqueidentifier -> uuid.
- `replace`: Type map: float -> double precision.
- `replace_regex`: Drop SET NOCOUNT ON/OFF (SSMS body preamble; no Postgres equivalent).
- `replace`: Function map: GETDATE() -> NOW().
- `replace`: Function map: GETUTCDATE() -> (now() at time zone 'utc').
- `replace`: Function map: SYSDATETIME() -> NOW() (run before any bare 'datetime' concerns; whole token).
- `replace_regex`: Function map: NEWID()/NEWSEQUENTIALID() -> gen_random_uuid().
- `replace_regex`: Function map: ISNULL( -> COALESCE( (Postgres has no ISNULL scalar).
- `replace_regex`: PRINT expr -> RAISE NOTICE '%', expr (PL/pgSQL logging). String concatenation with + inside expr is left as-is (parses in Postgres; use || for real text concat).
- `replace_regex`: Terminal 'END' + its trailing GO batch -> 'END;' plus the '$$;' that closes the dollar-quoted body and the CREATE statement. ($$$$ emits a literal $$.)
- `replace_regex`: Any remaining standalone GO batch separator -> statement terminator (defensive; a single-proc script has only the terminal GO handled above).
- `run_mog`: Canonical whitespace finalize via the shared fragment (tidy_chars de-spaces before ';').

## Tags

`sql` `mssql` `postgres` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
