#!/bin/bash
set -e
# Navigate to project root
cd "$(dirname "$0")/.."

# Build graft
cargo build --quiet

echo "Testing batch queries (multiple rewrites)..."

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Create test file
echo 'fn main() { let x = 1 + 2; let y = foo(x); }' > "$TEMP_DIR/target.rs"

# Run graft with multiple queries
./target/debug/graft "$TEMP_DIR/target.rs" \
    --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
    --template 'add(${l}, ${r})' \
    --query '(call_expression function: (identifier) @n (#eq? @n "foo") arguments: (arguments) @a) @target' \
    --template 'bar${a}' \
    --in-place

# Verify content
EXPECTED='fn main() { let x = add(1, 2); let y = bar(x); }'
ACTUAL=$(cat "$TEMP_DIR/target.rs")

if [ "$ACTUAL" == "$EXPECTED" ]; then
    echo "Batch query test PASSED"
else
    echo "Batch query test FAILED"
    echo "Expected: $EXPECTED"
    echo "Got:      $ACTUAL"
    exit 1
fi
