use clap::Parser;
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;
use graft::Transformer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the source file
    file: PathBuf,

    /// Tree-sitter query
    #[arg(short, long)]
    query: String,

    /// Replacement template
    #[arg(short, long)]
    template: String,

    /// Edit file in-place
    #[arg(short, long)]
    in_place: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let source = fs::read_to_string(&cli.file)
        .with_context(|| format!("Failed to read file: {:?}", cli.file))?;
    
    let ext = cli.file.extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow!("Could not detect file extension"))?;

    let mut transformer = Transformer::new(source, ext)?;
    transformer.apply(&cli.query, &cli.template)?;

    let new_source = transformer.get_source();

    if cli.in_place {
        fs::write(&cli.file, new_source)
            .with_context(|| format!("Failed to write to file: {:?}", cli.file))?;
    } else {
        print!("{}", new_source);
    }

    Ok(())
}