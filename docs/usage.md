# Usage Guide

Graft provides a flexible interface for structural code transformation.

## Basic Command

```bash
graft [FILES...] -q <QUERY> -t <TEMPLATE> [OPTIONS]
```

### Arguments

| Argument | Description |
|---|---|
| `FILES` | Positional paths to files or glob patterns (e.g., `"src/**/*.js"`). |
| `-q, --query` | Tree-sitter S-expression query. Must capture the replacement node as `@target`. |
| `-t, --template` | Replacement string. Use `${capture}` to insert matched node text. |
| `-i, --in-place` | Modify files directly. |
| `-l, --language` | Explicitly set the language (e.g., `rust`, `js`). |
| `--json` | Output transformation metadata in JSON format. |

## Constructing Queries

Graft uses standard Tree-sitter query syntax.

### 1. Simple Match
Capture an entire function call:
`--query '(call_expression) @target'`

### 2. Capture Sub-nodes
Capture arguments for reuse:
`--query '(call_expression function: (identifier) @name arguments: (arguments) @args) @target'`

### 3. Predicates
Filter matches using `#eq?` or `#match?`:
`--query '(call_expression function: (identifier) @n (#eq? @n "foo")) @target'`

## Templates

Templates define what the `@target` node should be replaced with.

- **Variables**: `${name}` inserts the text of the node captured as `@name`.
- **Newlines**: Use `\n` for multi-line replacements.
- **Spaces**: Indentation in templates is preserved.

## Chained Rewrites

You can specify multiple query/template pairs. They are applied sequentially:

```bash
graft file.rs \
  -q 'query1' -t 'template1' \
  -q 'query2' -t 'template2'
```

## Reading from Stdin

Graft can act as a filter in a pipeline. You must specify the language:

```bash
cat code.py | graft --language python -q '...' -t '...'
```

```