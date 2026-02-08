#!/bin/bash
set -e
# Navigate to project root
cd "$(dirname "$0")/.."

# Build graft
cargo build --quiet

echo "Testing rule file processing..."

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Create test file
echo 'fn main() { let x = 1 + 2; let y = foo(x); }' > "$TEMP_DIR/target.rs"

# Run graft with rule file
./target/debug/graft "$TEMP_DIR/target.rs" \
    --rule-file ./examples/rules.toml \
    --in-place

# Verify content
EXPECTED='fn main() { let x = pow(1, 2); let y = bar(x); }'
ACTUAL=$(cat "$TEMP_DIR/target.rs")

if [ "$ACTUAL" == "$EXPECTED" ]; then
    echo "Rule file test PASSED"
else
    echo "Rule file test FAILED"
    echo "Expected: $EXPECTED"
    echo "Got:      $ACTUAL"
    exit 1
fi
