#!/bin/bash
set -e
# Navigate to project root
cd "$(dirname "$0")/../.."

# Build graft
cargo build --quiet

# Run graft
# 1. Rewrite binary expression
# 2. Rename function call
# We do this in two steps or we can chain if we supported chaining, but CLI does one at a time.
# Let's do one complex transformation: a + b -> add(a, b)

# Create after.rs from before.rs
cp examples/rust/before.rs examples/rust/after.rs

# Apply binary expression rewrite
./target/debug/graft examples/rust/after.rs \
    --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
    --template 'add(${l}, ${r})' \
    --in-place

# Apply function rename
./target/debug/graft examples/rust/after.rs \
    --query '(call_expression function: (identifier) @n (#eq? @n "foo") arguments: (arguments) @a) @target' \
    --template 'bar${a}' \
    --in-place

echo "Rust transformation complete."

