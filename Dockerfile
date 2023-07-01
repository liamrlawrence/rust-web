FROM rust:1.70-slim-bookworm AS chef
LABEL description="Rust web server"
WORKDIR /app
RUN cargo install cargo-chef 
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssh-dev


# Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


# Build
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
#RUN cargo chef cook --release --recipe-path recipe.json         # Compile dependencies
RUN cargo chef cook --recipe-path recipe.json       # Compile dependencies

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
COPY . .
#RUN cargo build --release --bin app                             # Compile application
RUN cargo build --bin app                           # Compile application




# Run application
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl-dev

COPY --from=builder /app/target/debug/app /usr/local/bin

EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/app"]

