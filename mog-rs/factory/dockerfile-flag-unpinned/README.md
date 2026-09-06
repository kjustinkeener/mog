# Flag unpinned Dockerfile dependencies

Mark unpinned base images and package installs in a Dockerfile

Audit a Dockerfile for supply-chain drift: every FROM whose base image is not pinned by an immutable @sha256: digest is marked, as is every package step that installs whatever version happens to be current (apt-get/apk without a pinned version, pip without ==, npm install rather than npm ci) and every RUN that pipes a downloaded script straight into a shell. Nothing is rewritten: each finding gets one inline # marker naming the risk, so a reviewer or an agent can act on it. Comment lines are skipped, and each line gets exactly one marker (the most specific rule wins). Because the mog reads one line at a time, a FROM that refers back to an earlier build stage by its alias is reported too; ignore those markers.

## Run

```
mog -m dockerfile-flag-unpinned <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Build stage: pinned by digest, nothing to report here
FROM golang:1.22-alpine@sha256:0b7f0e4b0b3c8d0a5f1e2d3c4b5a69788a9b0c1d2e3f405162738495a6b7c8d9 AS builder
WORKDIR /src
RUN apk add --no-cache git
RUN apk add --no-cache git=2.43.0-r0
COPY . .
RUN go build -o /out/app ./cmd/app

# FROM ubuntu:22.04 -- an old base kept as a note, not built
FROM ubuntu
RUN apt-get update && apt-get install -y curl ca-certificates
RUN apt-get install -y jq=1.6-2.1ubuntu3
RUN curl -sSL https://example.test/install.sh | sh
RUN curl -sSL https://example.test/tool.tar.gz -o /tmp/tool.tar.gz

FROM node:20
RUN npm install
RUN npm ci
```

_(... 5 more line(s))_

Output:

```
# Build stage: pinned by digest, nothing to report here
FROM golang:1.22-alpine@sha256:0b7f0e4b0b3c8d0a5f1e2d3c4b5a69788a9b0c1d2e3f405162738495a6b7c8d9 AS builder
WORKDIR /src
RUN apk add --no-cache git # WARN(mog): apk package version is left open; write name=version
RUN apk add --no-cache git=2.43.0-r0
COPY . .
RUN go build -o /out/app ./cmd/app

# FROM ubuntu:22.04 -- an old base kept as a note, not built
FROM ubuntu # FIXME(mog): base image carries no tag and no digest, so every build can pick up a different one
RUN apt-get update && apt-get install -y curl ca-certificates # WARN(mog): apt package version is left open; write name=version
RUN apt-get install -y jq=1.6-2.1ubuntu3
RUN curl -sSL https://example.test/install.sh | sh # FIXME(mog): a downloaded script runs unchecked; fetch it, verify a checksum, then run it
RUN curl -sSL https://example.test/tool.tar.gz -o /tmp/tool.tar.gz

FROM node:20 # FIXME(mog): base image is not pinned by digest; add an @sha256 reference
RUN npm install # WARN(mog): npm resolves fresh here; npm ci installs exactly what the lockfile records
RUN npm ci
```

_(... 5 more line(s))_

## Pipeline

- `flag_matching`: FROM with neither a tag nor a digest
- `flag_matching`: FROM not pinned to an immutable digest
- `flag_matching`: A fetched script piped straight into a shell
- `flag_matching`: apt-get install without a version
- `flag_matching`: apk add without a version
- `flag_matching`: pip install without a pinned version or a requirements file
- `flag_matching`: npm install rather than a lockfile install

## Tags

`docker` `audit` `security` `supply-chain` `review`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
