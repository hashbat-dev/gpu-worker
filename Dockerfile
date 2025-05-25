# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY transformations/Cargo.toml ./transformations/

# Build dependencies - this is the caching Docker layer!
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir transformations/src && \
    echo "" > transformations/src/lib.rs && \
    cargo build --release && \
    rm -rf src transformations/src

# Copy source code
COPY . .

# Build application
RUN touch src/main.rs transformations/src/lib.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash appuser

# Copy binary from builder
COPY --from=builder /app/target/release/gpu-worker /usr/local/bin/gpu-worker

# Set ownership
RUN chown appuser:appuser /usr/local/bin/gpu-worker

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
ENTRYPOINT ["gpu-worker"]