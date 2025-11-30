FROM --platform=linux/arm64 rust:1.91-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash app

WORKDIR /app

# Copy all source
COPY . .

# Build application
ENV DATABASE_URL="postgres://dummy:dummy@dummy:5432/dummy"
RUN cargo build --release

# Set ownership and switch to non-root user
RUN chown -R app:app /app
USER app

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/k8s/health || exit 1

CMD ["./target/release/catalytics-core"]