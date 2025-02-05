# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml .

# Create a dummy main.rs file to pre-compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies (this step caches them)
RUN cargo build --release

# Remove the dummy main.rs file
RUN rm -rf src

# Copy the rest of the application code
COPY . .

# Build the application
RUN cargo build --release

# Use the same base image as the builder stage for the final stage
FROM debian:bookworm-slim

# Install necessary libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/web_server /usr/local/bin/web_server

# Copy the www directory
COPY www /www

# Expose the port the server listens on
EXPOSE 7878

# Run the server
CMD ["web_server"]
