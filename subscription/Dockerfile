FROM rust:1.75.0-slim-bookworm as chef

RUN cargo install cargo-chef

WORKDIR /app
COPY subscription/Cargo.* .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

WORKDIR /app
COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# TODO: Modify this to copy subscription/** to app/**
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
# TODO: After change the configuration file location, modify this line
COPY configuration.yaml .

ENTRYPOINT ["./app"]

