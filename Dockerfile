# LoamSpine Docker Image — musl-static ecoBin build
# Pure Rust, multi-stage build for minimal image size
# SPDX-License-Identifier: AGPL-3.0-or-later

# =============================================================================
# Stage 1: Builder — musl-static x86_64
# =============================================================================
FROM rust:1.87-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY .cargo/ ./.cargo/
COPY crates/loam-spine-core/Cargo.toml crates/loam-spine-core/
COPY crates/loam-spine-api/Cargo.toml crates/loam-spine-api/
COPY bin/loamspine-service/Cargo.toml bin/loamspine-service/

# Create dummy source files for dependency caching
RUN mkdir -p crates/loam-spine-core/src crates/loam-spine-api/src bin/loamspine-service && \
    echo "pub fn main() {}" > crates/loam-spine-core/src/lib.rs && \
    echo "pub fn main() {}" > crates/loam-spine-api/src/lib.rs && \
    echo "fn main() {}" > bin/loamspine-service/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release --target x86_64-unknown-linux-musl -p loamspine-service && \
    rm -rf crates/*/src bin/loamspine-service/main.rs

# Copy actual source code
COPY crates/ crates/
COPY bin/ bin/

# Touch source files to invalidate cache
RUN touch crates/loam-spine-core/src/lib.rs crates/loam-spine-api/src/lib.rs bin/loamspine-service/main.rs

# Build the binary
RUN cargo build --release --target x86_64-unknown-linux-musl --bin loamspine \
    && strip /app/target/x86_64-unknown-linux-musl/release/loamspine

# =============================================================================
# Stage 2: Minimal runtime — static binary needs no libc
# =============================================================================
FROM alpine:3.20 AS runtime

RUN addgroup -g 1000 loamspine && \
    adduser -D -u 1000 -G loamspine loamspine

RUN mkdir -p /var/lib/loamspine/data && \
    chown -R loamspine:loamspine /var/lib/loamspine

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/loamspine /usr/local/bin/loamspine

USER loamspine
WORKDIR /home/loamspine

ENV LOAMSPINE_DATA_DIR=/var/lib/loamspine/data
ENV LOAMSPINE_LOG_LEVEL=info

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD test -d /var/lib/loamspine/data || exit 1

# tarpc: 9001 (primal-to-primal), JSON-RPC: 8080 (external clients)
EXPOSE 9001 8080

VOLUME ["/var/lib/loamspine/data"]

CMD ["loamspine", "server"]
