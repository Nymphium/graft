use anyhow::{anyhow, Context, Result};
use clap::Parser;
use glob::glob;
use graft::languages::LANGUAGES;
use graft::rules::RuleFile;
use graft::Transformer;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Graft is a safe, structural code transformation tool. It uses Tree-sitter to parse source code into an AST, allowing you to rewrite code based on its structure rather than fragile regex patterns.\n\nExamples:\n  # Rewrite binary expressions (a + b -> add(a, b))\n  graft src/main.rs -q \'(binary_expression left: (_) @l operator: \"+\" right: (_) @r) @target\' -t \'add(${l}, ${r})\'

  # Rename function calls across multiple files\n  graft \"src/**/*.rs\" -q \'(call_expression function: (identifier) @n (#eq? @n \"old\")) @target\' -t \'new\' -i

  # Use a rule file for complex transformations\n  graft src/ -f rules.toml -i"
)]
struct Cli {
    /// Path to the source file(s) or glob pattern(s). Optional if reading from stdin.
    files: Vec<String>,

    /// Tree-sitter query (S-expression). Capture the node to replace with `@target`.
    /// Can be specified multiple times for sequential transformations.
    #[arg(short, long, value_name = "QUERY")]
    query: Vec<String>,

    /// Replacement template. Use `${capture_name}` to insert matched nodes.
    /// Must match the number of queries provided.
    #[arg(short, long, value_name = "TEMPLATE")]
    template: Vec<String>,

    /// Path to a TOML rule file containing multiple queries and templates.
    #[arg(short = 'f', long, value_name = "FILE")]
    rule_file: Option<PathBuf>,

    /// Edit file(s) in-place instead of printing to stdout.
    #[arg(short, long)]
    in_place: bool,

    /// Language of the source code. Required if reading from stdin or if extension detection fails.
    #[arg(short, long, value_name = "LANG")]
    language: Option<String>,

    /// List all supported languages and their file extensions.
    #[arg(long)]
    list_languages: bool,

    /// Output detailed modification metadata in JSON format.
    #[arg(long)]
    json: bool,
}

#[derive(Serialize)]
struct JsonOutput {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    modifications: Option<Vec<graft::Modification>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

struct ResolvedRule {
    query: String,
    template: String,
    priority: i32,
}

fn language_matches(rule_lang: &str, target_lang: &str) -> bool {
    if rule_lang == target_lang {
        return true;
    }
    // Handle common aliases
    let aliases = [
        ("rust", "rs"),
        ("javascript", "js"),
        ("typescript", "ts"),
        ("python", "py"),
        ("markdown", "md"),
        ("makefile", "make"),
        ("makefile", "mk"),
        ("dockerfile", "docker"),
    ];
    for (l1, l2) in aliases {
        if (rule_lang == l1 && target_lang == l2) || (rule_lang == l2 && target_lang == l1) {
            return true;
        }
    }
    false
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_languages {
        println!("| Language | Extensions |");
        println!("|---|---|");
        for lang in LANGUAGES {
            let exts_str = lang
                .extensions
                .iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join(", ");
            println!("| {} | {} |", lang.name, exts_str);
        }
        return Ok(());
    }

    let rule_file = if let Some(ref path) = cli.rule_file {
        Some(RuleFile::load(path)?)
    } else {
        None
    };

    // If no rule file and no CLI query, error out (unless listing languages)
    if rule_file.is_none() && cli.query.is_empty() {
        return Err(anyhow!(
            "Either --rule-file or --query/--template must be provided. Use --help for more information."
        ));
    }

    if cli.query.len() != cli.template.len() {
        return Err(anyhow!(
            "Mismatch between number of queries ({}) and templates ({})",
            cli.query.len(),
            cli.template.len()
        ));
    }

    // Collect all files from arguments (expanding globs)
    let mut file_paths = Vec::new();
    for pattern in &cli.files {
        let entries =
            glob(pattern).with_context(|| format!("Failed to read glob pattern: {}", pattern))?;
        for entry in entries {
            match entry {
                Ok(path) => file_paths.push(path),
                Err(e) => eprintln!("Warning: failed to read glob entry: {}", e),
            }
        }
    }

    // If no files provided, read from stdin
    if file_paths.is_empty() {
        if cli.in_place {
            return Err(anyhow!(
                "--in-place is only supported when files are provided"
            ));
        }
        let lang_name = cli
            .language
            .ok_or_else(|| anyhow!("--language is required when reading from stdin"))?;

        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .with_context(|| "Failed to read from stdin")?;

        let mut transformer = Transformer::new(source, &lang_name).with_context(|| {
            format!(
                "Failed to initialize transformer for language '{}'",
                lang_name
            )
        })?;

        // Resolve rules for this language
        let mut rules = Vec::new();
        // 1. From CLI (priority 0)
        for (q, t) in cli.query.iter().zip(cli.template.iter()) {
            rules.push(ResolvedRule {
                query: q.clone(),
                template: t.clone(),
                priority: 0,
            });
        }
        // 2. From RuleFile (matching language)
        if let Some(ref rf) = rule_file {
            for r in &rf.rules {
                if language_matches(&r.language, &lang_name) {
                    rules.push(ResolvedRule {
                        query: r.query.clone(),
                        template: r.template.clone(),
                        priority: r.priority,
                    });
                }
            }
        }

        // Sort by priority descending
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut all_modifications = Vec::new();
        for r in rules {
            let mut mods = transformer
                .apply(&r.query, &r.template)
                .with_context(|| "Failed to apply transformation")?;
            all_modifications.append(&mut mods);
        }

        if cli.json {
            let output = JsonOutput {
                status: "success".to_string(),
                modifications: Some(all_modifications),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            print!("{}", transformer.get_source());
        }
        return Ok(());
    }

    // Parallel processing for files
    let all_modifications_shared = Arc::new(Mutex::new(Vec::new()));
    let has_error = Arc::new(Mutex::new(false));

    file_paths.par_iter().for_each(|file_path| {
        let process_file = || -> Result<()> {
            let source = fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read file: {:?}", file_path))?;

            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .ok_or_else(|| anyhow!("Could not detect file extension for {:?}", file_path))?;

            // Prefer explicit language if provided, otherwise detect
            let lang_name = cli.language.clone().unwrap_or_else(|| ext.to_string());

            let mut transformer = Transformer::new(source, &lang_name).with_context(|| {
                format!("Failed to initialize transformer for file {:?}", file_path)
            })?;

            // Resolve rules for this file's language
            let mut rules = Vec::new();
            for (q, t) in cli.query.iter().zip(cli.template.iter()) {
                rules.push(ResolvedRule {
                    query: q.clone(),
                    template: t.clone(),
                    priority: 0,
                });
            }
            if let Some(ref rf) = rule_file {
                for r in &rf.rules {
                    // Match language name or extension
                    if language_matches(&r.language, &lang_name) {
                        rules.push(ResolvedRule {
                            query: r.query.clone(),
                            template: r.template.clone(),
                            priority: r.priority,
                        });
                    }
                }
            }
            rules.sort_by(|a, b| b.priority.cmp(&a.priority));

            let mut file_modifications = Vec::new();
            for r in rules {
                let mut mods = transformer.apply(&r.query, &r.template)?;
                file_modifications.append(&mut mods);
            }

            if cli.json {
                let mut mods = all_modifications_shared.lock().unwrap();
                for mut m in file_modifications {
                    m.filename = Some(file_path.to_string_lossy().to_string());
                    mods.push(m);
                }
            } else {
                let new_source = transformer.get_source();
                if cli.in_place {
                    fs::write(file_path, new_source)
                        .with_context(|| format!("Failed to write to file: {:?}", file_path))?;
                } else {
                    let mut stdout = io::stdout().lock();
                    use std::io::Write;
                    write!(stdout, "{}", new_source).ok();
                }
            }
            Ok(())
        };

        if let Err(e) = process_file() {
            eprintln!("Error processing {:?}: {:?}", file_path, e);
            let mut err_flag = has_error.lock().unwrap();
            *err_flag = true;
        }
    });

    if cli.json {
        let error_occurred = *has_error.lock().unwrap();
        let modifications = all_modifications_shared.lock().unwrap().clone();

        let output = JsonOutput {
            status: if error_occurred {
                "partial_error".to_string()
            } else {
                "success".to_string()
            },
            modifications: Some(modifications),
            error: if error_occurred {
                Some("One or more files failed to process".to_string())
            } else {
                None
            },
        };
        println!("{}", serde_json::to_string_pretty(&output)?);

        if error_occurred {
            std::process::exit(1);
        }
    } else if *has_error.lock().unwrap() {
        std::process::exit(1);
    }

    Ok(())
}