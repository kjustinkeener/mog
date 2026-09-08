# npm package-lock to dependency inventory CSV

npm package-lock.json to a CSV dependency inventory

Turn an npm package-lock.json (lockfileVersion 3) into a CSV dependency inventory: name,version,resolved_host,dev,integrity_present, one row per entry under the top-level `packages` object (the root "" entry is skipped). `name` comes from the package path with the leading `node_modules/` stripped, keeping scoped names like @scope/pkg intact and using the LAST `node_modules/` segment for nested transitive paths. `dev` is true/false from the entry's dev flag (absent means false). `resolved_host` is the hostname of the `resolved` URL, empty when `resolved` is absent or is not a URL (workspace links). IMPORTANT: an npm lockfile does NOT carry licence text or licence identifiers, so no licence column is produced and none is invented. This is the dependency inventory you feed to a licence checker (license-checker, oss-review-toolkit, Syft), not a licence report. Entry splitting keys on the `node_modules/` path prefix that only top-level `packages` keys carry, so it is specific to lockfileVersion 2/3 `packages` maps and not to the legacy v1 `dependencies` tree.

## Run

```
mog -m package-lock-to-inventory <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "name": "example-app",
  "version": "1.4.2",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {
    "": {
      "name": "example-app",
      "version": "1.4.2",
      "license": "MIT",
      "dependencies": {
        "@babel/core": "^7.24.0",
        "lodash": "^4.17.21"
      },
      "devDependencies": {
        "typescript": "^5.4.0"
      }
    },
```

_(... 48 more line(s))_

Output:

```
name,version,resolved_host,dev,integrity_present
@babel/core,7.24.7,registry.npmjs.org,false,true
lodash,4.17.21,registry.npmjs.org,false,true
semver,6.3.1,registry.yarnpkg.com,false,true
typescript,5.4.5,registry.npmjs.org,true,true
shared-utils,,,false,false
left-pad,1.3.0,,true,false
tar-fs,2.1.1,codeload.github.com,false,true
```

## Steps

- `json_minify`: Parse the whole lockfile as one JSON value and re-serialize it compactly. This validates the JSON and gives the later steps a canonical, whitespace-free form.
- `json_extract`: Pull out the top-level `packages` map as raw JSON, discarding the lockfile header (name, version, lockfileVersion, requires). A v1 lockfile with no `packages` map yields nothing here.
- `replace_regex`: Split the map into one line per package entry. Only top-level `packages` keys start with `node_modules/`, so that prefix is a safe entry boundary (nested `dependencies` maps are keyed by bare package names).
- `remove_lines_matching`: Drop the root project entry. It is the only chunk still carrying the map's opening brace, because it is the first key in the map.
- `replace_regex`: Drop the map's closing brace from the last entry, so every line is a self-contained object.
- `replace_regex_multiline`: Lift each entry's package path into the object as `__path__`, turning `"path":{...}` into a standalone JSON object. The result is valid JSONL that the JSON reader can walk.
- `replace_regex`: Repair an entry that had no fields at all, where the lifted path would otherwise leave a dangling comma.
- `json_extract`: Read the five source values out of each entry as a tab-separated row: path, version, resolved, dev, integrity. Absent keys leave the cell empty.
- `replace_regex`: Path to package name: strip everything through the LAST `node_modules/`, which keeps a scoped @scope/pkg intact and names a nested transitive dependency by the package itself.
- `replace_regex`: Blank a `resolved` value that is not a URL (a workspace link resolves to a relative directory, which has no host).
- `replace_regex`: Reduce the `resolved` URL to its hostname.
- `replace_regex_multiline`: Integrity present: a non-empty integrity cell becomes true. Run before the empty case so the literal `true` written here is not re-matched.
- `replace_regex_multiline`: Integrity absent: an empty integrity cell becomes false.
- `replace_regex_multiline`: Dev flag absent: an empty dev cell becomes false. Present means the lockfile wrote `dev: true`, which the JSON reader already rendered as `true`.
- `change_delimiter`: Re-delimit the tab-separated rows as RFC 4180 CSV, quoting any cell that needs it.
- `prepend`: Add the header row.
- `replace_regex`: Ensure the CSV ends with a newline (added only when one is missing).

## Tags

`convert` `json` `csv` `npm` `dependencies` `inventory`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
