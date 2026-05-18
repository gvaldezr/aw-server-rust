#################################################
# Stage 1: Builder
#################################################
ARG RUST_VERSION=1.95
FROM rust:${RUST_VERSION}-bookworm AS builder

# Set working directory
WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo workspace manifests first (for layer caching)
COPY Cargo.toml ./

# Create dummy main files to build dependencies first (cache optimization)
RUN mkdir -p aw-server/src && \
    mkdir -p aw-datastore/src && \
    mkdir -p aw-models/src && \
    mkdir -p aw-query/src && \
    mkdir -p aw-transform/src && \
    mkdir -p aw-client-rust/src && \
    mkdir -p aw-sync/src && \
    echo "fn main() {}" > aw-server/src/main.rs && \
    echo "pub fn dummy() {}" > aw-server/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-datastore/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-models/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-query/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-transform/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-client-rust/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-sync/src/lib.rs

# Copy crate manifests
COPY aw-server/Cargo.toml ./aw-server/
COPY aw-datastore/Cargo.toml ./aw-datastore/
COPY aw-models/Cargo.toml ./aw-models/
COPY aw-query/Cargo.toml ./aw-query/
COPY aw-transform/Cargo.toml ./aw-transform/
COPY aw-client-rust/Cargo.toml ./aw-client-rust/
COPY aw-sync/Cargo.toml ./aw-sync/

# Copy benches directories (needed for Cargo.toml bench declarations)
COPY aw-query/benches/ ./aw-query/benches/
COPY aw-transform/benches/ ./aw-transform/benches/

# Build dependencies only (cached layer)
RUN cargo build --release --bin aw-server

# Remove dummy source files and ALL workspace build artifacts
# (keep external dependencies cached)
RUN rm -rf aw-server/src aw-datastore/src aw-models/src aw-query/src aw-transform/src aw-client-rust/src aw-sync/src && \
    find target/release -name 'aw-*' -o -name 'aw_*' -o -name 'libaw_*' | xargs rm -rf && \
    rm -rf target/release/.fingerprint/aw-* target/release/.fingerprint/aw_*

# Copy actual source code
COPY aw-server/ ./aw-server/
COPY aw-datastore/ ./aw-datastore/
COPY aw-models/ ./aw-models/
COPY aw-query/ ./aw-query/
COPY aw-transform/ ./aw-transform/
COPY aw-client-rust/ ./aw-client-rust/
COPY aw-sync/ ./aw-sync/

# Build release binary with all optimizations
RUN cargo build --release --bin aw-server

# Strip debug symbols to reduce binary size
RUN strip target/release/aw-server

#################################################
# Stage 2: Runtime
#################################################
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash awuser && \
    mkdir -p /home/awuser/.local/share/activitywatch && \
    chown -R awuser:awuser /home/awuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/aw-server /usr/local/bin/aw-server

# Ensure binary is executable
RUN chmod +x /usr/local/bin/aw-server

# Switch to non-root user
USER awuser
WORKDIR /home/awuser

# Expose port
EXPOSE 5600

# Health check (using curl installed in runtime image)
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:5600/api/0/info || exit 1

# Set default environment variables
ENV RUST_LOG=info \
    DB_LOG_LEVEL=warn \
    AW_HOST=0.0.0.0 \
    AW_PORT=5600

# Run server
CMD ["aw-server"]
