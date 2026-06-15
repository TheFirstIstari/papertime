#!/bin/bash
# Fix: remove duplicate static/data/ directory and rebuild
set -e
cd "$(dirname "$0")"

# Remove the old static/data/ directory
rm -rf static/data/

# Rebuild
bun run build

echo "Build complete"
