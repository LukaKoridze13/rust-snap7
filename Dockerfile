FROM rust:latest

# Install necessary packages
RUN apt-get update && apt-get install -y build-essential

# Install cargo-watch globally
RUN cargo install cargo-watch

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs # Create a dummy main.rs to build dependencies
RUN cargo build --release
RUN rm -rf src

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Command to run the application with cargo-watch
CMD ["cargo-watch", "-qc", "-x", "run"]
