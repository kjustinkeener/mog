# Split a full-name column

Split a full-name column into first/middle/last/suffix

Split the last column of a CSV, a full name, into four columns: first, middle, last, suffix. Both the First Last order and the quoted Last, First order are read, a leading title (Mr Mrs Ms Miss Dr Prof) is dropped, a generational suffix (Jr Sr II III IV) is moved to its own column, and a surname that starts with a particle (van, von, de, della and friends) is kept whole in the last column. A three-word name puts the middle word in the middle column; a longer one puts everything between the first and last words there. A name that cannot be separated confidently (a single word) is kept in the first column and flagged for review rather than guessed at. The header cell is assumed to be full_name; change the first and last steps for a different header.

## Run

```
mog -m split-name-column <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,full_name
1,John Smith
2,"Smith, John"
3,Mary Jane Watson
4,Dr. Alan Grant
5,Johannes van der Berg
6,Robert Downey Jr.
7,Martin Luther King Sr.
8,Cher
9,"O'Neill, Tip"
10,Jean-Luc Picard
11,Ms. Ada King Lovelace
12,Ludwig van Beethoven III
```

Output:

```
id,first,middle,last,suffix
1,John,,Smith,
2,John,,Smith,
3,Mary,Jane,Watson,
4,Alan,,Grant,
5,Johannes,,van der Berg,
6,Robert,,Downey,Jr
7,Martin,Luther,King,Sr
8,Cher,,, # FIXME(mog): one word only so the parts could not be separated; kept whole for review
9,Tip,,O'Neill,
10,Jean-Luc,,Picard,
11,Ada,King,Lovelace,
12,Ludwig,,van Beethoven,III
```

## Steps

- `replace`: Park the header out of the way
- `replace_regex`: Quoted Last, First order becomes First Last
- `replace_regex`: Drop a leading title
- `replace_regex`: Split off a generational suffix
- `replace_regex`: Mark the names that carry no suffix
- `replace_regex`: A surname starting with a particle stays whole
- `replace_regex`: Three or more words become first middle last
- `replace_regex`: Two words become first and last
- `flag_matching`: Flag the names that could not be separated
- `replace_regex`: Pad an unsplit name out to four columns
- `replace`: Markers become column separators
- `replace`: Restore the header as four column names

## Tags

`csv` `names` `split` `table` `data` `cleanup`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
