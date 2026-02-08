#!/bin/bash
set -e
# Navigate to project root
cd "$(dirname "$0")/../.."

# Build graft
cargo build --quiet

# Create after.js from before.js
cp examples/javascript/before.js examples/javascript/after.js

# Apply binary expression rewrite
# Tree-sitter JS: (binary_expression left: (_) @l operator: "+" right: (_) @r)
./target/debug/graft examples/javascript/after.js \
    --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
    --template 'add(${l}, ${r})' \
    --in-place

# Apply console.log rewrite -> logger.info
# (call_expression function: (member_expression object: (identifier) @obj property: (property_identifier) @prop) arguments: (arguments) @args)
./target/debug/graft examples/javascript/after.js \
    --query '(call_expression function: (member_expression object: (identifier) @obj (#eq? @obj "console") property: (property_identifier) @prop (#eq? @prop "log")) arguments: (arguments) @args) @target' \
    --template 'logger.info${args}' \
    --in-place

echo "JavaScript transformation complete."

