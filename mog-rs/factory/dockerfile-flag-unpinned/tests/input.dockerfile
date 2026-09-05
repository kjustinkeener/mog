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
RUN pip install requests
RUN pip install requests==2.31.0
RUN pip install -r requirements.txt
COPY --from=builder /out/app /usr/local/bin/app
ENTRYPOINT ["app"]
