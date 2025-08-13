# Multi-stage Docker build for Astor Currency System

# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 astor

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY config ./config

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 astor

# Create necessary directories
RUN mkdir -p /app/config /app/logs /var/log/astor && \
    chown -R astor:astor /app /var/log/astor

# Copy binary from builder stage
COPY --from=builder /app/target/release/astor-currency /app/astor-currency
COPY --from=builder /app/config /app/config
COPY --from=builder /app/migrations /app/migrations

# Set permissions
RUN chmod +x /app/astor-currency && \
    chown -R astor:astor /app

# Switch to app user
USER astor

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["./astor-currency"]
