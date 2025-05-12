# ---------- Builder stage ----------
FROM rust:1.86-slim-bookworm AS builder

WORKDIR /app

# Install dependencies needed to compile
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy all source code into the container (excluding files in .dockerignore)
COPY . .

# Build the Rust app in release mode
RUN cargo build --release

# ---------- Runtime stage ----------
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies required by the Rust binary (like OpenSSL)
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy only the release binary from builder stage
COPY --from=builder /app/target/release/safatanc-connect-core .

# Copy the .env file from the Docker build context (created by GitHub Actions)
COPY .env ./

# Set runtime environment variables
ENV RUST_LOG=info
ENV APP_ENVIRONMENT=production

# Expose application port
EXPOSE 8080

# Command to run the app
CMD ["./safatanc-connect-core"]
