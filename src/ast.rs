use std::{iter::Peekable, path::PathBuf};

use crate::lex::{Token, TokenType::{self, *}};

#[derive(Debug, Clone)]
pub struct FunctionDecleration {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<ASTree>,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    args: Vec<ASTValue>,
}

#[derive(Debug, Clone)]
pub enum ASTree {
    Let {
        variable: String,
        value: ASTValue,
    },
    Assign {
        variable: String,
        value: ASTValue,
    },
    Function {
        name: String,
        args: Vec<ASTValue>,
    },
}

#[derive(Debug, Clone)]
pub enum ASTValue {
    Number {
        whole: u64,
        decimal: Option<u64>,
        negative: bool,
    },
    String {
        content: String,
    },
    Function {
        name: String,
        args: Vec<ASTValue>,
    },
    Variable {
        name: String,
    }
}

struct TokenIter {
    vec: Vec<Token>,
    index: usize,
    last: Token,
}

impl TokenIter {
    fn new(vec: Vec<Token>) -> TokenIter {
        let last = vec[0].clone();
        TokenIter { vec, index: 0, last }
    }
    fn next(&mut self) -> TokenType {
        if self.index == self.vec.len() {
            parse_error("Reached EOF", &self);
        }
        else {
            let value = self.vec[self.index].clone();
            self.last = value.clone();
            self.index += 1;
            value.token_type
        }
    }
    fn peek(&mut self) -> Option<TokenType> {
        if self.index == self.vec.len() {None}
        else {
            Some(self.vec[self.index].token_type.clone())
        }
    }
    fn peek_expect(&mut self) -> TokenType {
        if self.index == self.vec.len() {
            parse_error("Reached EOF", &self);
        }
        else {
            self.vec[self.index].token_type.clone()
        }
    }
    fn peek_ahead_expect(&mut self) -> Token {
        if self.index >= self.vec.len() {
            parse_error("Reached EOF", &self);
        }
        else {
            self.vec[self.index+1].clone()
        }
    }
    fn has_more(&self) -> bool {
        self.index != self.vec.len()
    }
}

pub fn ast_from_tokens(tokens_vec: Vec<Token>) -> (Vec<PathBuf>, Vec<FunctionDecleration>) {
    let mut tokens = TokenIter::new(tokens_vec);
    parse_all(&mut tokens)
}

fn parse_all(tokens: &mut TokenIter) -> (Vec<PathBuf>, Vec<FunctionDecleration>) {
    println!("parse_all");
    let imports = parse_imports(tokens);
    let functions = parse_functions(tokens);
    dbg!(imports, functions)
}

fn parse_imports(tokens: &mut TokenIter) -> Vec<PathBuf> {
    println!("parse_imports");
    let mut imported_files = Vec::new();
    loop { // check all the imports
        if let Some(token) = tokens.peek() {
            if let Identifier(name) = token {
                if name == "import" {
                    tokens.next();
                    if let StringToken(content) = tokens.next() {
                        imported_files.push(PathBuf::from(content));
                        continue;
                    } else {unreachable!();}
                }
            }
        }
        break;
    }
    imported_files
}

fn parse_functions(tokens: &mut TokenIter) -> Vec<FunctionDecleration> {
    println!("parse_function");
    let mut functions = Vec::new();
    while tokens.has_more() {
        if let Identifier(name) = tokens.next() {
            if name != "func" {parse_error("Expected `func` keyword", tokens)}
            functions.push(parse_function_decleration(tokens));
        } else {
            parse_error("Expected function decleration", tokens);
        }
    }
    functions
}

fn parse_function_decleration(tokens: &mut TokenIter) -> FunctionDecleration {
    println!("parse_function_decleration");
    if let Identifier(name) = tokens.next() {
        if tokens.next() != LeftParren {parse_error("Expected `(` after function name", tokens)}
        let args = parse_function_params(tokens);
        if tokens.next() != LeftCurly {parse_error("Expected `{` for the function code block", tokens)}
        let body = parse_fuction_body(tokens);
        return FunctionDecleration { name, args, body };
    }
    else {
        parse_error("Expected function name", tokens);
    }
}

fn parse_function_params(tokens: &mut TokenIter) -> Vec<String> {
    println!("parse_function_params");
    let mut params = Vec::new();
    loop {
        if tokens.peek_expect() == RightParren {
            tokens.next();
            return params;
        }
        params.push(parse_full_varriable(tokens));
        if tokens.peek_expect() == Comma {
            tokens.next();
        }
    }
}

fn parse_full_varriable(tokens: &mut TokenIter) -> String {
    println!("parse_full_variable");
    if let Identifier(name) = tokens.next() {
        return name;
    }
    parse_error("Expected variable name", tokens);
}

fn parse_type(tokens: &mut TokenIter) -> String {
    println!("parse_type");
    if let Identifier(name) = tokens.next() {
        return name;
    }
    parse_error("Expected type", tokens);
}

fn parse_fuction_body(tokens: &mut TokenIter) -> Vec<ASTree> {
    println!("parse_fuction_body");
    let mut expresions = Vec::new();

    loop {
        println!("loop, {:?}", tokens.peek_expect());
        if  tokens.peek_expect() == RightCurly {
            tokens.next();
            return expresions;
        }
        if let Identifier(name) = tokens.peek_expect() {
            if name == "let" {
                expresions.push(parse_let(tokens));
            }
            else { // variable/function names
                if tokens.peek_ahead_expect().token_type == Equal {
                    expresions.push(parse_assignment(tokens));
                } else if tokens.peek_ahead_expect().token_type == LeftParren {
                    expresions.push(parse_function_call(tokens));
                }
                else {
                    parse_error("not implemented", tokens);
                }
            }
        }
    }
}

fn parse_assignment(tokens: &mut TokenIter) -> ASTree {
    println!("parse_assignment");
    if let Identifier(name) = tokens.next() { // `var_name`
        if tokens.next() == Equal { // `=`
            let value = parse_value(tokens); // ...
            if tokens.next() == Semicolon { // `;`
                return ASTree::Assign { variable: name, value };
            }
            parse_error("Expected semicolon", tokens);
        }
        parse_error("Expected `=`", tokens);
    }
    parse_error("Expected variable name", tokens);
}

fn parse_let(tokens: &mut TokenIter) -> ASTree {
    println!("parse_let");
    if let Identifier(name) = tokens.next() { // `let`
        if let Identifier(name) = tokens.next() { // `var_name`
            if tokens.next() == Equal { // `=`
                let value = parse_value(tokens); // ...
                if tokens.next() == Semicolon { // `;`
                    return ASTree::Let { variable: name, value };
                }
                parse_error("Expected semicolon", tokens);
            }
            parse_error("Expected `=`", tokens);
        }
        parse_error("Expected variable name", tokens);
    }
    parse_error("Expected `let`", tokens);
}

fn parse_function_call(tokens: &mut TokenIter) -> ASTree {
    println!("parse_function_call");
    if let Identifier(name) = tokens.next() {
        if tokens.next() != LeftParren {parse_error("Expected `(` after function name", tokens)}
        let mut values = Vec::new();
        loop {
            if tokens.peek_expect() == RightParren {break;}
            values.push(parse_value(tokens));
            if tokens.peek_expect() == Comma {tokens.next();}
        }
        if tokens.next() != RightParren {parse_error("Expected `)` at the end of function call", tokens)}
        if tokens.next() != Semicolon {parse_error("Expected semicolon", tokens)}
        return ASTree::Function { name, args: values };
    }
    parse_error("expected function name", tokens);
    todo!()
}

fn parse_value(tokens: &mut TokenIter) -> ASTValue {
    println!("parse_value");
    let next_token = tokens.next();
    if let Number { whole, decimal } = next_token {
        return ASTValue::Number { whole, decimal, negative: false }; // TODO
    }
    if let StringToken(content) = next_token {
        return ASTValue::String { content };
    }
    if let Identifier(name) = next_token {
        return ASTValue::Variable { name };
    }
    dbg!(next_token);
    todo!();
}

fn parse_error(error: &str, tokens: &TokenIter) -> ! {
    println!(
        "ERROR {error} at ({}, {})",
        tokens.last.position.0,
        tokens.last.position.1,
    );
    std::process::exit(1);
}
