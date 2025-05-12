# Builder stage
FROM rust:1.86-slim-bookworm AS builder

WORKDIR /app

# Install minimal dependencies including OpenSSL
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy .env file first
COPY .env* ./

# Extract DATABASE_URL dari .env file
RUN if [ -f ".env" ]; then \
  grep -o "DATABASE_URL=.*" .env > /tmp/db_url || echo "DATABASE_URL tidak ditemukan"; \
  else \
  echo "File .env tidak ditemukan"; \
  fi

# Copy source code
COPY . .

# Build the application dengan DATABASE_URL dari .env
RUN if [ -f ".env" ]; then \
  export $(grep "DATABASE_URL" .env | xargs) && \
  echo "Building with DATABASE_URL=${DATABASE_URL}" && \
  cargo build --release; \
  else \
  echo "ERROR: .env file tidak ditemukan, DATABASE_URL harus tersedia"; \
  exit 1; \
  fi

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies for OpenSSL
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy only the binary
COPY --from=builder /app/target/release/safatanc-connect-core .

# Copy .env file for runtime
COPY --from=builder /app/.env* ./

# Set environment variables
ENV RUST_LOG=info
ENV APP_ENVIRONMENT=production

# Expose the port
EXPOSE 8080

# Run the binary
CMD ["./safatanc-connect-core"] 