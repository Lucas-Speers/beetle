use clap::Parser;
use args::Cli;

use anyhow::{Ok, Result};
use lex::tokenize;

mod args;
mod files;
mod lex;

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{}", cli.file.display());

    let file = files::read_full_file(&cli.file)?;

    let tokens = tokenize(&file)?;

    println!("{:?}", tokens);

    Ok(())
}