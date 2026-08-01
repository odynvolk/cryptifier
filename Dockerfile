# Syntax directive for buildkit features
# syntax=docker/dockerfile:1

# ===== Stage 1: Build =====
FROM rust:1.85-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    g++ \
    libssl-dev \
    pkg-config \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/cryptifier

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to pre-build dependencies
RUN mkdir src && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    cargo fetch && \
    cargo build --release && \
    rm -rf src && \
    rm -rf target/release/deps/cryptifier* && \
    rm -rf target/release/.fingerprint/cryptifier*

# Copy actual source code
COPY src ./src

# Build the release binary
RUN cargo build --release

# ===== Stage 2: Runtime =====
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r cryptifier && useradd -r -g cryptifier cryptifier

RUN apt-get install tzdata

WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /usr/src/cryptifier/target/release/cryptifier /app/cryptifier

# Copy .env file if it exists (will be mounted at runtime via docker-compose)
COPY .env.example /app/.env.example

# Create a non-root user and set permissions
RUN chown cryptifier:cryptifier /app

# Switch to non-root user
USER cryptifier

# Health check - verify the process is responsive by checking if it's running
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD pgrep -f cryptifier || exit 1

# Expose no ports (this is a background monitoring service, not a web server)

ENV TZ=Europe/Berlin

# Run the application
ENTRYPOINT ["/app/cryptifier"]
