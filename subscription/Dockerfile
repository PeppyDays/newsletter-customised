FROM rust:1.75-bookworm as chef

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner

# TODO: Simplify copying Cargo.toml and Cargo.lock file in a single shot
COPY Cargo.* ./
COPY domain/Cargo.* domain/
COPY infrastructure/messengers/Cargo.* infrastructure/messengers/
COPY infrastructure/repositories/Cargo.* infrastructure/repositories/
COPY interface/api/Cargo.* interface/api/
COPY runner/Cargo.* runner/
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin app

FROM debian:bookworm-slim AS runtime

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/app app
COPY configuration.yaml .

ENTRYPOINT ["./app"]

