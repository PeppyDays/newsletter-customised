# Rust Exercise

This is a repository for learning Rust and Axum by following a book named Zero to Production.

The original content is great, but I want to make it more practical for me. As a result, I will try to replace the framework from Actix to Axum. Furthermore, I will try to organize the code in a more modular way for more readability and maintainability.

## Local Environment Setup

Required to install dependencies:

- Docker and Docker Compose
- Just
- SQLx CLI
- Bunyan

If you are using Mac, you can install them with the following commands:

```bash
brew install --cask docker
brew install just
cargo install sqlx-cli --no-default-features --features rustls,postgres,mysql
cargo install bunyan
```
