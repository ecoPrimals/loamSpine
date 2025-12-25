# LoamSpine Docker Image
# Pure Rust, multi-stage build for minimal image size

# =============================================================================
# Stage 1: Builder
# =============================================================================
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/loam-spine-core/Cargo.toml crates/loam-spine-core/
COPY crates/loam-spine-api/Cargo.toml crates/loam-spine-api/

# Create dummy source files for dependency caching
RUN mkdir -p crates/loam-spine-core/src crates/loam-spine-api/src && \
    echo "pub fn main() {}" > crates/loam-spine-core/src/lib.rs && \
    echo "pub fn main() {}" > crates/loam-spine-api/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release && \
    rm -rf crates/*/src

# Copy actual source code
COPY crates/ crates/

# Touch source files to invalidate cache
RUN touch crates/loam-spine-core/src/lib.rs crates/loam-spine-api/src/lib.rs

# Build the actual application
RUN cargo build --release --all-features

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Create non-root user for security
RUN groupadd --gid 1000 loamspine && \
    useradd --uid 1000 --gid loamspine --shell /bin/bash --create-home loamspine

# Install runtime dependencies (minimal)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create data directories
RUN mkdir -p /var/lib/loamspine/data && \
    chown -R loamspine:loamspine /var/lib/loamspine

# Copy built artifacts
COPY --from=builder /app/target/release/libloam_spine_core.rlib /usr/local/lib/
COPY --from=builder /app/target/release/libloam_spine_api.rlib /usr/local/lib/

# Switch to non-root user
USER loamspine
WORKDIR /home/loamspine

# Environment variables
ENV LOAMSPINE_DATA_DIR=/var/lib/loamspine/data
ENV LOAMSPINE_LOG_LEVEL=info
ENV RUST_BACKTRACE=1

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD test -d /var/lib/loamspine/data || exit 1

# Expose ports
# tarpc: 9001 (primal-to-primal)
# JSON-RPC: 8080 (external clients)
EXPOSE 9001 8080

# Volume for persistent data
VOLUME ["/var/lib/loamspine/data"]

# Default command (placeholder - will be replaced when we have a binary)
CMD ["echo", "LoamSpine library built successfully. Add a service binary to run."]

