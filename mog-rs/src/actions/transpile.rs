//! True semantic SQL dialect transpilation via the pure-Rust `polyglot-sql` crate
//! (a from-scratch port of sqlglot). This is the correctness tier beside the regex
//! converters: it PARSES and REGENERATES the SQL, so it handles semantics the regex
//! mogs cannot (function/type/quoting rules across dialects) but REFORMATS the
//! output (not minimal-diff, comments may move). In the default build.
//!
//! Also hosts `sql_lint`: parse-validate SQL against a dialect and annotate problems
//! in place (the read/validate side of the same parser).

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use polyglot_sql::{
    AnalyzeQueryOptions, DataType, Dialect, DialectType, MappingSchema, Schema, ValidationOptions,
    ValidationSeverity,
};

use crate::model::Step;

/// Resolve a REQUIRED, named dialect option to a `DialectType`. Unlike
/// [`dialect_type_or_generic`], both absence and an unknown name are errors: the
/// actions that use this (type spelling, identifier casing) are dialect-specific and
/// the dialect-agnostic `Generic` grammar would give a meaningless answer.
fn require_dialect(step: &Step, key: &str, action: &str) -> Result<DialectType> {
    let n = step
        .get_string(key)
        .ok_or_else(|| anyhow!("{action} requires a '{key}' dialect"))?;
    Dialect::get_by_name(&n)
        .map(|d| d.dialect_type())
        .ok_or_else(|| anyhow!("{action}: unknown dialect '{n}'"))
}

/// Resolve an optional dialect-name option to a `DialectType`, defaulting to the
/// dialect-agnostic `Generic` grammar when the option is absent. An explicit but
/// unknown name is an error (fail loud, don't silently fall back to Generic).
fn dialect_type_or_generic(name: Option<String>) -> Result<DialectType> {
    match name {
        Some(n) => Dialect::get_by_name(&n)
            .map(|d| d.dialect_type())
            .ok_or_else(|| anyhow!("unknown dialect '{n}'")),
        None => Ok(DialectType::default()),
    }
}

/// `sql_transpile`: transpile the whole input from the `from` SQL dialect to the
/// `to` dialect (e.g. `from: "snowflake"`, `to: "bigquery"`). Dialect names are the
/// lowercase identifiers polyglot-sql accepts (snowflake, bigquery, redshift,
/// postgres, databricks, spark, ...). Reformats the SQL.
pub fn sql_transpile(input: &str, step: &Step) -> Result<String> {
    let from = step
        .get_string("from")
        .ok_or_else(|| anyhow!("sql_transpile requires a 'from' dialect"))?;
    let to = step
        .get_string("to")
        .ok_or_else(|| anyhow!("sql_transpile requires a 'to' dialect"))?;
    let statements = polyglot_sql::transpile_by_name(input, &from, &to)
        .map_err(|e| anyhow!("sql_transpile ({from} -> {to}): {e}"))?;
    // polyglot-sql returns one regenerated statement per input statement, without a
    // trailing semicolon; re-join them into a script.
    if statements.is_empty() {
        return Ok(String::new());
    }
    Ok(format!("{};", statements.join(";\n")))
}

/// `sql_lint`: parse-validate the whole input as SQL against a target `dialect` and
/// annotate problems in place as `-- LINT ...` comment lines above the offending
/// statement. Reports syntax errors (E-codes, with line/column) and, when `semantic`
/// is on (default), query-quality warnings (W001 SELECT *, W002 aggregate without
/// GROUP BY, W003 DISTINCT+ORDER BY, W004 LIMIT without ORDER BY). Clean input is
/// returned unchanged, so a `mog --check` run exits non-zero exactly when there are
/// findings. The natural tail of a transpile recipe: lint the OUTPUT against the
/// target dialect to surface whatever the transpiler + fixups could not fully carry.
pub fn sql_lint(input: &str, step: &Step) -> Result<String> {
    let dialect = step
        .get_string("dialect")
        .ok_or_else(|| anyhow!("sql_lint requires a 'dialect'"))?;
    let d = Dialect::get_by_name(&dialect)
        .ok_or_else(|| anyhow!("sql_lint: unknown dialect '{dialect}'"))?;
    let opts = ValidationOptions {
        strict_syntax: step.get_bool("strict", false)?,
        semantic: step.get_bool("semantic", true)?,
    };
    let result = polyglot_sql::validate_with_dialect(input, &d, &opts);
    if result.errors.is_empty() {
        // Clean: leave the SQL untouched so --check reports no change.
        return Ok(input.to_string());
    }
    let prefix = step
        .get_string("note_prefix")
        .unwrap_or_else(|| "-- LINT".to_string());

    // Group findings by 1-based line; findings with no line sort to the top (key 0).
    let mut by_line: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for e in &result.errors {
        let sev = match e.severity {
            ValidationSeverity::Error => "error",
            ValidationSeverity::Warning => "warning",
        };
        let loc = match (e.line, e.column) {
            (Some(l), Some(c)) => format!(" (line {l}, col {c})"),
            (Some(l), None) => format!(" (line {l})"),
            _ => String::new(),
        };
        let note = format!("{prefix} {} {sev}{loc}: {}", e.code, e.message);
        by_line.entry(e.line.unwrap_or(0)).or_default().push(note);
    }

    let lines: Vec<&str> = input.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + result.errors.len());
    if let Some(top) = by_line.get(&0) {
        out.extend(top.iter().cloned());
    }
    for (i, line) in lines.iter().enumerate() {
        if let Some(notes) = by_line.get(&(i + 1)) {
            // Indent each note to match the offending line so it reads inline.
            let indent: String = line
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect();
            out.extend(notes.iter().map(|n| format!("{indent}{n}")));
        }
        out.push((*line).to_string());
    }
    Ok(out.join("\n"))
}

/// `sql_format`: pretty-print (canonically re-format) the whole input as SQL in the
/// given `dialect`, using polyglot-sql's generator. Reflows whitespace, casing, and
/// layout to the dialect's canonical style -- a formatter, not a minimal diff. Unlike
/// `sql_transpile` it stays in ONE dialect (no semantic translation), so use it to
/// normalize hand-written SQL before diffing or committing.
pub fn sql_format(input: &str, step: &Step) -> Result<String> {
    let dialect = step
        .get_string("dialect")
        .ok_or_else(|| anyhow!("sql_format requires a 'dialect'"))?;
    let statements = polyglot_sql::format_by_name(input, &dialect)
        .map_err(|e| anyhow!("sql_format ({dialect}): {e}"))?;
    // One formatted string per statement; re-join into a script (mirrors sql_transpile).
    if statements.is_empty() {
        return Ok(String::new());
    }
    Ok(format!("{};", statements.join(";\n")))
}

/// `sql_tables`: parse the input as SQL and REPLACE it with the list of physical source
/// tables it reads (one per line), resolved through CTEs, subqueries, and set operations.
/// `dialect` is optional (defaults to the dialect-agnostic grammar); by default names are
/// fully qualified (`schema.table` as written in the SQL), set `qualified: false` for the
/// bare table name. The list is deduped and sorted. A read/analysis action -- the answer
/// to "what does this query depend on?" without a warehouse connection.
pub fn sql_tables(input: &str, step: &Step) -> Result<String> {
    let dialect = dialect_type_or_generic(step.get_string("dialect"))?;
    let qualified = step.get_bool("qualified", true)?;
    let analysis = polyglot_sql::analyze_query(
        input,
        AnalyzeQueryOptions {
            dialect,
            schema: None,
        },
    )
    .map_err(|e| anyhow!("sql_tables: {e}"))?;
    // base_tables is already deduped + sorted by qualified name; when reducing to bare
    // names a second dedup collapses same-named tables from different schemas.
    let mut names: Vec<String> = analysis
        .base_tables
        .iter()
        .map(|t| {
            if qualified {
                t.name.clone()
            } else {
                t.table.clone().unwrap_or_else(|| t.name.clone())
            }
        })
        .collect();
    names.dedup();
    Ok(names.join("\n"))
}

/// `sql_canonicalize_identifiers`: parse the input as SQL and REGENERATE it with all
/// identifiers normalized to the target `dialect`'s canonical case (e.g. Snowflake
/// upper-cases unquoted identifiers, Postgres lower-cases them). Optionally `quote`
/// every identifier so the exact spelling survives a round-trip into another dialect.
/// A same-dialect rewrite (like `sql_format` but touching only identifier spelling,
/// not layout); use it to make hand-written SQL consistent before diffing or transpiling.
pub fn sql_canonicalize_identifiers(input: &str, step: &Step) -> Result<String> {
    let dialect = require_dialect(step, "dialect", "sql_canonicalize_identifiers")?;
    let quote = step.get_bool("quote", false)?;
    let statements = polyglot_sql::parse(input, dialect)
        .map_err(|e| anyhow!("sql_canonicalize_identifiers: {e}"))?;
    if statements.is_empty() {
        return Ok(String::new());
    }
    let mut out: Vec<String> = Vec::with_capacity(statements.len());
    for expr in statements {
        let mut e = polyglot_sql::optimizer::normalize_identifiers(expr, Some(dialect));
        if quote {
            e = polyglot_sql::optimizer::quote_identifiers(e, Some(dialect));
        }
        out.push(
            polyglot_sql::generate(&e, dialect)
                .map_err(|e| anyhow!("sql_canonicalize_identifiers: {e}"))?,
        );
    }
    Ok(format!("{};", out.join(";\n")))
}

/// `sql_datatype_convert`: treat EACH non-blank line as a single SQL data type in the
/// `from` dialect and REWRITE it as the equivalent type in the `to` dialect (e.g.
/// Oracle `VARCHAR2(50)` -> Postgres `VARCHAR(50)`, Snowflake `NUMBER(38,0)` ->
/// BigQuery `NUMERIC(38, 0)`). Blank lines pass through. Per line, so it stays linear:
/// the tool for building a type-mapping table or converting a column-type list, without
/// wrapping the types in a full statement for `sql_transpile`. Both dialects are required.
pub fn sql_datatype_convert(input: &str, step: &Step) -> Result<String> {
    let from = require_dialect(step, "from", "sql_datatype_convert")?;
    let to = require_dialect(step, "to", "sql_datatype_convert")?;
    let mut out: Vec<String> = Vec::new();
    for line in input.split('\n') {
        if line.trim().is_empty() {
            out.push(line.to_string());
            continue;
        }
        let dt = polyglot_sql::parse_data_type(line.trim(), from).map_err(|e| {
            anyhow!(
                "sql_datatype_convert: cannot parse type '{}': {e}",
                line.trim()
            )
        })?;
        let s = polyglot_sql::generate_data_type(&dt, to)
            .map_err(|e| anyhow!("sql_datatype_convert: {e}"))?;
        out.push(s);
    }
    Ok(out.join("\n"))
}

/// `sql_lineage`: parse the input as SQL and REPLACE it with a per-output-column lineage
/// listing (`out_col <- table.src_col, ...`, one per line), resolving each SELECT
/// projection back to the source column(s) it derives from through CTEs and subqueries.
/// A column with no source (a literal/constant, or a `*` that needs a schema to expand)
/// shows `<- (none)`. `dialect` is optional (defaults to the dialect-agnostic grammar).
/// The column-level companion to `sql_tables`: "where does each output column come from?"
/// with no warehouse connection.
pub fn sql_lineage(input: &str, step: &Step) -> Result<String> {
    let dialect = dialect_type_or_generic(step.get_string("dialect"))?;
    let analysis = polyglot_sql::analyze_query(
        input,
        AnalyzeQueryOptions {
            dialect,
            schema: None,
        },
    )
    .map_err(|e| anyhow!("sql_lineage: {e}"))?;
    let mut out: Vec<String> = Vec::with_capacity(analysis.projections.len());
    for p in &analysis.projections {
        let out_name = p
            .name
            .clone()
            .unwrap_or_else(|| format!("col{}", p.index + 1));
        let srcs: Vec<String> = p
            .upstream
            .iter()
            .map(|u| match &u.table {
                Some(t) => format!("{t}.{}", u.column),
                None => u.column.clone(),
            })
            .collect();
        let rhs = if srcs.is_empty() {
            "(none)".to_string()
        } else {
            srcs.join(", ")
        };
        out.push(format!("{out_name} <- {rhs}"));
    }
    Ok(out.join("\n"))
}

/// Build a `MappingSchema` from a loaded `ref` source whose lines are CSV
/// `table,column,type` (an optional header row `table,column,type` is skipped). Each
/// type is parsed in `dialect`. This is how `sql_qualify` gets the schema it needs to
/// resolve unqualified columns to their tables.
fn schema_from_ref(refl: &[String], dialect: DialectType) -> Result<MappingSchema> {
    // table -> ordered columns, preserving first-seen order.
    let mut tables: Vec<(String, Vec<(String, DataType)>)> = Vec::new();
    for (i, raw) in refl.iter().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, ',').map(|s| s.trim()).collect();
        if parts.len() < 3 {
            return Err(anyhow!(
                "sql_qualify schema line {} must be 'table,column,type': got '{line}'",
                i + 1
            ));
        }
        // Skip a header row.
        if i == 0
            && parts[0].eq_ignore_ascii_case("table")
            && parts[1].eq_ignore_ascii_case("column")
        {
            continue;
        }
        let (table, column, ty) = (parts[0], parts[1], parts[2]);
        let dt = polyglot_sql::parse_data_type(ty, dialect)
            .map_err(|e| anyhow!("sql_qualify: bad type '{ty}' for {table}.{column}: {e}"))?;
        match tables.iter_mut().find(|(t, _)| t == table) {
            Some((_, cols)) => cols.push((column.to_string(), dt)),
            None => tables.push((table.to_string(), vec![(column.to_string(), dt)])),
        }
    }
    let mut schema = MappingSchema::new();
    for (table, cols) in &tables {
        schema
            .add_table(table, cols, Some(dialect))
            .map_err(|e| anyhow!("sql_qualify: schema build failed for '{table}': {e}"))?;
    }
    Ok(schema)
}

/// `sql_qualify`: parse the input as SQL and REGENERATE it with every column reference
/// qualified by its table (`col` -> `table.col`), resolved against a schema supplied as a
/// named `ref` source of `table,column,type` rows. With `expand_stars` on, `SELECT *` is
/// expanded to the explicit column list. Schema-aware -- the one SQL action that needs to
/// know the table shapes -- so it disambiguates columns a schema-free parse cannot. Reformats.
pub fn sql_qualify(
    input: &str,
    step: &Step,
    sources: &std::collections::BTreeMap<String, Vec<String>>,
) -> Result<String> {
    let ref_name = step
        .get_string("ref")
        .ok_or_else(|| anyhow!("sql_qualify requires a 'ref' option (a loaded schema source)"))?;
    let refl = sources.get(&ref_name).ok_or_else(|| {
        anyhow!(
            "sql_qualify: source '{ref_name}' is not loaded \
             (bind it with --source {ref_name}=<path> or a 'sources' entry)"
        )
    })?;
    let dialect = dialect_type_or_generic(step.get_string("dialect"))?;
    let schema = schema_from_ref(refl, dialect)?;
    let opts = polyglot_sql::optimizer::QualifyColumnsOptions {
        expand_alias_refs: step.get_bool("expand_alias_refs", false)?,
        expand_stars: step.get_bool("expand_stars", false)?,
        dialect: Some(dialect),
        ..Default::default()
    };
    let statements =
        polyglot_sql::parse(input, dialect).map_err(|e| anyhow!("sql_qualify: {e}"))?;
    if statements.is_empty() {
        return Ok(String::new());
    }
    let mut out: Vec<String> = Vec::with_capacity(statements.len());
    for expr in statements {
        let q = polyglot_sql::optimizer::qualify_columns(expr, &schema, &opts)
            .map_err(|e| anyhow!("sql_qualify: {e}"))?;
        out.push(polyglot_sql::generate(&q, dialect).map_err(|e| anyhow!("sql_qualify: {e}"))?);
    }
    Ok(format!("{};", out.join(";\n")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn step(opts: serde_json::Value) -> Step {
        Step {
            description: None,
            section: None,
            action: None,
            disabled: false,
            only_lines_matching: None,
            except_lines_matching: None,
            match_ignore_case: false,
            scope: None,
            options: opts.as_object().cloned().unwrap_or_default(),
        }
    }

    #[test]
    fn transpiles_snowflake_to_bigquery() {
        let st = step(json!({"from": "snowflake", "to": "bigquery"}));
        let out = sql_transpile("SELECT IFF(a > 0, 1, 0) FROM t", &st).unwrap();
        // BigQuery uses IF, not IFF; polyglot-sql should rewrite it.
        assert!(out.to_uppercase().contains("IF("), "got: {out}");
        assert!(!out.to_uppercase().contains("IFF("), "got: {out}");
    }

    #[test]
    fn missing_dialect_errors() {
        let st = step(json!({"from": "snowflake"}));
        assert!(sql_transpile("SELECT 1", &st).is_err());
    }

    #[test]
    fn lint_clean_sql_is_unchanged() {
        let st = step(json!({"dialect": "postgres", "semantic": false}));
        let sql = "SELECT a FROM t WHERE a > 1";
        assert_eq!(sql_lint(sql, &st).unwrap(), sql);
    }

    #[test]
    fn lint_syntax_error_is_annotated() {
        let st = step(json!({"dialect": "bigquery"}));
        // Missing table after FROM -> a parse error with a location.
        let out = sql_lint("SELECT FROM WHERE x", &st).unwrap();
        assert!(out.contains("-- LINT"), "got: {out}");
        assert!(out.to_uppercase().contains("SELECT FROM"), "got: {out}");
    }

    #[test]
    fn lint_semantic_warning_flags_select_star() {
        let st = step(json!({"dialect": "postgres", "semantic": true}));
        let out = sql_lint("SELECT * FROM t", &st).unwrap();
        // W001 warns on SELECT *; the original line is preserved.
        assert!(out.contains("-- LINT"), "got: {out}");
        assert!(
            out.contains("W001") || out.to_lowercase().contains("warning"),
            "got: {out}"
        );
    }

    #[test]
    fn lint_unknown_dialect_errors() {
        let st = step(json!({"dialect": "nope"}));
        assert!(sql_lint("SELECT 1", &st).is_err());
    }

    #[test]
    fn format_pretty_prints_sql() {
        let st = step(json!({"dialect": "postgres"}));
        let out = sql_format("select a,b from t where a>1", &st).unwrap();
        // Canonical formatting upper-cases keywords and reflows onto multiple lines.
        assert!(out.to_uppercase().contains("SELECT"), "got: {out}");
        assert!(out.contains('\n'), "got: {out}");
        assert!(out.trim_end().ends_with(';'), "got: {out}");
    }

    #[test]
    fn format_requires_dialect() {
        let st = step(json!({}));
        assert!(sql_format("SELECT 1", &st).is_err());
    }

    #[test]
    fn tables_lists_qualified_source_tables() {
        let st = step(json!({"dialect": "postgres"}));
        let out = sql_tables(
            "SELECT * FROM sales.orders o JOIN dim.customers c ON o.cid = c.id",
            &st,
        )
        .unwrap();
        assert!(out.contains("sales.orders"), "got: {out}");
        assert!(out.contains("dim.customers"), "got: {out}");
    }

    #[test]
    fn tables_bare_names_when_unqualified() {
        let st = step(json!({"dialect": "postgres", "qualified": false}));
        let out = sql_tables("SELECT * FROM sales.orders", &st).unwrap();
        assert_eq!(out, "orders");
    }

    #[test]
    fn tables_resolves_through_cte() {
        let st = step(json!({}));
        let out = sql_tables("WITH x AS (SELECT * FROM base_tbl) SELECT * FROM x", &st).unwrap();
        // The CTE name x is not a physical table; base_tbl is.
        assert!(out.contains("base_tbl"), "got: {out}");
        assert!(!out.split('\n').any(|l| l == "x"), "got: {out}");
    }

    #[test]
    fn canonicalize_uppercases_for_snowflake() {
        let st = step(json!({"dialect": "snowflake"}));
        let out = sql_canonicalize_identifiers("select id, name from users", &st).unwrap();
        assert!(out.contains("ID"), "got: {out}");
        assert!(out.contains("USERS"), "got: {out}");
    }

    #[test]
    fn canonicalize_requires_dialect() {
        let st = step(json!({}));
        assert!(sql_canonicalize_identifiers("SELECT 1", &st).is_err());
    }

    #[test]
    fn datatype_convert_maps_per_line_and_keeps_blanks() {
        let st = step(json!({"from": "oracle", "to": "postgres"}));
        let out = sql_datatype_convert("VARCHAR2(50)\n\nNUMBER(38,0)", &st).unwrap();
        let lines: Vec<&str> = out.split('\n').collect();
        assert_eq!(lines[0], "VARCHAR(50)", "got: {out}");
        assert_eq!(lines[1], "", "blank line preserved; got: {out}");
        assert!(lines[2].to_uppercase().contains("(38, 0)"), "got: {out}");
    }

    #[test]
    fn datatype_convert_requires_both_dialects() {
        let st = step(json!({"from": "oracle"}));
        assert!(sql_datatype_convert("DATE", &st).is_err());
    }

    #[test]
    fn lineage_maps_output_columns_to_sources() {
        let st = step(json!({"dialect": "postgres"}));
        let out = sql_lineage(
            "select o.id as oid, c.name, 1 as flag from orders o join cust c on o.cid=c.id",
            &st,
        )
        .unwrap();
        assert!(out.contains("oid <- orders.id"), "got: {out}");
        assert!(out.contains("name <- cust.name"), "got: {out}");
        // A constant projection has no source column.
        assert!(out.contains("flag <- (none)"), "got: {out}");
    }

    fn sources(pairs: &[(&str, &str)]) -> std::collections::BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, v)| {
                (
                    (*k).to_string(),
                    v.split('\n').map(str::to_string).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn qualify_qualifies_columns_against_schema() {
        let st = step(json!({"ref": "schema", "dialect": "postgres"}));
        let src = sources(&[(
            "schema",
            "orders,oid,int\norders,cid,int\ncust,id,int\ncust,name,text",
        )]);
        let out = sql_qualify(
            "select oid, name from orders join cust on cid = cust.id",
            &st,
            &src,
        )
        .unwrap();
        assert!(out.contains("orders.oid"), "got: {out}");
        assert!(out.contains("cust.name"), "got: {out}");
        assert!(out.contains("orders.cid"), "got: {out}");
    }

    #[test]
    fn qualify_expands_stars() {
        let st = step(json!({"ref": "schema", "dialect": "postgres", "expand_stars": true}));
        let src = sources(&[("schema", "cust,id,int\ncust,name,text")]);
        let out = sql_qualify("select * from cust", &st, &src).unwrap();
        assert!(out.contains("cust.id"), "got: {out}");
        assert!(out.contains("cust.name"), "got: {out}");
    }

    #[test]
    fn qualify_errors_when_source_missing() {
        let st = step(json!({"ref": "nope"}));
        let src = sources(&[("schema", "cust,id,int")]);
        assert!(sql_qualify("select id from cust", &st, &src).is_err());
    }

    #[test]
    fn qualify_header_row_is_skipped() {
        let st = step(json!({"ref": "schema", "dialect": "postgres", "expand_stars": true}));
        let src = sources(&[("schema", "table,column,type\ncust,id,int\ncust,name,text")]);
        let out = sql_qualify("select * from cust", &st, &src).unwrap();
        assert!(out.contains("cust.id"), "got: {out}");
        assert!(
            !out.to_lowercase().contains("column"),
            "header leaked: {out}"
        );
    }
}
