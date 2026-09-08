# Bump deprecated Kubernetes apiVersions

Bump deprecated Kubernetes apiVersions to stable

Rewrite the common removed/deprecated Kubernetes apiVersion values to their current stable ones (extensions/v1beta1 and apps/v1beta1|v1beta2 -> apps/v1, networking/rbac/batch v1beta1 -> v1). Any remaining v1beta1 / v1alpha1 apiVersion is flagged with a review marker rather than guessed, since those may need a schema change. Check the result against your cluster version.

## Run

```
mog -m k8s-apiversion-bump <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apiVersion: extensions/v1beta1
kind: Deployment
---
apiVersion: networking.k8s.io/v1beta1
kind: Ingress
---
apiVersion: custom.io/v1alpha1
kind: Widget
```

Output:

```
apiVersion: apps/v1
kind: Deployment
---
apiVersion: networking.k8s.io/v1
kind: Ingress
---
apiVersion: custom.io/v1alpha1 # FIXME(mog): deprecated apiVersion, review manually
kind: Widget
```

## Steps

- `replace_map`: Rewrite deprecated apiVersions to current stable ones
- `flag_matching`: Flag any remaining beta/alpha apiVersion for manual review

## Tags

`k8s` `yaml` `config` `migration` `codemod`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
