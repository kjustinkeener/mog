# Sort IP addresses

Sort IP addresses numerically, IPv4 before IPv6

Sort a list of IP addresses by numeric value (not lexically, so 10.0.0.2 comes before 10.0.0.10), IPv4 before IPv6. The IP is the first token of each line, so plain lists and 'IP rest-of-line' access logs both sort correctly; lines whose first token is not an IP keep their order at the end.

## Run

```
mog -m sort-ip-addresses <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
10.0.0.10
192.168.1.5
10.0.0.2
2001:db8::1
10.0.0.1
unparsed line
192.168.1.20
```

Output:

```
10.0.0.1
10.0.0.2
10.0.0.10
192.168.1.5
192.168.1.20
2001:db8::1
unparsed line
```

## Steps

- `sort_ip`: Order lines by IP address value

## Tags

`sort` `network` `log` `list`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
