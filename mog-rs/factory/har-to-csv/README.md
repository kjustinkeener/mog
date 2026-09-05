# HAR to CSV

Convert a browser HAR export to a CSV request table

Turn a browser HAR network export (the JSON a devtools Network tab saves) into a CSV request table -- one row per entry, with the columns method,url,status,mime_type,transfer_size,time_ms: the fields you actually triage a slow or failing page load with. The HAR is read with the JSON actions, not scraped with regex: the log.entries array is lifted out, expanded to one object per line, and each entry is projected through a template. transfer_size prefers the Chrome response._transferSize (bytes on the wire) and falls back to response.bodySize; time_ms is the HAR's own entry time in milliseconds, left at its original precision. Values are emitted as RFC 4180 CSV, so URLs holding commas or quotes are quoted and internal quotes are doubled. Sort or filter the result with any CSV tool -- by status to find failures, by time_ms to find the slow requests. Scope: one HAR log per file; request/response bodies, headers, cookies and the timings breakdown are not exported (add ${timings.wait} and friends to the template step if you want them).

## Run

```
mog -m har-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "log": {
    "version": "1.2",
    "creator": {
      "name": "WebInspector",
      "version": "537.36"
    },
    "pages": [
      {
        "startedDateTime": "2026-08-14T09:12:03.100Z",
        "id": "page_1",
        "title": "https://shop.example.com/catalog",
        "pageTimings": {
          "onContentLoad": 412.3,
          "onLoad": 981.7
        }
      }
    ],
```

_(... 334 more line(s))_

Output:

```
method,url,status,mime_type,transfer_size,time_ms
GET,https://shop.example.com/catalog,200,text/html,18422,512.774
GET,https://cdn.example.com/app.js?v=4.2.1,200,application/javascript,91055,143.201
GET,"https://api.example.com/v1/search?q=shoes,boots&sort=price,asc&page=2",200,application/json,3120,88.5
POST,https://api.example.com/v1/cart/items,201,application/json,410,210.049
GET,https://cdn.example.com/img/hero.png,304,image/png,118,12.003
GET,https://tracker.example.net/px?id=99&t=1755162723,404,text/plain,260,61.44
```

## Pipeline

- `json_minify`: Parse the HAR and re-serialize it on a single line.
- `replace_regex`: Drop everything ahead of the log.entries array.
- `extract_json`: Take the balanced entries array and discard the rest of the log.
- `json_to_jsonl`: Expand the array to one request object per line.
- `json_extract`: Emit method, URL, status, MIME type, transfer size and time for each request.
- `replace`: Double every embedded double quote.
- `replace_regex`: Wrap any field holding a comma or a quote in double quotes.
- `replace`: Swap each unit separator for a comma.
- `prepend`: Add the CSV header row.

## Tags

`har` `json` `csv` `convert` `web` `observability`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
