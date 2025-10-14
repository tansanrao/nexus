# Multi-stage build for Rust API server
FROM rust:1.90-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies first (caching layer)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY migrations ./migrations

# Build the real application
# Touch main.rs to force rebuild
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api-server /app/api-server

# Copy Rocket.toml (will be overridden by env vars in production)
COPY Rocket.toml /app/Rocket.toml

# Create directories for mirrors and cache
RUN mkdir -p /app/mirrors /app/cache

# Expose API port
EXPOSE 8000

# Run the binary
CMD ["/app/api-server"]
