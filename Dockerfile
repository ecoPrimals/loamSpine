# LoamSpine Docker Image
# Pure Rust, multi-stage build for minimal image size
# SPDX-License-Identifier: AGPL-3.0-or-later

# =============================================================================
# Stage 1: Builder
# =============================================================================
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/loam-spine-core/Cargo.toml crates/loam-spine-core/
COPY crates/loam-spine-api/Cargo.toml crates/loam-spine-api/
COPY bin/loamspine-service/Cargo.toml bin/loamspine-service/

# Create dummy source files for dependency caching
RUN mkdir -p crates/loam-spine-core/src crates/loam-spine-api/src bin/loamspine-service && \
    echo "pub fn main() {}" > crates/loam-spine-core/src/lib.rs && \
    echo "pub fn main() {}" > crates/loam-spine-api/src/lib.rs && \
    echo "fn main() {}" > bin/loamspine-service/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release && \
    rm -rf crates/*/src bin/loamspine-service/main.rs

# Copy actual source code
COPY crates/ crates/
COPY bin/ bin/

# Touch source files to invalidate cache
RUN touch crates/loam-spine-core/src/lib.rs crates/loam-spine-api/src/lib.rs bin/loamspine-service/main.rs

# Build the binary
RUN cargo build --release --bin loamspine

# =============================================================================
# Stage 2: Runtime (minimal)
# =============================================================================
FROM debian:bookworm-slim AS runtime

RUN groupadd --gid 1000 loamspine && \
    useradd --uid 1000 --gid loamspine --shell /bin/bash --create-home loamspine

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /var/lib/loamspine/data && \
    chown -R loamspine:loamspine /var/lib/loamspine

COPY --from=builder /app/target/release/loamspine /usr/local/bin/loamspine

USER loamspine
WORKDIR /home/loamspine

ENV LOAMSPINE_DATA_DIR=/var/lib/loamspine/data
ENV LOAMSPINE_LOG_LEVEL=info
ENV RUST_BACKTRACE=1

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD test -d /var/lib/loamspine/data || exit 1

# tarpc: 9001 (primal-to-primal), JSON-RPC: 8080 (external clients)
EXPOSE 9001 8080

VOLUME ["/var/lib/loamspine/data"]

CMD ["loamspine", "server"]
