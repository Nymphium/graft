use anyhow::Result;
use graft::graft::Transformer;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_binary_expression_rewrite() -> Result<()> {
    let source = "fn main() { let x = a + b; }";
    let mut transformer = Transformer::new(source.to_string(), "rust")?;

    let query = "(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target";
    let template = "pow(${l}, ${r})";

    let _ = transformer.apply(query, template)?;
    let output = transformer.get_source();

    assert_eq!(output, "fn main() { let x = pow(a, b); }");
    Ok(())
}

#[test]
fn test_function_call_rewrite() -> Result<()> {
    let source = "fn main() { foo(1, 2); }";
    let mut transformer = Transformer::new(source.to_string(), "rust")?;

    let query = "(call_expression function: (identifier) @name (#eq? @name \"foo\") arguments: (arguments) @args) @target";
    let template = "bar${args}";

    let _ = transformer.apply(query, template)?;
    let output = transformer.get_source();

    assert_eq!(output, "fn main() { bar(1, 2); }");
    Ok(())
}

#[test]
fn test_insertion_before_statement() -> Result<()> {
    let source = "fn main() { process(); }";
    let mut transformer = Transformer::new(source.to_string(), "rust")?;

    let query = "(expression_statement (call_expression function: (identifier) @name (#eq? @name \"process\"))) @target";
    let template = "log(\"start\");\n    process();";

    let _ = transformer.apply(query, template)?;
    let output = transformer.get_source();

    // Note: indentation might be tricky, but we provided explicit spaces in template
    assert_eq!(output, "fn main() { log(\"start\");\n    process(); }");
    Ok(())
}

#[test]
fn test_multiple_occurrences_bottom_up() -> Result<()> {
    let source = "fn main() { let a = 1 + 2; let b = 3 + 4; }";
    let mut transformer = Transformer::new(source.to_string(), "rust")?;

    let query = "(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target";
    let template = "add(${l}, ${r})";

    let _ = transformer.apply(query, template)?;
    let output = transformer.get_source();

    assert_eq!(
        output,
        "fn main() { let a = add(1, 2); let b = add(3, 4); }"
    );
    Ok(())
}

#[test]
fn test_syntax_error_detection() {
    let source = "fn main() { return; }";
    let mut transformer = Transformer::new(source.to_string(), "rust").unwrap();

    let query = "(return_expression) @target";
    let template = "return 1 + ;"; // Invalid syntax

    let result = transformer.apply(query, template);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Transformation resulted in syntax error"));
    assert!(err_msg.contains("return 1 + ;"));
}

#[test]
fn test_multiline_error_context() {
    let source = "fn main() {\n    let x = 1;\n    process();\n}";
    let mut transformer = Transformer::new(source.to_string(), "rust").unwrap();

    let query = "(expression_statement (call_expression function: (identifier) @n (#eq? @n \"process\"))) @target";
    let template = "if ( {"; // Extremely broken syntax

    let result = transformer.apply(query, template);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());

    assert!(err_msg.contains("Transformation resulted in syntax error"));
    assert!(err_msg.contains("if ( {"));
}

#[test]
fn test_go_template_validation() {
    let source = "1 + 2";
    let mut transformer = Transformer::new(source.to_string(), "go").unwrap();

    let query = "(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target";
    let template = "add(${l} ${r})"; // Missing comma in Go

    let result = transformer.apply(query, template);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Transformation resulted in syntax error"));
    assert!(err_msg.contains("add(1 2)"));
}

#[test]
fn test_batch_queries_sequential() -> Result<()> {
    let source = "fn main() { let x = 1 + 2; let y = foo(x); }";
    let mut transformer = Transformer::new(source.to_string(), "rust")?;

    // Step 1: 1+2 -> add(1,2)
    let _ = transformer.apply(
        "(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target",
        "add(${l}, ${r})",
    )?;

    // Step 2: foo(x) -> bar(x)
    let _ = transformer.apply(
        "(call_expression function: (identifier) @n (#eq? @n \"foo\") arguments: (arguments) @a) @target",
        "bar${a}",
    )?;

    let output = transformer.get_source();
    assert_eq!(output, "fn main() { let x = add(1, 2); let y = bar(x); }");
    Ok(())
}

#[test]
fn test_batch_processing_files() -> Result<()> {
    let dir = tempdir()?;
    let file_a = dir.path().join("a.rs");
    let file_b = dir.path().join("b.rs");

    fs::write(&file_a, "fn main() { let x = 1 + 2; }")?;
    fs::write(&file_b, "fn test() { let y = 3 + 4; }")?;

    let cli = graft::cli::Cli {
        files: vec![dir.path().join("*.rs").to_string_lossy().to_string()],
        query: vec!["(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target"
            .to_string()],
        template: vec!["add(${l}, ${r})".to_string()],
        rule_file: None,
        in_place: true,
        language: None,
        list_languages: false,
        json: false,
    };

    graft::cli::run_with_args(cli)?;

    let content_a = fs::read_to_string(file_a)?;
    let content_b = fs::read_to_string(file_b)?;

    assert!(content_a.contains("add(1, 2)"));
    assert!(content_b.contains("add(3, 4)"));

    Ok(())
}

#[test]
fn test_rule_file_loading() -> Result<()> {
    let dir = tempdir()?;
    let target_file = dir.path().join("target.rs");
    let rules_file = dir.path().join("rules.toml");

    fs::write(&target_file, "fn main() { let x = 1 + 2; let y = foo(x); }")?;

    let mut rules_content = String::new();
    rules_content.push_str("[[rules]]\n");
    rules_content.push_str("name = \"add-to-pow\"\n");
    rules_content.push_str("language = \"rust\"\n");
    rules_content.push_str("priority = 10\n");
    rules_content.push_str("query = \"(binary_expression left: (_) @l operator: \\\"+\\\" right: (_) @r) @target\"\n");
    rules_content.push_str("template = \"pow(${l}, ${r})\"\n\n");
    rules_content.push_str("[[rules]]\n");
    rules_content.push_str("name = \"rename-foo\"\n");
    rules_content.push_str("language = \"rust\"\n");
    rules_content.push_str("priority = 5\n");
    rules_content.push_str("query = \"(call_expression function: (identifier) @n (#eq? @n \\\"foo\\\") arguments: (arguments) @a) @target\"\n");
    rules_content.push_str("template = \"bar${a}\"\n");

    fs::write(&rules_file, rules_content)?;

    let cli = graft::cli::Cli {
        files: vec![target_file.to_string_lossy().to_string()],
        query: vec![],
        template: vec![],
        rule_file: Some(rules_file),
        in_place: true,
        language: None,
        list_languages: false,
        json: false,
    };

    graft::cli::run_with_args(cli)?;

    let output = fs::read_to_string(target_file)?;
    assert_eq!(output, "fn main() { let x = pow(1, 2); let y = bar(x); }");

    Ok(())
}
