# Future Plans for Graft

This document outlines planned features and improvements for `graft`, specifically targeting enhanced usability for AI coding agents and automated refactoring workflows.

## 🤖 Agent-Centric Features

### 1. Structured Output (JSON) [DONE]
*   **Goal**: Provide machine-readable output for easier parsing by agents.
*   **Details**:
    *   Flag: `--json`
    *   Output format:
        ```json
        {
          "status": "success",
          "modifications": [
            {
              "file": "src/main.rs",
              "start_byte": 10,
              "end_byte": 20,
              "replacement": "new_code",
              "original": "old_code"
            }
          ]
        }
        ```
    *   Include syntax error details in a structured format if validation fails.

### 2. Context-Aware Query Generation
*   **Goal**: Help agents construct correct queries.
*   **Details**:
    *   Command: `graft --inspect <file> --at-line <line>`
    *   Output: The S-expression of the AST node at the specified line/column. This allows an agent to "see" the structure it needs to match without guessing.

### 3. Batch Queries (Multiple Rewrites) [DONE]
*   **Goal**: Apply multiple transformations in a single pass.
*   **Details**:
    *   Allow multiple `-q` and `-t` flags.
    *   Apply them sequentially: `Source -> T1 -> Source' -> T2 -> Source''`.
    *   Example: `graft file.rs -q '(foo)' -t 'bar' -q '(baz)' -t 'qux'`

### 4. Batch Processing [DONE]
*   **Goal**: Apply transformations across multiple files efficiently.
*   **Details**:
    *   Support glob patterns: `graft "src/**/*.rs" ...`
    *   Parallel processing for speed.

### 5. Query Library / Presets
*   **Goal**: Reuse common refactoring patterns.
*   **Details**:
    *   Load queries from a file or a built-in library.
    *   Example: `graft --preset rust/unwrap-to-expect ...`

## 🛠 Core Improvements

*   **Auto-Formatting**: Integrate with language formatters (e.g., `rustfmt`, `prettier`) to clean up the output after structural changes.
*   **Cross-File Refactoring**: (Ambitious) Support renaming symbols across file boundaries.

## 📦 Distribution & Integration

*   **WASM Build**: Compile `graft` to WASM for running in browser-based agents or restricted environments.
*   **LSP Integration**: Expose `graft` capabilities via a Language Server Protocol extension for IDEs.