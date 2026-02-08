use anyhow::Result;
use graft::graft::Transformer;

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
