#!/bin/bash
set -e
# Navigate to project root
cd "$(dirname "$0")/../.."

# Build graft
cargo build --quiet

# Create after.go from before.go
cp examples/go/before.go examples/go/after.go

# Apply binary expression rewrite
# Go: (binary_expression left: (_) @l operator: "+" right: (_) @r)
./target/debug/graft examples/go/after.go \
    --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
    --template 'add(${l}, ${r})' \
    --in-place

# Rewrite fmt.Println -> logger.Info
# (call_expression function: (selector_expression operand: (identifier) @pkg field: (field_identifier) @func) arguments: (argument_list) @args)
./target/debug/graft examples/go/after.go \
    --query '(call_expression function: (selector_expression operand: (identifier) @pkg (#eq? @pkg "fmt") field: (field_identifier) @func (#eq? @func "Println")) arguments: (argument_list) @args) @target' \
    --template 'logger.Info${args}' \
    --in-place

echo "Go transformation complete."

