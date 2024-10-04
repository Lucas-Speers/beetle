use clap::Parser;
use args::Cli;

use anyhow::{Ok, Result};

mod args;
mod files;

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{}", cli.file.display());

    println!("{:?}", files::read_full_file(&cli.file));

    Ok(())
}