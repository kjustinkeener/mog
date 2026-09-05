# Normalize phone numbers to E.164

Rewrite a phone column to E.164 and flag the rest

Rewrite a messy phone column into E.164 (+15551234567): drop spaces, parentheses, dots, slashes and hyphens, turn a leading 00 international prefix into a plus sign, and give a bare national number the default country code. The default column is the third comma-separated field and the default country is +1 (US/Canada, so a bare number must be 10 digits, or 11 digits starting with 1); edit the scope index or the country code on the steps for another layout. Nothing is guessed: any value that does not end up as a plus sign followed by 8 to 15 digits (a trailing extension, a truncated number, letters such as 555-CALL-NOW) keeps its digits, gets no country code, and is flagged for review.

## Run

```
mog -m normalize-phone-numbers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name,email,phone
Ada Byron,ada@example.com,(555) 123-4567
Bo Diaz,bo@example.com,0044 20 7946 0958
Cy Nolan,cy@example.com,+1 555 987 6543
Di Marsh,di@example.com,1-555-222-3333
Eli Frost,eli@example.com,555.CALL.NOW
Fay Oduya,fay@example.com,12345
Gus Ilic,gus@example.com,+1 (555) 123-4567 ext 89
Hana Sato,hana@example.com,
Ivo Petrov,ivo@example.com,00 33 1 42 68 53 00
```

Output:

```
name,email,phone
Ada Byron,ada@example.com,+15551234567
Bo Diaz,bo@example.com,+442079460958
Cy Nolan,cy@example.com,+15559876543
Di Marsh,di@example.com,+15552223333
Eli Frost,eli@example.com,555CALLNOW # FIXME(mog): not a plausible E.164 value; left as written for review
Fay Oduya,fay@example.com,12345 # FIXME(mog): not a plausible E.164 value; left as written for review
Gus Ilic,gus@example.com,+15551234567ext89 # FIXME(mog): not a plausible E.164 value; left as written for review
Hana Sato,hana@example.com,
Ivo Petrov,ivo@example.com,+33142685300
```

## Pipeline

- `replace_regex`: Strip separators from the phone column
- `replace_regex`: Leading 00 international prefix to a plus sign
- `replace_regex`: Eleven-digit national number keeps its country digit
- `replace_regex`: Bare ten-digit number gets the default country code
- `flag_matching`: Flag values that are not valid E.164

## Tags

`phone` `contacts` `csv` `normalize` `data` `cleanup`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
