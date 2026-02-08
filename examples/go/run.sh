#!/usr/bin/env bash
set -eu

TARGET=examples/go/target.go

# Build graft
cargo build --quiet

diff --color=auto -u "$TARGET" <(./target/debug/graft "$TARGET" --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' --template 'add(${l}, ${r})') || true

diff --color=auto -u "$TARGET" <(./target/debug/graft "$TARGET" --query '(call_expression function: (selector_expression operand: (identifier) @pkg (#eq? @pkg "fmt") field: (field_identifier) @func (#eq? @func "Println")) arguments: (argument_list) @args) @target' --template 'logger.Info${args}') || true
