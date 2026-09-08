# Strip Oracle DBMS_METADATA.GET_DDL noise

Remove Oracle GET_DDL quoting/schema/ENABLE/USING INDEX boilerplate

Clean the output of Oracle DBMS_METADATA.GET_DDL (with storage/tablespace transforms already suppressed), so a downstream dialect converter sees a clean CREATE TABLE. Trims the leading whitespace, drops the schema qualifier and the double-quote identifier quoting, removes the ENABLE keyword and the CONSTRAINT USING INDEX clause, and normalizes 'TIMESTAMP (n)' spacing. Leaves the NUMBER/VARCHAR2/CLOB/DATE types for the target converter. Target-agnostic; run it first. Assumes LF.

## Run

```
mog -m strip-oracle-getddl-noise <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```

  CREATE TABLE "APP"."CUSTOMERS"
   (	"ID" NUMBER(38,0) NOT NULL ENABLE,
	"NAME" VARCHAR2(200),
	"EMAIL" VARCHAR2(255),
	"DESCRIPTION" CLOB,
	"BALANCE" NUMBER(18,2),
	"IS_ACTIVE" NUMBER(1,0),
	"CREATED_AT" DATE,
	"UPDATED_AT" TIMESTAMP (6),
	 CONSTRAINT "PK_CUSTOMERS" PRIMARY KEY ("ID")
  USING INDEX  ENABLE
   )
```

Output:

```
CREATE TABLE CUSTOMERS
   (	ID NUMBER(38,0) NOT NULL,
	NAME VARCHAR2(200),
	EMAIL VARCHAR2(255),
	DESCRIPTION CLOB,
	BALANCE NUMBER(18,2),
	IS_ACTIVE NUMBER(1,0),
	CREATED_AT DATE,
	UPDATED_AT TIMESTAMP(6),
	 CONSTRAINT PK_CUSTOMERS PRIMARY KEY (ID)
  )
```

## Steps

- `eol_lf`
- `replace_regex`: Trim leading whitespace/newlines.
- `replace_regex`: Drop the "SCHEMA". qualifier.
- `replace`: Unquote identifiers.
- `replace`: Remove ENABLE keyword.
- `replace_regex`: Remove the CONSTRAINT USING INDEX clause.
- `replace`: Normalize TIMESTAMP (n) spacing.
- `replace_regex`: Collapse blank lines.

## Tags

`sql` `oracle` `mssql` `strip` `migration` `fragment`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
