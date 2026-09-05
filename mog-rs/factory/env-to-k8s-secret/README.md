# .env to k8s Secret

Convert .env to a base64 Kubernetes Secret

Turn a .env file into a Kubernetes Secret manifest (YAML) with each value base64-encoded under data, as a Secret requires (the ConfigMap sibling keeps values plain). Set the name with -D name=... (default app-secret). Comments (# lines) and blank lines are dropped; optional `export ` and surrounding quotes are stripped from each value; the first = splits key from value so values that themselves contain = are encoded whole.

## Run

```
mog -m env-to-k8s-secret <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# app config
NAME=App

export PORT=8080
URL="postgres://u:p@h/db?x=1"
TOKEN=a=b=c
QUOTED='hello world'
```

Output:

```
apiVersion: v1
kind: Secret
metadata:
  name: app-secret
type: Opaque
data:
  NAME: QXBw
  PORT: ODA4MA==
  URL: cG9zdGdyZXM6Ly91OnBAaC9kYj94PTE=
  TOKEN: YT1iPWM=
  QUOTED: aGVsbG8gd29ybGQ=
```

## Pipeline

- `remove_lines_matching`: Drop comment lines (# ...).
- `remove_empty_lines`: Drop blank lines.
- `replace_regex_multiline`: Normalize to KEY<tab>VALUE: strip optional `export ` and whitespace around the first =.
- `replace_regex_multiline`: Strip a matching pair of surrounding double or single quotes from the value.
- `replace_regex_multiline`: Strip a matching pair of surrounding single quotes from the value.
- `base64_encode`: Encode only the value field (right of the tab); keys stay plain.
- `replace_regex_multiline`: Indent each KEY<tab>VALUE two spaces under data as KEY: VALUE.
- `prepend`: Add the Secret envelope above the data block.

## Tags

`convert` `env` `dotenv` `k8s` `secrets` `base64` `yaml`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
