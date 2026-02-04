FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app
# Install system dependencies required for building
RUN apk add --no-cache musl-dev openssl-dev pkgconf binutils curl

FROM chef AS planner
COPY . .
# Compute a lock-like file for the project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
# Enable offline mode for sqlx to build without a database connection.
# NOTE: This requires `sqlx-data.json` to be present in the project root.
# Run `cargo sqlx prepare` locally before building the image.
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin cayopay-server && \
    strip target/release/cayopay-server

FROM alpine:latest AS runtime
WORKDIR /app

# ca-certificates are required for TLS
RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /app/target/release/cayopay-server /usr/local/bin/

ENV APP_ENVIRONMENT=production
ENV HOST=0.0.0.0
ENTRYPOINT ["/usr/local/bin/cayopay-server"]
