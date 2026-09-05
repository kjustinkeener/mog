# Qualify SQL columns with a schema

Prefix each SELECT column with its table using a schema

Rewrite a SELECT so every bare column is prefixed with the table it belongs to, using a schema you supply as a reference source. Point the 'schema' source at a CSV of 'table,column,type' rows with --source schema=<path> (an optional header row is skipped). Each unqualified column is matched against that schema and rewritten to table.column; a column that exists in two joined tables is genuinely ambiguous and is reported as an error rather than guessed. With 'expand_stars' on, SELECT * is expanded to the explicit, table-qualified column list. 'dialect' selects the SQL flavor for parsing and output. Parse-based via polyglot-sql; the whole input is one statement. This makes a query unambiguous and diff-friendly before it is checked in.

## Run

```
mog -m sql-qualify-columns <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT order_id, name, total, region FROM orders JOIN customers ON cust_id = id
```

Output:

```
SELECT orders.order_id AS order_id, customers.name AS name, orders.total AS total, customers.region AS region FROM orders JOIN customers ON orders.cust_id = customers.id;
```

## Pipeline

- `sql_qualify`: Prefix each column with its owning table; expand SELECT *

## Tags

`sql` `qualify` `schema` `compare` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
