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
    
    let mut all_tokens: Vec<Token> = Vec::new();
    
    // hold a list of all files that need to be read from
    let mut files_to_read: Vec<String> = Vec::new();
    match cli.file.to_str() {
        Some(x) => files_to_read.push(x.to_owned()),
        None => return Err(MainError::UnicodeError),
    }
    
    let mut file_index = 0;
    while files_to_read.len() > file_index {
        let file = files::read_full_file(&PathBuf::from(files_to_read[file_index].clone()))?;
        let mut tokens = Tokenizer::new(&file, file_index).generate();

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

    let (paths, functions) = ast::ASTParser::new(all_tokens).parse_all();

    std::thread::Builder::new().stack_size(8 * 1024 * 1024).spawn(||{
        let mut code_state = interpreter::CodeState::new(functions);
        let result = code_state.run_function("main".to_string(), &Vec::new(), (0,0,0));
        match result {
            Ok(_) => (),
            Err(x) => println!("{x}"),
        }
    }).unwrap().join().unwrap();
    

    Ok(())
}
