# Builder stage
FROM rust:1.86-slim-bookworm AS builder

WORKDIR /app

# Install minimal dependencies including OpenSSL
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy .env file first
COPY .env ./

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies for OpenSSL
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy only the binary
COPY --from=builder /app/target/release/safatanc-connect-core .

# Copy .env for runtime
COPY .env ./

# Set environment variables
ENV RUST_LOG=info
ENV APP_ENVIRONMENT=production

# Expose the port
EXPOSE 8080

# Run the binary
CMD ["./safatanc-connect-core"] 