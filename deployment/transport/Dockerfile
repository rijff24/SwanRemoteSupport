# Candidate only: build the maintained transport from immutable public inputs.
# No company configuration, credentials or local build context enter this image.
FROM rust:1.90.0-bookworm@sha256:3914072ca0c3b8aad871db9169a651ccfce30cf58303e5d6f2db16d1d8a7e58f AS builder
RUN apt-get update && apt-get install -y --no-install-recommends git python3 pkg-config libssl-dev libsodium-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
RUN git init recipe && git -C recipe remote add origin https://github.com/rijff24/SwanRemoteSupport && \
    git -C recipe fetch --depth 1 origin 650361413543baf7ef150f5960544154ec77ddeb && \
    git -C recipe checkout --detach FETCH_HEAD && \
    test "$(git -C recipe rev-parse HEAD)" = 650361413543baf7ef150f5960544154ec77ddeb
RUN git init transport && git -C transport remote add origin https://github.com/rustdesk/rustdesk-server && \
    git -C transport fetch --depth 1 origin 9bae9f2f39d92c4b4ba2e28e089da5071897b22e && \
    git -C transport checkout --detach FETCH_HEAD && git -C transport submodule update --init --recursive
RUN python3 recipe/deployment/transport/prepare_source.py transport tests && \
    python3 recipe/deployment/transport/prepare_source.py transport fix
WORKDIR /build/transport
RUN cargo build --release --locked --bin hbbs --bin hbbr
RUN python3 ../recipe/deployment/transport/archive_source.py . /build/transport-build-source.zip

FROM debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 libsodium23 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/transport/target/release/hbbs /build/transport/target/release/hbbr /usr/local/bin/
COPY --from=builder /build/transport/SWAN-SOURCE.json /build/transport-build-source.zip /usr/share/doc/swan-transport/
COPY --from=builder /build/recipe/LICENCE /usr/share/doc/swan-transport/LICENCE
LABEL org.opencontainers.image.source="https://github.com/rijff24/SwanRemoteSupport" \
      org.opencontainers.image.revision="650361413543baf7ef150f5960544154ec77ddeb" \
      org.opencontainers.image.licenses="AGPL-3.0" \
      com.swan.transport.status="candidate-not-production-approved"
WORKDIR /root
CMD ["hbbs"]
