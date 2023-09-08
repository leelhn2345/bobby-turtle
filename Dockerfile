FROM lukemathwalker/cargo-chef:latest-rust-slim-bookworm AS chef
WORKDIR /app
RUN apt update && apt install pkg-config openssl libssl-dev -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin turtle-bot

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt update -y \
    && apt install -y --no-install-recommends openssl libssl-dev ca-certificates \
    # Clean up
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY config config
COPY --from=builder /app/target/release/turtle-bot turtle-bot
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./turtle-bot"]
