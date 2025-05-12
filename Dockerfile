# Builder stage
FROM rust:1.86-slim-bookworm as builder

WORKDIR /app

# Install minimal dependencies including OpenSSL
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

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

# Set environment variables
ENV RUST_LOG=info
ENV APP_ENVIRONMENT=production

# Expose the port
EXPOSE 8080

# Run the binary
CMD ["./safatanc-connect-core"] 