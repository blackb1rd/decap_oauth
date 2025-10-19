# Stage 1: Build the application
FROM rust:trixie AS builder

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Create the final image
FROM debian:trixie

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/decap_cms_oauth /usr/local/bin/decap_cms_oauth

# Expose the port the application runs on
EXPOSE 3005

# Set the entrypoint
CMD ["decap_cms_oauth"]
