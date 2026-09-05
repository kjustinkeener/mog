# Access log to CSV

Convert Common/Combined access logs to CSV

Parse Common/Combined access-log lines into CSV with columns ip, timestamp, method, path, protocol, status, size, referer, user_agent. Lines that do not match are skipped.

## Run

```
mog -m access-log-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
127.0.0.1 - - [10/Oct/2023:13:55:36 +0000] "GET /index.html HTTP/1.1" 200 2326 "http://example.com" "Mozilla/5.0"
10.0.0.2 - - [10/Oct/2023:13:55:40 +0000] "POST /api HTTP/1.1" 404 12 "-" "curl/8.0"
```

Output:

```
ip,timestamp,method,path,protocol,status,size,referer,user_agent
127.0.0.1,10/Oct/2023:13:55:36 +0000,GET,/index.html,HTTP/1.1,200,2326,http://example.com,Mozilla/5.0
10.0.0.2,10/Oct/2023:13:55:40 +0000,POST,/api,HTTP/1.1,404,12,-,curl/8.0
```

## Pipeline

- `access_log_to_csv`: Parse the access log into CSV.

## Tags

`convert` `log` `csv` `observability`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
