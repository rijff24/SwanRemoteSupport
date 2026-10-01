FROM rust:1.90.0-bookworm AS builder
WORKDIR /source
COPY product ./product
COPY branding ./branding
COPY deployment ./deployment
COPY LICENCE ./LICENCE
RUN cargo build --manifest-path product/Cargo.toml --locked --release -p swan-management

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 --create-home swan && mkdir -p /var/lib/swan && chown swan:swan /var/lib/swan
COPY --from=builder /source/product/target/release/swan-management /usr/local/bin/swan-management
USER swan
ENV SWAN_DATA_DIR=/var/lib/swan SWAN_LISTEN=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["swan-management"]
