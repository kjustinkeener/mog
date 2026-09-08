# normalize-log-for-diff

Mask volatile log values (timestamps, UUIDs, hashes) for diffing

Replace volatile values in a log (ISO timestamps, UUIDs, long hex hashes, 0x addresses) with stable <TS>/<UUID>/<HASH>/<ADDR> placeholders, so two runs diff down to the real differences. Deterministic and reversible-in-meaning.

## Run

```
mog -m normalize-log-for-diff <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
2026-08-23T20:41:05.123Z INFO request 3fa85f64-5717-4562-b3fc-2c963f66afa6 done
sha256=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
allocated at 0x7ffee3b12a4c
plain line stays
```

Output:

```
<TS> INFO request <UUID> done
sha256=<HASH>
allocated at <ADDR>
plain line stays
```

## Steps

- `replace_regex`: ISO 8601 timestamps
- `replace_regex`: UUIDs
- `replace_regex`: long hex hashes (md5/sha1/sha256)
- `replace_regex`: hex addresses

## Tags

`log` `diff` `scan` `config` `normalize`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
