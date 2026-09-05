# Canonicalize SQL identifier case

Fold SQL identifier case to a dialect's storage rule

Normalize unquoted identifier case in a SQL statement to the target dialect's storage rule, so the same query written with inconsistent casing collapses to one canonical form. 'dialect' is required and decides the direction: snowflake folds unquoted identifiers to UPPER CASE, postgres folds them to lower case. Quoted identifiers are left as written (they are case-sensitive). Set 'quote':true to additionally wrap every identifier in the dialect's quote character. Parse-based (polyglot-sql): it rewrites identifiers structurally, not by find/replace, so keywords, string literals, and aliases are handled correctly. The whole input is treated as one statement.

## Run

```
mog -m sql-canonicalize-identifiers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
select id, Name, Total_Amount from MyOrders where Status = 'Paid' order by Order_Date
```

Output:

```
SELECT ID, NAME, TOTAL_AMOUNT FROM MYORDERS WHERE STATUS = 'Paid' ORDER BY ORDER_DATE;
```

## Pipeline

- `sql_canonicalize_identifiers`: Fold unquoted identifiers to Snowflake's upper-case storage form

## Tags

`sql` `snowflake` `normalize` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
