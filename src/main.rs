use anyhow::{Context, Result, anyhow};
use clap::Parser;
use graft::Transformer;
use graft::languages::LANGUAGES;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the source file
    file: Option<PathBuf>,

    /// Tree-sitter query
    #[arg(short, long)]
    query: Option<String>,

    /// Replacement template
    #[arg(short, long)]
    template: Option<String>,

    /// Edit file in-place
    #[arg(short, long)]
    in_place: bool,

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

    // Manually enforce required arguments if not listing languages
    let file_path = cli
        .file
        .ok_or_else(|| anyhow!("Missing argument: <FILE>"))?;
    let query = cli
        .query
        .ok_or_else(|| anyhow!("Missing argument: --query <QUERY>"))?;
    let template = cli
        .template
        .ok_or_else(|| anyhow!("Missing argument: --template <TEMPLATE>"))?;

    let source = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow!("Could not detect file extension"))?;

    let mut transformer = Transformer::new(source, ext)
        .with_context(|| format!("Failed to initialize transformer for file extension '.{}'", ext))?;
        
    transformer.apply(&query, &template)
        .with_context(|| "Failed to apply transformation")?;

    let new_source = transformer.get_source();

    if cli.in_place {
        fs::write(&file_path, new_source)
            .with_context(|| format!("Failed to write to file: {:?}", file_path))?;
    } else {
        print!("{}", new_source);
    }

    Ok(())
}