#![allow(unused)]

use std::path::{Path, PathBuf};

use ast::FunctionDecleration;
use clap::Parser;
use args::Cli;

use lex::{Token, TokenType, Tokenizer};

mod args;
mod files;
mod lex;
mod ast;
mod interpreter;

fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    
    let mut all_functions: Vec<FunctionDecleration> = Vec::new();

    let orig_path;
    
    // hold a list of all files that need to be read from
    let mut files_to_read: Vec<String> = Vec::new();
    match cli.file.to_str() {
        Some(x) => {
            orig_path = PathBuf::from(x);
            files_to_read.push(orig_path.file_name().unwrap().to_str().unwrap().to_string());
        },
        None => return Err(()),
    }
    
    let mut file_index = 0;
    while files_to_read.len() > file_index {
        let file = files::read_full_file(&orig_path.parent().unwrap().join(&PathBuf::from(files_to_read[file_index].clone())))?;
        let mut tokens = Tokenizer::new(&file, file_index).generate();

        let (mut paths, mut functions) = ast::ASTParser::new(tokens).parse_all();
        files_to_read.append(&mut paths);
        all_functions.append(&mut functions);
        
        file_index += 1;
    }


    std::thread::Builder::new().stack_size(8 * 1024 * 1024).spawn(||{
        let mut code_state = interpreter::CodeState::new(all_functions);
        let result = code_state.run_function("main".to_string(), &Vec::new(), (0,0,0));
        match result {
            Ok(_) => (),
            Err(x) => println!("{x}"),
        }
    }).unwrap().join().unwrap();
    

    Ok(())
}
