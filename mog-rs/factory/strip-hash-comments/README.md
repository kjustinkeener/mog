# Strip # comment lines

Remove whole-line # comments and collapse blank runs

Remove whole-line # comments from a shell script, config, YAML, or Dockerfile, then collapse the blank runs the removals leave behind. Only lines whose first non-whitespace is # are dropped; inline trailing comments and # inside values are left alone (removing those safely needs awareness of quoting).

## Run

```
mog -m strip-hash-comments <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# set up the environment
export PATH=/usr/local/bin:$PATH

# start the server
run_server --port 8080
echo "done"
```

Output:

```
export PATH=/usr/local/bin:$PATH

run_server --port 8080
echo "done"
```

## Pipeline

- `remove_lines_matching`: Drop full-line # comments
- `squeeze_blank_lines`: Collapse the blank runs left behind

## Tags

`config` `shell` `comment` `strip` `cleanup`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
