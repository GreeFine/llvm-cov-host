FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN rustup install nightly

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .

RUN cargo +nightly build --release 

FROM debian:stable-slim AS runtime
WORKDIR /app

RUN apt update && apt install -y ca-certificates && rm -rf /var/lib/apt/lists/* 
COPY --from=builder /app/target/release/llvm-cov-host /app/llvm-cov-host
ENTRYPOINT ["/app/llvm-cov-host"]
