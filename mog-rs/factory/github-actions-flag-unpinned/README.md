# Flag unpinned GitHub Actions steps

Mark GitHub Actions steps that are not pinned to a commit hash

Audit a GitHub Actions workflow for supply-chain drift: every `uses:` that is not pinned to a full 40-character commit hash is marked (a tag such as @v4 and a branch such as @main are both mutable, and a tag can be moved to point at different code), container actions without an @sha256 digest are marked, and any `run:` line that pipes a downloaded script straight into a shell is marked too. Local `uses: ./path` references are left alone because they come from the repository itself. Nothing is rewritten: each finding gets one inline # marker, and each line gets exactly one (the most specific rule wins).

## Run

```
mog -m github-actions-flag-unpinned <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name: ci
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      # uses: actions/checkout@v3 -- the old pin, kept for reference
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - uses: some-org/nightly-action@main
      - uses: ./.github/actions/local-setup
      - uses: docker://alpine:3.19
      - uses: docker://alpine@sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b
```

_(... 7 more line(s))_

Output:

```
name: ci
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      # uses: actions/checkout@v3 -- the old pin, kept for reference
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
      - uses: actions/setup-node@v4 # FIXME(mog): this step is pinned to a movable name; use a full commit hash instead
        with:
          node-version: 20
      - uses: some-org/nightly-action@main # FIXME(mog): this step follows a branch, so its code can change under you; use a full commit hash
      - uses: ./.github/actions/local-setup
      - uses: docker://alpine:3.19 # FIXME(mog): container reference has no digest; add an @sha256 reference
      - uses: docker://alpine@sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b
```

_(... 7 more line(s))_

## Steps

- `flag_matching`: uses: tracking a moving branch
- `flag_matching`: Container action without a digest
- `flag_matching`: uses: pinned to a tag rather than a commit hash
- `flag_matching`: A fetched script piped straight into a shell

## Tags

`github-actions` `yaml` `audit` `security` `supply-chain` `review`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
