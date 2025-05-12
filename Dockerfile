# Builder stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /usr/src/app

# Install minimal build dependencies
RUN apt-get update && \
  apt-get install -y \
  pkg-config \
  && rm -rf /var/lib/apt/lists/*

# Copy and build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/safatanc-connect-core ./app

# Create a non-root user
RUN useradd -m -u 1001 appuser
USER appuser

# Set environment variables
ENV RUST_LOG=info
ENV APP_ENVIRONMENT=production

# Expose the port the app runs on
EXPOSE 8080

# Run the binary
CMD ["./app"] 