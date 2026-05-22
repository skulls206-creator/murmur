# ── Murmur Dockerfile ──────────────────────────────────
# Multi-stage build for minimal production image

# ── Builder Stage ──
FROM rust:1.79-slim-bookworm AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p static && \
    cargo build --release 2>/dev/null || true

# Copy real source and rebuild
COPY src/ src/
COPY templates/ templates/
COPY static/ static/

# Touch main.rs to force recompile
RUN touch src/main.rs && \
    cargo build --release

# ── Runtime Stage ──
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN addgroup --system murmur && \
    adduser --system --ingroup murmur murmur

WORKDIR /app

COPY --from=builder /build/target/release/murmur /app/murmur
COPY --from=builder /build/static/ /app/static/
COPY --from=builder /build/templates/ /app/templates/
COPY .env.example /app/.env.example

USER murmur

EXPOSE 8080

ENV MURMUR_BIND_ADDR=0.0.0.0:8080

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -sf http://localhost:8080/api/health || exit 1

ENTRYPOINT ["/app/murmur"]
