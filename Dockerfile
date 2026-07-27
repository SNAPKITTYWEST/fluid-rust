# Multi-stage build for FLUID RUST

FROM rust:1.70 as builder

WORKDIR /build

COPY . .

RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 fluid-rust

COPY --from=builder /build/target/release/fluid-rust-* /usr/local/bin/

WORKDIR /home/fluid-rust

USER fluid-rust

ENTRYPOINT ["fluid-rust-compiler"]

CMD ["--version"]
