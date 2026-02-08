---
name: graft
description: Safe, structural code refactoring using Tree-sitter. Use when you need to perform complex code transformations based on AST structure rather than regex.
allowed-tools: [ graft, read_file, list_dir, run_shell_command, search_for_pattern ]
license: MIT
---

# Graft: Structural Code Refactoring

Perform safe, syntax-aware code transformations on source files. This skill enables you to refactor code based on its Abstract Syntax Tree (AST) structure rather than fragile text matching.

## Tool Usage

`graft` is a CLI tool. Execute it via `run_shell_command`.

Command: `graft [files...] --query <query> --template <template> [--in-place] [--language <lang>] [--json] [--rule-file <file>]`

## How to Construct Queries

1.  **S-Expressions**: Use standard Tree-sitter query syntax.
2.  **Target Node**: You **MUST** capture the node to be replaced with `@target`.
    *   Example: `(call_expression) @target`
3.  **Captures**: Use `@name` to capture sub-nodes for reuse in the template.
    *   Example: `(call_expression function: (identifier) @func arguments: (arguments) @args) @target`
4.  **Predicates**: Use `#eq?`, `#match?` to filter nodes.
    *   Example: `(#eq? @func "my_function")`

## How to Construct Templates

1.  **Placeholders**: Use `${name}` to insert the text of captured nodes.
    *   Example: `new_function(${args})`
2.  **Structure**: Write the replacement code as a valid code snippet.
3.  **Formatting**: `graft` preserves some formatting, but complex multi-line templates may need manual adjustment or a formatter run (e.g., `cargo fmt`) afterwards.

## Workflow

1.  **Draft**: Formulate the query and template based on the code structure.
2.  **Dry Run**: Run `graft` **without** the `--in-place` (or `-i`) flag to verify the transformation in stdout.
3.  **Apply**: Run with `--in-place` (or `-i`) to modify the file(s).
4.  **Verify**: Run tests or checks to ensure valid code.

## Advanced Usage

### Rule Files (TOML)
Define multiple rules in a persistent file. Graft will automatically filter rules by language and apply them by priority (highest first).
`graft src/ -f rules.toml -i`

### Batch Queries
Apply multiple transformations in sequence via CLI.
`graft file.rs -q 'q1' -t 't1' -q 'q2' -t 't2'`

### Batch Processing
Apply transformations to multiple files using glob patterns.
`graft "src/**/*.rs" -q '...' -t '...' -i`

## Examples

### 1. Rule File (`rules.toml`)
```toml
[[rules]]
language = "rust"
priority = 10
query = "(binary_expression) @target"
template = "add(${target})"
```
Command: `graft src/ -f rules.toml -i`

### 2. Rename Function `old(x)` -> `new(x)`
*   **Query**: `(call_expression function: (identifier) @n (#eq? @n "old") arguments: (arguments) @a) @target`
*   **template**: `new${a}`
