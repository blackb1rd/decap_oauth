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

# Create .cargo/config.toml if it doesn't exist or has different content
EXPECTED_CONFIG='[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"'

if [ -f .cargo/config.toml ]; then
    CURRENT_CONFIG=$(cat .cargo/config.toml)
    if [ "$CURRENT_CONFIG" != "$EXPECTED_CONFIG" ]; then
        echo "Warning: .cargo/config.toml exists with different content."
        echo "The expected content for Docker builds is:"
        echo "$EXPECTED_CONFIG"
        echo ""
        read -p "Do you want to overwrite it? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Skipping .cargo/config.toml creation. Docker build may fail if the existing config is incompatible."
        else
            echo "$EXPECTED_CONFIG" > .cargo/config.toml
            echo ".cargo/config.toml updated."
        fi
    else
        echo ".cargo/config.toml already configured correctly."
    fi
else
    echo "Creating .cargo/config.toml..."
    mkdir -p .cargo
    echo "$EXPECTED_CONFIG" > .cargo/config.toml
fi

echo "✓ Project is ready for Docker build!"
echo ""
echo "You can now build the Docker image with:"
echo "  docker build -t decap-oauth ."
echo ""
echo "Or use docker-compose:"
echo "  docker-compose up -d"
