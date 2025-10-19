#!/bin/bash
# Script to prepare the project for Docker build

set -e

echo "Preparing project for Docker build..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust first."
    exit 1
fi

# Vendor dependencies
echo "Vendoring dependencies..."
cargo vendor

# Create .cargo/config.toml if it doesn't exist
if [ ! -f .cargo/config.toml ]; then
    echo "Creating .cargo/config.toml..."
    mkdir -p .cargo
    cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF
fi

echo "✓ Project is ready for Docker build!"
echo ""
echo "You can now build the Docker image with:"
echo "  docker build -t decap-oauth ."
echo ""
echo "Or use docker-compose:"
echo "  docker-compose up -d"
