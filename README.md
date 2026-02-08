# Graft - Structural Code Transformer

`graft` is a CLI tool for safe, structural code transformation powered by [Tree-sitter](https://tree-sitter.github.io/). Unlike traditional regex-based find-and-replace tools, `graft` operates on the Abstract Syntax Tree (AST), ensuring that code modifications are syntactically valid and structure-aware.

## 🚀 Features

*   **AST-Based Transformation**: Edit code based on its structure, not just text patterns.
*   **Safe Rewrites**: Uses incremental parsing to validate syntax after every change.
*   **Bottom-Up Processing**: Preserves offset integrity for multiple replacements in a single file.
*   **Template Expansion**: Supports flexible template strings with captured variables (e.g., `${name}`).
*   **Multi-Language Support**: Supports a wide range of languages including Rust, JavaScript, Python, Go, and more.
*   **Nix-First**: Reproducible development environment with Nix and direnv.

## 🛠 Prerequisites

*   **Rust**: v1.93.0+ (Edition 2024)
*   **Nix** (Optional but recommended): For reproducible builds using `flake.nix`.

## 📦 Installation

### Using Cargo

```bash
cargo install --path .
```


<!-- not supported yet
### Using Nix

```bash
nix build .
```
-->

## 📖 Usage

Basic command structure:

```bash
graft <file> --query <query> --template <template> [--in-place]
```

### Arguments

*   `<file>`: Path to the source file to transform.
*   `--query, -q`: Tree-sitter S-expression query to match nodes. Use `@target` to specify the node to replace.
*   `--template, -t`: Replacement string. Use `${capture_name}` for captured nodes.
*   `--in-place, -i`: Modify the file directly instead of printing to stdout.
*   `--list-languages`: List all supported languages and their file extensions.

## 💡 Examples

### 1. Rewrite Binary Expressions (`a + b` → `add(a, b)`)

Rewrite addition operations into function calls.

```bash
graft src/main.rs \
  --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
  --template 'add(${l}, ${r})'
```

### 2. Rename Function Calls (`foo(x)` → `bar(x)`)

Rename specific function calls while keeping arguments intact.

```bash
graft src/main.rs \
  --query '(call_expression function: (identifier) @name (#eq? @name "foo") arguments: (arguments) @args) @target' \
  --template 'bar${args}'
```

### 3. Insert Logging Statement

Insert a log statement before a specific function call.

```bash
graft src/main.rs \
  --query '(expression_statement (call_expression function: (identifier) @name (#eq? @name "process"))) @target' \
  --template 'log("start");\n    process();'
```

## 🌐 Supported Languages

Graft supports a variety of languages. You can list them using:

```bash
graft --list-languages
```

For a full list of supported languages and extensions, see [SUPPORTED_LANGUAGES.md](SUPPORTED_LANGUAGES.md).

## 🧪 Development

### Running Tests

Run the test suite to verify core functionality:

```bash
cargo test
```

### Project Structure

*   `src/lib.rs`: Core transformation logic (`Transformer` struct).
*   `src/languages.rs`: Language definitions and mappings.
*   `src/main.rs`: CLI entry point using `clap`.
*   `tests/integration_tests.rs`: Integration tests for various transformation scenarios.

## 📄 License

[MIT](LICENSE)
