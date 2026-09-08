# Strip REPL and shell prompts

Remove REPL and shell prompts from pasted sessions

Remove interactive prompts from pasted sessions so you are left with just the code/commands: Python's >>> and ... continuation, a leading $ shell prompt, and Jupyter's In [n]: / Out[n]: markers. Handy for turning a copied REPL transcript into a runnable script. Lines without a prompt are left alone.

## Run

```
mog -m strip-repl-prompts <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
>>> x = 1
>>> print(x)
... still going
$ ls -la
In [1]: df.head()
regular line stays
```

Output:

```
x = 1
print(x)
still going
ls -la
df.head()
regular line stays
```

## Steps

- `replace_regex_multiline`: Remove a leading prompt marker

## Tags

`paste` `jupyter` `python` `shell` `cleanup`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
