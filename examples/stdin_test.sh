#!/bin/bash
set -e
# Navigate to project root
cd "$(dirname "$0")/.."

# Build graft
cargo build --quiet

echo "Testing stdin support..."

# Input: Rust code
INPUT='fn main() { let x = 1 + 2; }'

# Expected Output: Rust code with transformation
EXPECTED='fn main() { let x = add(1, 2); }'

# Run graft reading from stdin
OUTPUT=$(echo "$INPUT" | ./target/debug/graft \
    --language rust \
    --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
    --template 'add(${l}, ${r})')

if [ "$OUTPUT" == "$EXPECTED" ]; then
    echo "Stdin test PASSED"
else
    echo "Stdin test FAILED"
    echo "Expected: $EXPECTED"
    echo "Got:      $OUTPUT"
    exit 1
fi
