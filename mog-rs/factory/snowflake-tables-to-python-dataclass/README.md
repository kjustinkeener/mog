# Snowflake tables to Python dataclasses

Generate Python @dataclass classes from a Snowflake GET_DDL export

Generate Python standard-library @dataclass classes from a Snowflake table export (SELECT GET_DDL('TABLE', ...) or a Snowsight DDL copy), one class per table. Strips the Snowflake-only rendering (statement terminators, inline COMMENT 'x', the glued )COMMENT='x', cluster by, autoincrement, DEFAULT and table-level constraint lines). Snowflake identifiers are UPPER_SNAKE_CASE: the class name is PascalCased and field names are lowercased to snake_case. Snowflake types map to Python types (NUMBER(p,0)/INT/BIGINT -> int, NUMBER(p,s) s>0 -> Decimal, FLOAT/DOUBLE/REAL -> float, BOOLEAN -> bool, VARCHAR/STRING/TEXT/CHAR -> str, DATE -> date, TIME -> time, TIMESTAMP_NTZ/TIMESTAMP_TZ/TIMESTAMP_LTZ -> datetime, BINARY/VARBINARY -> bytes, VARIANT/OBJECT -> dict[str, Any], ARRAY -> list[Any]). Nullable columns become 'type | None'. Emits a fixed stdlib import header with 'from __future__ import annotations' so the module runs on Python 3.7+. Lossy mappings are flagged with a # TODO(mog): comment: semi-structured columns and NUMBER(p,s) fixed-point columns the connector may hand back as float. Assumes LF.

## Run

```
mog -m snowflake-tables-to-python-dataclass <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
create or replace TABLE SUBSCRIBER (
	SUBSCRIBER_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	FIRST_NAME VARCHAR(64) NOT NULL,
	LAST_NAME VARCHAR(64) NOT NULL,
	UPDATED_AT TIMESTAMP_NTZ(9) NOT NULL,
	primary key (SUBSCRIBER_ID)
);

create or replace TABLE CAMPAIGN_RUN (
	CAMPAIGN_RUN_ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	ACCOUNT_CODE VARCHAR(32) NOT NULL,
	EMAIL VARCHAR(255) COMMENT 'primary contact email',
	IS_ACTIVE BOOLEAN NOT NULL DEFAULT TRUE,
	SIGNED_UP_ON DATE NOT NULL,
	NOTES STRING,
	BALANCE NUMBER(18,2) NOT NULL,
	LAST_SEEN_AT TIMESTAMP_LTZ(9)
)COMMENT='account master table'
```

_(... 20 more line(s))_

Output:

```
from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from typing import Any

@dataclass
class Subscriber:
    subscriber_id: int
    first_name: str
    last_name: str
    updated_at: datetime

@dataclass
class CampaignRun:
    campaign_run_id: int
    account_code: str
    email: str | None
```

_(... 24 more line(s))_

## Steps

- `run_mog`: Remove the Snowflake GET_DDL rendering: statement terminators, inline column COMMENT 'x', the glued table )COMMENT='x', and the cluster by clause.
- `replace_regex`: Drop the SQL comment lines the strip fragment leaves behind (the CLUSTER BY note).
- `replace_regex`: Drop table-level constraint lines (primary key, foreign key, unique, constraint).
- `replace_regex`: Collapse the blank lines the removals left behind.
- `replace_regex`: Convert the tab indent GET_DDL emits to four spaces, so column lines have a fixed shape.
- `replace_regex`: Rewrite each 'create or replace TABLE <tbl> (' header.
- `replace_regex`: Drop each table-closing ')' entirely (Python has no closing brace).
- `replace_regex`: Drop the Snowflake autoincrement / identity clause.
- `replace_regex`: Drop COLLATE clauses.
- `replace_regex`: Drop DEFAULT clauses (sequences, CURRENT_TIMESTAMP(), literals).
- `replace_regex`: Drop trailing commas that separated columns.
- `replace_regex`: Mark nullable columns (any column line NOT ending in NOT NULL) with a sentinel.
- `replace_regex`: Remove the NOT NULL constraint marker from non-nullable columns.
- `flag_matching`: Flag semi-structured VARIANT / OBJECT columns: mapped to dict.
- `flag_matching`: Flag ARRAY columns: Snowflake arrays are untyped.
- `flag_matching`: Flag fixed-point columns (NUMBER(p,s) with s > 0): mapped to Decimal, not float.
- `replace_regex`: TIMESTAMP_NTZ / DATETIME (wall-clock, no zone) -> datetime.
- `replace_regex`: TIMESTAMP_TZ / TIMESTAMP_LTZ (zone-aware) -> datetime.
- `replace_regex`: Bare TIMESTAMP (the session default, normally NTZ) -> datetime.
- `replace_regex`: TIME -> time.
- `replace_regex`: DATE -> date.
- `replace_regex`: NUMBER(p,s) / DECIMAL(p,s) with s > 0 -> Decimal (via sentinel).
- `replace_regex`: NUMBER(p,0) with p <= 9 -> int (via sentinel).
- `replace_regex`: NUMBER(p,0) with p between 10 and 18 -> int (via sentinel).
- `replace_regex`: NUMBER(p,0) with p >= 19, including Snowflake's NUMBER(38,0) default -> int (via sentinel).
- `replace_regex`: NUMBER(p) with no scale -> int (via sentinel).
- `replace_regex`: Bare NUMBER / DECIMAL / NUMERIC (defaults to NUMBER(38,0)) -> int (via sentinel).
- `replace_regex`: BIGINT / INT / INTEGER (all aliases of NUMBER(38,0) in Snowflake) -> int (via sentinel).
- `replace_regex`: SMALLINT / TINYINT / BYTEINT (also NUMBER(38,0) in Snowflake) -> int (via sentinel).
- `replace_regex`: FLOAT / FLOAT4 / FLOAT8 / DOUBLE PRECISION / DOUBLE / REAL -> float.
- `replace_regex`: BOOLEAN -> bool.
- `replace_regex`: VARCHAR(n) / CHAR(n) / CHARACTER(n) -> str.
- `replace_regex`: Bare VARCHAR / STRING / TEXT / CHAR -> str.
- `replace_regex`: VARIANT -> dict (flagged above).
- `replace_regex`: OBJECT -> dict (flagged above).
- `replace_regex`: ARRAY -> list (flagged above).
- `replace_regex`: GEOGRAPHY / GEOMETRY -> str (carry the GeoJSON/WKT text through).
- `replace_regex`: BINARY(n) / VARBINARY -> bytes (last, so it cannot be re-matched by an earlier rule).
- `replace_regex`: Resolve the fixed-point sentinel to Decimal.
- `replace_regex`: Resolve the small-integer sentinel to int.
- `replace_regex`: Resolve the mid-integer sentinel to int.
- `replace_regex`: Resolve the wide-integer sentinel to int.
- `replace_regex`: Rewrite nullable columns into a dataclass field with a 'type | None' annotation.
- `replace_regex`: Rewrite non-nullable columns into a dataclass field with a plain annotation.
- `replace_regex`: Safety: strip any leftover nullability sentinel.
- `to_pascal`: PascalCase each class name (the Snowflake table identifier).
- `to_lower`: snake_case each field name; Snowflake returns UPPER_SNAKE_CASE identifiers.
- `replace_regex`: Collapse multiple blank lines to a single blank between classes.
- `replace_regex`: Trim leading blank lines.
- `replace_regex`: Trim trailing blank lines to a single final newline.
- `replace_regex`: Widen the bare dict annotation to dict[str, Any] so the output passes a strict type checker. Runs after nullability, and is anchored to the annotation so it never touches the word 'dict' inside a TODO comment.
- `replace_regex`: Widen the bare list annotation to list[Any] so the output passes a strict type checker. The lookahead leaves an already-parameterised list[T] alone, and the anchor keeps the word 'list' in a TODO comment safe.
- `replace_regex`: Prepend the fixed stdlib import header (last, so the type-map regexes never touch 'date'/'time'/'Decimal' in the imports).

## Tags

`sql` `snowflake` `python` `codegen` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
