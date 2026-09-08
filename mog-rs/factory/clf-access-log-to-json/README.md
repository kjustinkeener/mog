# Access log (CLF) to JSON

Convert Common/Combined access logs to JSON

Parse Common/Combined Log Format access-log lines into JSON, one object per line (JSONL), with fields ip, timestamp, method, path, protocol, status, size, referer, user_agent. Lines that do not match the log format are skipped.

## Run

```
mog -m clf-access-log-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
127.0.0.1 - - [10/Oct/2023:13:55:36 +0000] "GET /index.html HTTP/1.1" 200 2326
10.0.0.5 - - [10/Oct/2023:13:56:01 +0000] "POST /api/login HTTP/1.1" 401 512
```

Output:

```
{"ip":"127.0.0.1","timestamp":"10/Oct/2023:13:55:36 +0000","method":"GET","path":"/index.html","protocol":"HTTP/1.1","status":"200","size":"2326","referer":"","user_agent":""}
{"ip":"10.0.0.5","timestamp":"10/Oct/2023:13:56:01 +0000","method":"POST","path":"/api/login","protocol":"HTTP/1.1","status":"401","size":"512","referer":"","user_agent":""}
```

## Steps

- `access_log_to_csv`: Parse the log lines into a CSV table
- `csv_to_json`: Turn the CSV rows into a JSON array of objects

## Tags

`log` `json` `convert` `observability`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
