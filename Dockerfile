FROM rust:1.90-bookworm AS rust-build

WORKDIR /usr/local/src/beep

RUN cargo install sqlx-cli --no-default-features --features postgres

COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY core/Cargo.toml ./core/
COPY libs/config/Cargo.toml ./libs/config/

RUN \
    mkdir -p api/src core/src libs/config/src && \
    echo "fn main() {}" > api/src/main.rs && \
    touch core/src/lib.rs && \
    touch libs/config/src/lib.rs && \
    cargo build --release

COPY api api
COPY core core
COPY libs/config libs/config

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates=20230311+deb12u1 \
    libssl3=3.0.17-1~deb12u2 && \
    rm -rf /var/lib/apt/lists/* && \
    addgroup \
    --system \
    --gid 1000 \
    beep && \
    adduser \
    --system \
    --no-create-home \
    --disabled-login \
    --uid 1000 \
    --gid 1000 \
    beep

USER beep

FROM runtime AS api

COPY --from=rust-build /usr/local/src/beep/target/release/api /usr/local/bin/
#COPY --from=rust-build /usr/local/src/beep/core/migrations /usr/local/src/beep/migrations
COPY --from=rust-build /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["api"]
