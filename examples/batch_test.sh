#!/bin/bash
set -e
# Navigate to project root
cd "$(dirname "$0")/.."

# Build graft
cargo build --quiet

echo "Testing batch processing..."

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Create test files
echo 'fn main() { let x = 1 + 2; }' > "$TEMP_DIR/a.rs"
echo 'fn test() { let y = 3 + 4; }' > "$TEMP_DIR/b.rs"

# Run graft on multiple files using glob
./target/debug/graft "$TEMP_DIR/*.rs" \
    --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
    --template 'add(${l}, ${r})' \
    --in-place

# Verify content
grep -q "add(1, 2)" "$TEMP_DIR/a.rs" || { echo "Failed to transform a.rs"; exit 1; }
grep -q "add(3, 4)" "$TEMP_DIR/b.rs" || { echo "Failed to transform b.rs"; exit 1; }

echo "Batch test PASSED"
