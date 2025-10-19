# Build stage
FROM rust:1.82-slim AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests and cargo config
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo

# Copy vendored dependencies
COPY vendor ./vendor

# Copy source code
COPY src ./src

# Build for release with vendored dependencies
RUN cargo build --release --offline

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/decap_oauth /usr/local/bin/decap_oauth

# Set the default port
ENV PORT=3005

# Expose the port
EXPOSE 3005

# Run the binary
CMD ["decap_oauth"]
