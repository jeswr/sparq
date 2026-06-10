# syntax=docker/dockerfile:1
# sparq-server container image (T20).
#
# Multi-stage: a rust:slim builder compiles the release binary; the runtime stage is
# distroless (cc-debian12 — glibc + libgcc only, no shell, no package manager), so the
# final image is small (~25 MB + binary) and has a minimal attack surface. Both stages
# are Debian 12 (bookworm), so the glibc the binary links against matches at runtime.
#
# Build:    docker build -t sparq-server .
#           (pass --build-arg CARGO_FLAGS="-j 2" to cap build parallelism)
# Run:      docker run --rm -p 3030:3030 -v "$PWD/data:/data:ro" sparq-server \
#             --format turtle /data/dataset.ttl
#           (no data file => starts with an empty default graph)
# Endpoint: http://localhost:3030/sparql
#
# CI builds + pushes this to ghcr.io on version tags — see .github/workflows/release.yml.

# -------- builder --------
# Pin a toolchain >= the workspace's rust-version (1.85). Bump deliberately.
FROM rust:1.87-slim-bookworm AS builder

ARG CARGO_FLAGS=""
WORKDIR /build

# .dockerignore keeps the context to the workspace sources (no target/, bench data, .git).
COPY . .

# --locked: build exactly the committed Cargo.lock. The workspace release profile is
# fat-LTO + codegen-units=1 (the shipped-binary configuration), so this step is slow
# but produces the same binary the benchmarks measured.
RUN cargo build --release --locked -p sparq-server ${CARGO_FLAGS}

# -------- runtime --------
FROM gcr.io/distroless/cc-debian12:nonroot

LABEL org.opencontainers.image.title="sparq-server" \
      org.opencontainers.image.description="W3C SPARQL 1.1 Protocol server for the sparq RDF triplestore (dictionary-encoded, six permutation indexes, parallel execution)" \
      org.opencontainers.image.source="https://github.com/jeswr/sparq" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.documentation="https://github.com/jeswr/sparq#readme"

COPY --from=builder /build/target/release/sparq-server /usr/local/bin/sparq-server

# Conventional mount point for read-only datasets.
VOLUME ["/data"]
EXPOSE 3030

# Bind 0.0.0.0 inside the container (the binary's default 127.0.0.1 is unreachable through
# Docker's port mapping). Flags repeat-override in sparq-server's arg parser, so callers can
# still append e.g. `--addr 0.0.0.0:8080` plus `--format ntriples /data/file.nt` as CMD args.
ENTRYPOINT ["/usr/local/bin/sparq-server", "--addr", "0.0.0.0:3030"]
CMD []
