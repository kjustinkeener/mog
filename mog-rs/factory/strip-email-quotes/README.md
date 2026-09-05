# Strip email quote markers

Remove leading > quote markers from email replies

Remove the leading > (and >>, >>>) quote-level markers that email clients add to quoted replies, leaving the underlying text. Handles one or more markers with optional spaces at the start of a line; lines with no marker are untouched.

## Run

```
mog -m strip-email-quotes <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
> On Monday, Bob wrote:
>> The original message
> a reply line
not quoted at all
```

Output:

```
On Monday, Bob wrote:
The original message
a reply line
not quoted at all
```

## Pipeline

- `replace_regex_multiline`: Remove leading > quote markers

## Tags

`email` `quote` `strip` `cleanup` `text`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
