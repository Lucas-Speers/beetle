#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;
use args::Cli;

use anyhow::{anyhow, Context, Ok, Result};
use lex::{tokenize, Token};

mod args;
mod files;
mod lex;
mod ast;
mod interpreter;

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{}", cli.file.display());
    
    let mut all_tokens: Vec<Token> = Vec::new();
    
    // hold a list of all files that need to be read from
    let mut files_to_read: Vec<PathBuf> = Vec::new();
    match cli.file.to_str() {
        Some(x) => files_to_read.push(PathBuf::from(x)),
        None => return Err(anyhow!("Filename not propper unicode")),
    }
    
    let mut file_index = 0;
    while files_to_read.len() > file_index {
        let file = files::read_full_file(&files_to_read[file_index])?;
        let mut tokens = tokenize(&file, files_to_read[file_index].to_str().context("Filename is not proper unicode")?);

        all_tokens.append(&mut tokens);
    
        // let mut search_for_import = tokens.iter().peekable();
        // while let Some(t) = search_for_import.next() {
        //     if let Token::Import = t {
        //         if let Some(Token::StringLiteral(x)) = search_for_import.peek() {
        //             let path = PathBuf::from(x);
        //             if !files_to_read.contains(&path) {
        //                 files_to_read.push(path);
        //             }
        //         }
        //     }
        // }
        
        file_index += 1;
    }

    let (paths, functions) = ast::ast_from_tokens(all_tokens);

    println!("test");

    let mut code_state = interpreter::CodeState::new();
    code_state.run_function("main".to_string(), Vec::new(), &functions);

    Ok(())
}
