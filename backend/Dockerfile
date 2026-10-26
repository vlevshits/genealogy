# =============================================================================
# Multi-stage Dockerfile for genealogy-backend (Rust Axum)
# =============================================================================

# --- Stage 1: Build binary ---
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependency layer
COPY Cargo.toml Cargo.lock ./
# Create dummy lib/main to build dependencies only
RUN mkdir src tests && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src tests

# Copy real sources and migrations
COPY src ./src
COPY migrations ./migrations
COPY tests ./tests

# Build final release binary
RUN touch src/main.rs src/lib.rs && cargo build --release

# --- Stage 2: Runtime image ---
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install runtime dependencies (curl is required for docker healthcheck)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create directories for media and backups
RUN mkdir -p /app/data/media /app/data/backups

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/genealogy-backend /app/genealogy-backend

# Expose backend port
EXPOSE 3000

ENV PORT=3000 \
    RUST_LOG=info,genealogy=debug \
    MEDIA_STORAGE_PATH=/app/data/media \
    BACKUP_PATH=/app/data/backups

# Healthcheck matching docker-compose
HEALTHCHECK --interval=10s --timeout=5s --retries=5 --start-period=5s \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

ENTRYPOINT ["/app/genealogy-backend"]
