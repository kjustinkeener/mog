# Redact MAC addresses

Redact MAC addresses to [MAC]

Replace every MAC address (six colon-separated hex pairs, like 00:1A:2B:3C:4D:5E) with [MAC] so a network log can be shared without exposing hardware addresses. Case-insensitive. Hyphen-separated MACs are not matched.

## Run

```
mog -m redact-mac-addresses <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Device 00:1A:2B:3C:4D:5E joined the network.
Gateway aa:bb:cc:dd:ee:ff seen. No MAC here.
```

Output:

```
Device [MAC] joined the network.
Gateway [MAC] seen. No MAC here.
```

## Steps

- `replace_regex`: Replace colon-separated MAC addresses with a placeholder

## Tags

`redact` `eol` `network` `privacy` `security` `log`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
