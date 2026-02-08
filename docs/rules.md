# Rule Files (TOML)

For complex refactorings or recurring tasks, you can define rules in a TOML file.

## Rule Structure

A rule file consists of an array of `rules`:

```toml
[[rules]]
name = "upgrade-api"
language = "rust"
priority = 10
query = """
(call_expression
  function: (identifier) @name (#eq? @name "old_api")
  arguments: (arguments) @args) @target
"""
template = "new_api${args}"
```

### Fields

- `name` (Optional): A descriptive name for the rule.
- `language`: The language this rule applies to. Graft uses common aliases (e.g., `rs` matches `rust`).
- `priority`: Integer. Higher priority rules are applied first in a single pass.
- `query`: The Tree-sitter S-expression.
- `template`: The replacement template.

## Using a Rule File

Pass the `-f` or `--rule-file` flag:

```bash
graft src/ -f my_rules.toml --in-place
```

Graft will:
1. Load all rules from the file.
2. Filter rules that match the language of each target file.
3. Sort matching rules by `priority` (descending).
4. Apply them sequentially to the file.
