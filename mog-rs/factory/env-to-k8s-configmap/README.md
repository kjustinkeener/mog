# .env to k8s ConfigMap

Convert .env to a Kubernetes ConfigMap

Turn a .env file into a Kubernetes ConfigMap manifest (YAML), with the variables under data. Set the name with -D name=... (default app-config).

## Run

```
mog -m env-to-k8s-configmap <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
NAME=App
PORT=8080
```

Output:

```
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  NAME: App
  PORT: '8080'
```

## Pipeline

- `env_to_json`: Parse the .env into a JSON object.
- `json_wrap`: Nest the vars under data.
- `json_merge`: Add the ConfigMap envelope.
- `json_to_yaml`: Render as YAML.

## Tags

`convert` `env` `dotenv` `k8s` `yaml`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
