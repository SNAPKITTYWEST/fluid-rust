# FLUID RUST v1.0.0 - Production Dockerfile
# Multi-stage build: compile → runtime

FROM rust:1.70 as builder

LABEL maintainer="jessica@collectivekitty.com"
LABEL description="FLUID RUST: Verified systems language with Liquid types and algebraic effects"

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY compiler ./compiler
COPY prover ./prover
COPY runtime ./runtime

# Build all components with optimizations
RUN cargo build --release --workspace \
    && cargo test --release --all 2>&1 | grep -E "test result|passed|failed"

# Runtime stage - minimal footprint
FROM debian:bookworm-slim

LABEL version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/SNAPKITTYWEST/fluid-rust"

# Install minimal dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged user for security
RUN useradd -m -u 1000 -s /sbin/nologin fluid-rust

# Copy compiled binaries from builder
COPY --from=builder /build/target/release/fluidc /usr/local/bin/
COPY --from=builder /build/target/release/fluid-prover /usr/local/bin/
COPY --from=builder /build/README.md /usr/local/share/doc/fluid-rust/
COPY --from=builder /build/LICENSE-* /usr/local/share/doc/fluid-rust/

# Set up working directory
WORKDIR /home/fluid-rust

# Switch to unprivileged user
USER fluid-rust

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD fluidc --version || exit 1

# Default entrypoint
ENTRYPOINT ["fluidc"]
CMD ["--help"]
