# Multi-stage build for Catalytics Core Rust application
# Using ARM64 architecture for AWS Graviton2 compatibility

FROM --platform=linux/arm64 rust:1.82-slim AS builder

# Set environment variables for build
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV RUSTFLAGS="-C target-feature=+crt-static"

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source code, migrations, and SQLx offline data
COPY src/ ./src/
COPY migrations/ ./migrations/
COPY .sqlx/ ./.sqlx/

# Build the actual application in offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Production stage
FROM --platform=linux/arm64 debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash app

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/catalytics-core ./catalytics-core

# Set ownership
RUN chown -R app:app /app

# Switch to non-root user
USER app

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/k8s/health || exit 1

# Command to run the application
CMD ["./catalytics-core"]