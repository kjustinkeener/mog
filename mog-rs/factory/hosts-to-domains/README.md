# Hosts blocklist to domain list

Extract domains from a hosts blocklist

Extract the bare domains from /etc/hosts blocklist entries by removing the leading 0.0.0.0 (or 127.0.0.1) and its whitespace, leaving one domain per line. Comment and blank lines pass through. The inverse of domains-to-hosts-block.

## Run

```
mog -m hosts-to-domains <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# blocklist
0.0.0.0 ads.example.com
0.0.0.0 tracker.net
127.0.0.1 analytics.example.io
```

Output:

```
# blocklist
ads.example.com
tracker.net
analytics.example.io
```

## Pipeline

- `replace_regex_multiline`: Strip a leading 0.0.0.0 / 127.0.0.1 and its whitespace

## Tags

`network` `blocklist` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
