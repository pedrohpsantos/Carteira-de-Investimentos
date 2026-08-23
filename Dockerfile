# Build stage
FROM rust:1.80-slim AS builder

WORKDIR /usr/src/app
COPY . .

# Install pkg-config and libssl-dev if required by any dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Build the application
# Use the SQLX_OFFLINE mode if you ran `cargo sqlx prepare`
# Otherwise, for simplicity, we assume we compile without the live DB or we provide it.
# Note: Since sqlx::query! macros need the DB at compile time, you must run `cargo sqlx prepare` locally
# before building this image, and pass ENV SQLX_OFFLINE=true.
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Run stage
FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/wallet-live /app/wallet-live
COPY --from=builder /usr/src/app/templates /app/templates
COPY --from=builder /usr/src/app/.env /app/.env

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

EXPOSE 3000

CMD ["./wallet-live"]
