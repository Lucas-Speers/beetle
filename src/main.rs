#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;
use args::Cli;

use lex::{Tokenizer, Token};

mod args;
mod files;
mod lex;
mod ast;
mod interpreter;

#[derive(Debug)]
enum MainError {
    UnicodeError,
    FileNotFound,
}

fn main() -> Result<(), MainError> {
    let cli = Cli::parse();

    println!("{}", cli.file.display());
    
    let mut all_tokens: Vec<Token> = Vec::new();
    
    // hold a list of all files that need to be read from
    let mut files_to_read: Vec<PathBuf> = Vec::new();
    match cli.file.to_str() {
        Some(x) => files_to_read.push(PathBuf::from(x)),
        None => return Err(MainError::UnicodeError),
    }
    
    let mut file_index = 0;
    while files_to_read.len() > file_index {
        let file = files::read_full_file(&files_to_read[file_index])?;
        let mut tokens = Tokenizer::new(&file, files_to_read[file_index].to_str().unwrap()).generate();

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

    for t in &all_tokens {
        println!("{t:?}");
    }

    let (paths, functions) = ast::ASTParser::new(all_tokens).parse_all();

    let mut code_state = interpreter::CodeState::new(functions);
    let result = code_state.run_function("main".to_string(), &Vec::new());
    
    match result {
        Ok(_) => (),
        Err(x) => println!("{x}"),
    }

    Ok(())
}
