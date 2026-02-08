# Graft - Structural Code Transformer

`graft` is a CLI tool for safe, structural code transformation powered by [Tree-sitter](https://tree-sitter.github.io/). Unlike traditional regex-based find-and-replace tools, `graft` operates on the Abstract Syntax Tree (AST), ensuring that code modifications are syntactically valid and structure-aware.

## 🚀 Features

*   **AST-Based Transformation**: Edit code based on its structure, not just text patterns.
*   **Safe Rewrites**: Uses incremental parsing to validate syntax after every change.
*   **Bottom-Up Processing**: Preserves offset integrity for multiple replacements in a single file.
*   **Template Expansion**: Supports flexible template strings with captured variables (e.g., `${name}`).
*   **Multi-Language Support**: Supports a wide range of languages including Rust, JavaScript, Python, Go, and more.
*   **Batch Queries**: Apply multiple transformations in a single pass (like `sed -e ... -e ...`).
*   **Rule Files (TOML)**: Define reusable transformation rules in a persistent file with priority support.
*   **Batch Processing**: Apply transformations across multiple files using glob patterns (e.g., `src/**/*.rs`).
*   **Parallel Execution**: Processes multiple files concurrently for speed.
*   **Structured Output**: Optional JSON output for integration with other tools and agents.
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
graft [files...] --query <query> --template <template> [--in-place]
```

### Arguments

*   `[files...]`: Paths to source files or glob patterns (e.g., `src/**/*.rs`). Optional if reading from stdin (requires `--language`).
*   `--query, -q`: Tree-sitter S-expression query to match nodes. Can be specified multiple times.
*   `--template, -t`: Replacement string. Can be specified multiple times.
*   `--rule-file, -f`: Path to a TOML rule file.
*   `--in-place, -i`: Modify the file directly instead of printing to stdout.
*   `--language, -l`: Language of the source code.
*   `--json`: Output modifications in JSON format.
*   `--list-languages`: List all supported languages and their file extensions.

## 💡 Examples

### 1. Rule File (TOML)

Create a `rules.toml`:

```toml
[[rules]]
name = "add-to-pow"
language = "rust"
priority = 10
query = "(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target"
template = "pow(${l}, ${r})"
```

Apply it:

```bash
graft src/main.rs --rule-file rules.toml --in-place
```

### 2. Batch Queries

Chain multiple transformations in a single pass.

```bash
graft src/main.rs \
  --query '(binary_expression left: (_) @l operator: "+" right: (_) @r) @target' \
  --template 'add(${l}, ${r})' \
  --query '(call_expression function: (identifier) @n (#eq? @n "foo") arguments: (arguments) @a) @target' \
  --template 'bar${a}'
```

## 🤖 Agent Skills

Graft comes with a [Gemini CLI](https://github.com/google/gemini-cli) agent skill that enables your AI assistant to perform structural refactoring safely.

### Installation

To install the skill for your agent:

```bash
gemini skills install .skills/graft/graft.skill
```

Then reload your skills in the agent session:

```
/skills reload
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
*   `src/rules.rs`: Rule file loading logic.
*   `src/main.rs`: CLI entry point using `clap`.
*   `tests/integration_tests.rs`: Integration tests for various transformation scenarios.

## 📄 License

[MIT](LICENSE)