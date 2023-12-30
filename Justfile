default:
	just --list

test: migrate
	cargo test

build:
	cargo build

run: migrate
	cargo run

sqlx-prepare:
	cargo sqlx prepare --workspace

watch-run: migrate
	cargo watch -x run

container-up:
	docker compose up -d

container-down:
	docker compose down

migrate:
	sqlx database create
	sqlx migrate run

