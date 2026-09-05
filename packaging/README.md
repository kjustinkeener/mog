# Packaging & CI adoption

One-line ways to run a `.mog` transform in a CI pipeline. mog fits the
`prettier --check` / `gofmt` shape: `--check` reports whether a transform *would*
change files (nonzero exit fails the build), and `--in-place` applies it.

> **Status:** these artifacts are authored but have **not** been smoke-tested in a
> live build/CI yet. Validate each once before relying on it. Checklist below.

## Files

| File | What it is |
| --- | --- |
| `../Dockerfile` | Multi-stage build producing a small `mog` image. Recipes are embedded in the binary. |
| `../action.yml` | A Docker-based GitHub Action wrapper (`uses: kjustinkeener/mog@v1`). |
| `gitlab-ci.example.yml` | Example GitLab CI jobs (whole-repo and changed-files). |

## GitHub Actions

```yaml
- uses: kjustinkeener/mog@v1
  with:
    args: "-m recipes/fix.mog --check ."      # fail if anything would change
```

Changed files only:

```yaml
- uses: kjustinkeener/mog@v1
  with:
    args: "-m recipes/fix.mog --check --files-from -"   # feed paths on stdin from a prior step
```

## Docker (any CI)

```bash
docker build -t mog .
docker run --rm -v "$PWD:/work" mog -m recipes/fix.mog --check .
```

## Local (no container)

```bash
git diff --name-only origin/main...HEAD | mog -m recipes/fix.mog --check --files-from -
```

## Exit codes

- `0` clean (no change / all checks passed)
- `1` a file would change, or a flag/assert fired (`--check`)
- `2` an error (bad recipe, unreadable input, ...)

## Validation checklist (do before tagging a release)

- [ ] `docker build -t mog .` succeeds; `docker run --rm mog --list-actions` prints JSON.
- [ ] If the runtime image errors on a missing shared library, add
      `ca-certificates` (and any TLS lib) to the `debian:stable-slim` stage.
- [ ] The GitHub Action runs on a throwaway repo and fails/passes as expected.
- [ ] The GitLab jobs run against a real registry image.
