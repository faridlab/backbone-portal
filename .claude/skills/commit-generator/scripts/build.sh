#!/bin/bash

# Build script for commit-analyzer Rust binary
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

echo "Building commit-analyzer Rust binary..."
cd "$SCRIPT_DIR"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Build in release mode for better performance
cargo build --release

# Create symlink in repo root for easy access
if [ -f "$SCRIPT_DIR/target/release/commit-analyzer" ]; then
    ln -sf "$SCRIPT_DIR/target/release/commit-analyzer" "$REPO_ROOT/commit-analyzer"
    echo "✓ Built successfully: commit-analyzer binary available at $REPO_ROOT/commit-analyzer"
else
    echo "✗ Build failed"
    exit 1
fi

echo ""
echo "Usage examples:"
echo "  ./commit-analyzer                    # Show suggested commit message"
echo "  ./commit-analyzer --verbose          # Show detailed analysis"
echo "  ./commit-analyzer --quiet            # Show only commit message"
echo ""