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
RUN pip install requests # WARN(mog): python package version is left open; write name==version or use a requirements file
RUN pip install requests==2.31.0
RUN pip install -r requirements.txt
COPY --from=builder /out/app /usr/local/bin/app
ENTRYPOINT ["app"]
