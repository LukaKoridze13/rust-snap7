FROM rust:latest

RUN apt-get update && apt-get install -y build-essential

RUN cargo install cargo-watch

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src

CMD ["cargo-watch", "-qc", "-x", "run"]
