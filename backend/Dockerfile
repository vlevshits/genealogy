# =============================================================================
# Multi-stage Dockerfile for genealogy-backend (Rust Axum + genealogy-cli)
# =============================================================================

# --- Stage 1: Build binaries ---
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependency layer
COPY Cargo.toml Cargo.lock ./
# Create dummy lib/main/cli to build dependencies only
RUN mkdir -p src/bin tests && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/cli.rs && \
    cargo build --release --bins && \
    rm -rf src tests

# Copy real sources and migrations
COPY src ./src
COPY migrations ./migrations
COPY tests ./tests

# Build final release binaries (genealogy-backend & genealogy-cli)
RUN touch src/main.rs src/lib.rs src/bin/cli.rs && cargo build --release --bins

# --- Stage 2: Runtime image (non-root hardened) ---
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install runtime dependencies (curl for healthcheck, postgresql-client for pg_dump/pg_restore)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged group and user (UID/GID 10001)
RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -M -s /usr/sbin/nologin appuser && \
    mkdir -p /app/data/media /app/data/backups

# Copy binaries from builder and symlink genealogy-cli into PATH
COPY --from=builder /usr/src/app/target/release/genealogy-backend /app/genealogy-backend
COPY --from=builder /usr/src/app/target/release/genealogy-cli /app/genealogy-cli
RUN ln -s /app/genealogy-cli /usr/local/bin/genealogy-cli && \
    chown -R appuser:appgroup /app

USER 10001:10001

# Expose backend port
EXPOSE 3000

ENV PORT=3000 \
    APP_ENV=production \
    RUST_LOG=info,genealogy=debug \
    MEDIA_STORAGE_PATH=/app/data/media \
    BACKUP_PATH=/app/data/backups

# Healthcheck matching docker-compose
HEALTHCHECK --interval=10s --timeout=5s --retries=5 --start-period=5s \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

ENTRYPOINT ["/app/genealogy-backend"]
