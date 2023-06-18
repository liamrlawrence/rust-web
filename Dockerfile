FROM rust:1.69-slim-bullseye AS chef
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
RUN cargo chef cook --release --recipe-path recipe.json     # Compile dependencies

COPY . .
RUN cargo build --release --bin app



# Run application
FROM debian:bullseye-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates
COPY --from=builder /app/target/release/app /usr/local/bin
COPY --from=builder /app/static /app/static
COPY --from=builder /app/templates /app/templates

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/app"]

