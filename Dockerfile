# Build mog and ship it in a small image for CI use. Recipes are embedded into
# the binary at build time (include_dir!), so the runtime image is just the
# executable plus a libc.
#
# NOTE: this Dockerfile has been authored but NOT yet smoke-tested in a real
# build. Validate with `docker build -t mog .` before relying on it (see
# packaging/README.md). If mog's HTTP store features pull in a native-TLS
# backend, the runtime stage may also need `ca-certificates`/`libssl`.
FROM rust:1-slim AS build
WORKDIR /src
COPY mog-rs ./mog-rs
RUN cargo build --release --manifest-path mog-rs/Cargo.toml --bin mog

FROM debian:stable-slim
COPY --from=build /src/mog-rs/target/release/mog /usr/local/bin/mog
WORKDIR /work
ENTRYPOINT ["mog"]
