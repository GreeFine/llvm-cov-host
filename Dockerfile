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

# This should be debian:stable-slim but we need rustup to use llvm-cov components and tools
FROM chef AS runtime
WORKDIR /app
RUN rustup component add llvm-tools-x86_64-unknown-linux-gnu
RUN cargo install cargo-llvm-cov llvm-cov-pretty

RUN apt update && apt install -y ca-certificates && rm -rf /var/lib/apt/lists/* 
COPY --from=builder /app/target/release/llvm-cov-host /app/llvm-cov-host
COPY templates/main.css templates/main.css
ENTRYPOINT ["/app/llvm-cov-host"]
