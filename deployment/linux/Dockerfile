FROM rust:1.90.0-bookworm@sha256:3914072ca0c3b8aad871db9169a651ccfce30cf58303e5d6f2db16d1d8a7e58f AS builder
WORKDIR /source
COPY product ./product
COPY branding ./branding
COPY deployment ./deployment
COPY LICENCE ./LICENCE
RUN cargo build --manifest-path product/Cargo.toml --locked --release -p swan-management

FROM debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
RUN useradd --system --uid 10001 --create-home swan && mkdir -p /var/lib/swan && chown swan:swan /var/lib/swan
COPY --from=builder /source/product/target/release/swan-management /usr/local/bin/swan-management
USER swan
ENV SWAN_DATA_DIR=/var/lib/swan SWAN_LISTEN=0.0.0.0:8080 SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
EXPOSE 8080
ENTRYPOINT ["swan-management"]
