# SARIF to CSV

Convert a SARIF scanner report to a CSV of findings

Turn a SARIF 2.1.0 static-analysis report (CodeQL, Semgrep, Trivy, ESLint, Bandit, gitleaks ...) into a CSV finding table -- one row per result, with the columns rule_id,level,message,file,line: the fields you actually triage a scanner run with. The report is read with the JSON actions, not scraped with regex: the runs array is lifted out, each run is expanded to one line, every runs[].results[] entry is collected as its own JSON line, and each finding is projected through a template -- so a report holding several runs (several tools in one file) is handled, not just the first. file and line come from the finding's FIRST physical location; a finding carrying more than one location is flagged so you know the table is showing only part of it. Escaped newlines and tabs in a message are folded to spaces to keep each finding on one row, and the result is RFC 4180 CSV, so a message holding a comma or a quote is quoted and internal quotes are doubled. Scope: level is taken as written on the result -- SARIF lets a result inherit its level from the rule's defaultConfiguration, and that inheritance is NOT resolved here, so a blank level means the result did not carry one (kind is used as a fallback). Rule metadata (name, help text, tags), fixes, code flows, related locations, suppressions and the artifacts table are not exported.

## Run

```
mog -m sarif-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "Semgrep",
          "semanticVersion": "1.86.0",
          "rules": [
            {
              "id": "python.lang.security.audit.exec-detected"
            },
            {
              "id": "generic.secrets.hardcoded-token"
            }
          ]
        }
```

_(... 111 more line(s))_

Output:

```
rule_id,level,message,file,line
python.lang.security.audit.exec-detected,error,"Detected the use of exec(), which is dangerous. Audit the input, or remove the call.",src/loader.py,42
# WARN(mog): finding carries more than one location; only the first is in this table
generic.secrets.hardcoded-token,warning,"Hardcoded token assigned to ""API_KEY""; move it to the environment",config/settings.py,7
generic.style.no-todo,informational,TODO left in shipped code,"src/ui/app,v2.tsx",118
aws-access-token,,aws-access-token has detected a secret.,infra/terraform/main.tf,231
```

## Pipeline

- `json_minify`: Parse the report and re-serialize it on a single line.
- `replace_regex`: Drop everything ahead of the runs array.
- `extract_json`: Take the balanced runs array and discard the rest of the report.
- `json_to_jsonl`: Expand the array to one run per line.
- `json_extract`: Collect every result of every run as its own JSON line.
- `replace_regex`: Fold escaped newlines and tabs in a message so each finding stays on one row.
- `json_extract`: Emit rule, level, message, file and line for each finding, plus a second-location probe.
- `flag_matching`: Flag a finding that carries more than one location.
- `replace_regex_multiline`: Drop the second-location probe column.
- `replace`: Double every embedded double quote.
- `replace_regex`: Wrap any field holding a comma or a quote in double quotes.
- `replace`: Swap each unit separator for a comma.
- `prepend`: Add the CSV header row.

## Tags

`sarif` `json` `csv` `convert` `security` `ci`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
