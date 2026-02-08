use anyhow::{Context, Result, anyhow};
use clap::Parser;
use glob::glob;
use graft::Transformer;
use graft::languages::LANGUAGES;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::io::{self, Read};
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the source file(s) or glob pattern(s)
    files: Vec<String>,

    /// Tree-sitter query (can be specified multiple times)
    #[arg(short, long)]
    query: Vec<String>,

    /// Replacement template (can be specified multiple times)
    #[arg(short, long)]
    template: Vec<String>,

    /// Edit file in-place (only applicable when files are provided)
    #[arg(short, long)]
    in_place: bool,

    /// Language of the source code (required if reading from stdin)
    #[arg(short, long)]
    language: Option<String>,

    /// List supported languages and exit
    #[arg(long)]
    list_languages: bool,

    /// Output modifications in JSON format
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_languages {
        println!("| Language | Extensions |");
        println!("|---|---|");
        for lang in LANGUAGES {
            let exts_str = lang
                .extensions
                .iter()
                .map(|s| format!("`.{}`", s))
                .collect::<Vec<_>>()
                .join(", ");
            println!("| {} | {} |", lang.name, exts_str);
        }
        return Ok(());
    }

    if cli.query.is_empty() {
        return Err(anyhow!("Missing argument: --query <QUERY>"));
    }
    if cli.template.is_empty() {
        return Err(anyhow!("Missing argument: --template <TEMPLATE>"));
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

        let mut all_modifications = Vec::new();
        for (q, t) in cli.query.iter().zip(cli.template.iter()) {
            let mut mods = transformer.apply(q, t).with_context(|| "Failed to apply transformation")?;
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
    let all_modifications = Arc::new(Mutex::new(Vec::new()));
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

            let mut file_modifications = Vec::new();
            for (q, t) in cli.query.iter().zip(cli.template.iter()) {
                let mut mods = transformer.apply(q, t)?;
                file_modifications.append(&mut mods);
            }

            if cli.json {
                let mut mods = all_modifications.lock().unwrap();
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
                    // Avoid interleaved output
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
        let modifications = all_modifications.lock().unwrap().clone();

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