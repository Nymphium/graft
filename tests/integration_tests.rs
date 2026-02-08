use anyhow::Result;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_batch_processing_files() -> Result<()> {
    let dir = tempdir()?;
    let file_a = dir.path().join("a.rs");
    let file_b = dir.path().join("b.rs");

    fs::write(&file_a, "fn main() { let x = 1 + 2; }")?;
    fs::write(&file_b, "fn test() { let y = 3 + 4; }")?;

    let cli = graft::cli::Cli {
        files: vec![dir.path().join("*.rs").to_string_lossy().to_string()],
        query: vec![
            "(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target".to_string(),
        ],
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
    rules_content.push_str(
        "query = \"(binary_expression left: (_) @l operator: \\\"+\\\" right: (_) @r) @target\"\n",
    );
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
