# Kubernetes manifests to CSV

Convert a multi-document Kubernetes YAML file to a CSV resource inventory

Turn a multi-document Kubernetes YAML file (a `kubectl get -o yaml` dump, a helm template render, a kustomize build) into a CSV inventory -- one row per resource, with the columns kind,name,namespace,api_version. The stream is read with the real YAML parser rather than scraped with regex: each `---` separator becomes a sequence entry, so the whole file parses as one YAML list, and each document is then projected through a JSON template. That means quoted keys, block scalars, anchors and flow mappings are all handled by the parser, and a `---` inside a string value cannot be mistaken for a separator. A `...` end-of-document marker is dropped, and a document that is empty or holds only comments contributes no row. A missing namespace is rendered as the EMPTY STRING, not as `default`: the manifest genuinely does not say, because the resource is either cluster-scoped or takes its namespace from the apply command, and inventing `default` would be wrong for both. A document with no kind, or with a kind but no metadata.name, is flagged rather than dropped. Values are emitted as RFC 4180 CSV, so a name or namespace holding a comma or a quote is quoted and internal quotes are doubled. Scope: one YAML stream per file, and only the four identity fields are exported -- labels, annotations, spec, status and the containers/images are not. Anchors are expanded by the parser (so a merged document reports its resolved kind), but Helm/Kustomize templating is NOT: a file still holding {{ }} template directives is not valid YAML and will fail the parse; render it first.

## Run

```
mog -m k8s-manifests-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Rendered by: helm template shop ./charts/shop
apiVersion: v1
kind: Namespace
metadata:
  name: shop-prod
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: shop-config
  namespace: shop-prod
  labels:
    app.kubernetes.io/name: shop
data:
  MOTD: |
    Welcome to the shop.
    ---
    Not a document separator.
```

_(... 46 more line(s))_

Output:

```
kind,name,namespace,api_version
Namespace,shop-prod,,v1
ConfigMap,shop-config,shop-prod,v1
Deployment,shop-web,shop-prod,apps/v1
ClusterRole,shop-reader,,rbac.authorization.k8s.io/v1
Service,"shop-web, external",shop-prod,v1
# WARN(mog): resource has a kind but no metadata.name
Secret,,shop-prod,v1
# WARN(mog): document has no kind field; it may be a values or kustomization file
,,,
```

## Pipeline

- `remove_lines_matching`: Drop the ... end-of-document markers.
- `prepend`: Give the first document an explicit separator.
- `replace_regex`: Normalize every separator line (a trailing comment or directive is dropped).
- `replace_regex`: Collapse the injected separator against one the file already had.
- `indent`: Indent every document so it can sit under a list entry.
- `replace_regex`: Turn each separator into a YAML list entry.
- `yaml_to_json`: Parse the whole stream as one YAML list.
- `json_to_jsonl`: Expand the list to one resource per line.
- `remove_lines_matching`: Drop the empty and comment-only documents.
- `json_extract`: Emit kind, name, namespace and apiVersion for each document.
- `flag_matching`: Flag a document that carries no kind.
- `flag_matching`: Flag a resource that carries a kind but no name.
- `replace`: Double every embedded double quote.
- `replace_regex`: Wrap any field holding a comma or a quote in double quotes.
- `replace`: Swap each unit separator for a comma.
- `prepend`: Add the CSV header row.

## Tags

`kubernetes` `yaml` `csv` `convert` `devops` `inventory`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
