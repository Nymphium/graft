#!/usr/bin/env bash
set -eu

TARGET=examples/rust/target.rs

# Build graft
cargo build --quiet

diff -u "$TARGET" <(./target/debug/graft "$TARGET" --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' --template 'add(${l}, ${r})') || true

diff -u "$TARGET" <(./target/debug/graft "$TARGET" --query '(call_expression function: (identifier) @n (#eq? @n "foo") arguments: (arguments) @a) @target' --template 'bar${a}') || true