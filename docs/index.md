# Graft

**Graft** is a robust CLI tool for safe, structural code transformation powered by [Tree-sitter](https://tree-sitter.github.io/).

Unlike traditional regex-based find-and-replace tools, Graft operates on the Abstract Syntax Tree (AST). This ensures that your code modifications are syntactically valid and aware of the code's actual structure.

## Key Features

- **AST-Based**: Edit code based on structure, not just text patterns.
- **Safe Rewrites**: Incremental parsing validates syntax after every change.
- **Multi-Language**: Support for 22+ languages (Rust, JS, Python, Go, etc.).
- **Batch Processing**: Parallel execution across multiple files using glob patterns.
- **Chained Rewrites**: Apply multiple transformations in a single pass.
- **Rule Files**: Persistent TOML-based rules with priority support.

## Quick Start

Transform addition into a function call in a Rust file:

```bash
graft src/main.rs \
  --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
  --template 'add(${l}, ${r})'
```

## Navigation

- [Installation](installation.md)
- [Basic Usage](usage.md)
- [Rule Files (TOML)](rules.md)
- [Supported Languages](languages.md)

