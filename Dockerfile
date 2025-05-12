# Builder stage
FROM rust:1.77-slim-bookworm as builder

WORKDIR /usr/src/app

# Install minimal build dependencies
RUN apt-get update && \
  apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# Create empty project for caching dependencies
RUN cargo init --lib
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy and build
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install SSL certificates for HTTPS requests
RUN apt-get update && \
  apt-get install -y ca-certificates && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/safatanc-connect-core ./app

# Create a non-root user
RUN useradd -m -u 1001 appuser
USER appuser

# Set environment variables
ARG APP_ENVIRONMENT=production
ENV RUST_LOG=info
ENV APP_ENVIRONMENT=${APP_ENVIRONMENT}

# Expose the port the app runs on
EXPOSE 8080

# Run the binary
CMD ["./app"] 