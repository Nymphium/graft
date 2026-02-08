#!/usr/bin/env bash
set -eu

TARGET=examples/javascript/target.js

# Build graft
cargo build --quiet

diff --color=auto -u "$TARGET" <(./target/debug/graft "$TARGET" --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' --template 'add(${l}, ${r})') || true

diff --color=auto -u "$TARGET" <(./target/debug/graft "$TARGET" --query '(call_expression function: (member_expression object: (identifier) @obj (#eq? @obj "console") property: (property_identifier) @prop (#eq? @prop "log")) arguments: (arguments) @args) @target' --template 'logger.info${args}') || true
