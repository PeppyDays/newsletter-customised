FROM rust:1.75.0-slim-bookworm as chef

RUN cargo install cargo-chef

WORKDIR /app
COPY Cargo.* .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

WORKDIR /app
COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
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

