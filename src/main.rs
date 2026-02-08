use anyhow::{Context, Result, anyhow};
use clap::Parser;
use graft::Transformer;
use graft::languages::LANGUAGES;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the source file (optional, defaults to stdin)
    file: Option<PathBuf>,

    /// Tree-sitter query
    #[arg(short, long)]
    query: Option<String>,

    /// Replacement template
    #[arg(short, long)]
    template: Option<String>,

    /// Edit file in-place (only applicable when a file is provided)
    #[arg(short, long)]
    in_place: bool,

    /// Language of the source code (required if reading from stdin)
    #[arg(short, long)]
    language: Option<String>,

    /// List supported languages and exit
    #[arg(long)]
    list_languages: bool,
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

    let query = cli
        .query
        .ok_or_else(|| anyhow!("Missing argument: --query <QUERY>"))?;
    let template = cli
        .template
        .ok_or_else(|| anyhow!("Missing argument: --template <TEMPLATE>"))?;

    let (source, lang_name) = match cli.file {
        Some(ref file_path) => {
            let source = fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read file: {:?}", file_path))?;
            
            let lang = if let Some(l) = cli.language.clone() {
                l
            } else {
                file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .ok_or_else(|| anyhow!("Could not detect file extension. Please specify language with --language"))?
                    .to_string()
            };
            (source, lang)
        }
        None => {
            if cli.in_place {
                return Err(anyhow!("--in-place is only supported when a file is provided"));
            }
            let lang = cli.language.ok_or_else(|| anyhow!("--language is required when reading from stdin"))?;
            
            let mut source = String::new();
            io::stdin().read_to_string(&mut source)
                .with_context(|| "Failed to read from stdin")?;
            (source, lang)
        }
    };

    let mut transformer = Transformer::new(source, &lang_name)
        .with_context(|| format!("Failed to initialize transformer for language '{}'", lang_name))?;

    transformer
        .apply(&query, &template)
        .with_context(|| "Failed to apply transformation")?;

    let new_source = transformer.get_source();

    if cli.in_place {
        if let Some(file_path) = cli.file {
            fs::write(&file_path, new_source)
                .with_context(|| format!("Failed to write to file: {:?}", file_path))?;
        }
    } else {
        print!("{}", new_source);
    }

    Ok(())
}