# Domain list to hosts blocklist

Convert a domain list to a hosts blocklist

Turn a plain list of domains into /etc/hosts blocklist entries by prefixing each domain with 0.0.0.0 (so the host resolves nowhere). Comment lines (#) and blank lines pass through untouched. Point it at a domain list to build an ad/tracker blocklist you can append to your hosts file.

## Run

```
mog -m domains-to-hosts-block <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# ad + tracker blocklist
ads.example.com
tracker.net

analytics.example.io
```

Output:

```
# ad + tracker blocklist
0.0.0.0 ads.example.com
0.0.0.0 tracker.net

0.0.0.0 analytics.example.io
```

## Pipeline

- `prefix_lines`: Prefix each domain line with 0.0.0.0

## Tags

`network` `blocklist` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
