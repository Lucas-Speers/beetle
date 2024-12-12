use std::iter::Peekable;

use crate::lex::{Token, Symbol, TokenType::{self, *}};

#[derive(Debug)]
pub struct FunctionDecleration {
    name: String,
    args: Vec<(String, String)>,
    body: Vec<ASTree>,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    args: Vec<ASTValue>,
}

#[derive(Debug)]
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
        func: Function,
    },
}

#[derive(Debug)]
pub enum ASTValue {
    Number {
        left: u64,
        right: u64,
        decimal: bool,
    },
    String {
        content: String,
    },
    Function {
        func: Function,
    },
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
    fn next(&mut self) -> Token {
        if self.index == self.vec.len() {
            parse_error("Reached EOF", &self);
        }
        else {
            let value = self.vec[self.index].clone();
            self.last = value.clone();
            self.index += 1;
            value
        }
    }
    fn peek(&mut self) -> Option<Token> {
        if self.index == self.vec.len() {None}
        else {
            Some(self.vec[self.index].clone())
        }
    }
    fn peek_expect(&mut self) -> Token {
        if self.index == self.vec.len() {
            parse_error("Reached EOF", &self);
        }
        else {
            self.vec[self.index].clone()
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

pub fn ast_from_tokens(tokens_vec: Vec<Token>) {
    println!("{:?}", tokens_vec);
    let mut tokens = TokenIter::new(tokens_vec);
    parse_all(&mut tokens);
}

fn parse_all(tokens: &mut TokenIter) {
    println!("parse_all");
    let imports = parse_imports(tokens);
    let functions = parse_functions(tokens);
    dbg!(imports, functions);
}

fn parse_imports(tokens: &mut TokenIter) -> Vec<String> {
    println!("parse_imports");
    let mut imported_files = Vec::new();
    loop { // check all the imports
        if let Some(token) = tokens.peek() {
            if let TokIdentifier { name } = token.token_type {
                if name == "import" {
                    tokens.next();
                    if let TokString { content } = tokens.next().token_type {
                        imported_files.push(content);
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
    let mut functions = Vec::new();
    println!("parse_function");
    while tokens.has_more() {
        if let TokIdentifier { name } = tokens.next().token_type {
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
    if let TokIdentifier { name } = tokens.next().token_type {
        if tokens.next().token_type != (TokSymbol { symbol: Symbol::LeftParren }) {parse_error("Expected `(` after function name", tokens)}
        let args = parse_function_params(tokens);
        if tokens.next().token_type != (TokSymbol { symbol: Symbol::LeftCurly }) {parse_error("Expected `{` for the function code block", tokens)}
        let body = parse_fuction_body(tokens);
        return FunctionDecleration { name, args, body };
    }
    else {
        parse_error("Expected function name", tokens);
    }
}

fn parse_function_params(tokens: &mut TokenIter) -> Vec<(String, String)> {
    println!("parse_function_params");
    let mut params = Vec::new();
    loop {
        if tokens.peek_expect().token_type == (TokSymbol { symbol: Symbol::RightParren }) {
            tokens.next();
            return params;
        }
        params.push(parse_full_varriable(tokens));
        if tokens.peek_expect().token_type == (TokSymbol { symbol: Symbol::Comma }) {
            tokens.next();
        }
    }
}

fn parse_full_varriable(tokens: &mut TokenIter) -> (String, String) {
    println!("parse_full_variable");
    if let TokIdentifier { name } = tokens.next().token_type {
        if let TokSymbol { symbol: Symbol::Colon } = tokens.next().token_type {
            return (name, parse_type(tokens));
        }
    }
    parse_error("Expected variable name", tokens);
}

fn parse_type(tokens: &mut TokenIter) -> String {
    println!("parse_type");
    if let TokIdentifier { name } = tokens.next().token_type {
        return name;
    }
    parse_error("Expected type", tokens);
}

fn parse_fuction_body(tokens: &mut TokenIter) -> Vec<ASTree> {
    let mut expresions = Vec::new();

    loop {
        if let TokSymbol { symbol: Symbol::RightCurly } = tokens.peek_expect().token_type {
            tokens.next();
            return expresions;
        }
        if let TokIdentifier { name } = tokens.peek_expect().token_type {
            if name == "let" {
                expresions.push(parse_let(tokens));
            }
            else { // variable/function names
                if let TokSymbol { symbol: Symbol::Equal } = tokens.peek_ahead_expect().token_type {
                    expresions.push(parse_assignment(tokens));
                } else if let TokSymbol { symbol: Symbol::LeftParren } = tokens.peek_ahead_expect().token_type {
                    expresions.push(parse_assignment(tokens));
                }
                else {
                    parse_error("not implemented", tokens);
                }
            }
        }
    }
}

fn parse_assignment(tokens: &mut TokenIter) -> ASTree {
    if let TokIdentifier { name } = tokens.next().token_type { // `var_name`
        if let TokSymbol { symbol: Symbol::Equal } = tokens.next().token_type { // `=`
            let value = parse_value(tokens); // ...
            if let TokSemicolon = tokens.next().token_type { // `;`
                return ASTree::Assign { variable: name, value };
            }
            parse_error("Expected semicolon", tokens);
        }
        parse_error("Expected `=`", tokens);
    }
    parse_error("Expected variable name", tokens);
}

fn parse_let(tokens: &mut TokenIter) -> ASTree {
    if let TokIdentifier { name } = tokens.next().token_type { // `let`
        if let TokIdentifier { name } = tokens.next().token_type { // `var_name`
            if let TokSymbol { symbol: Symbol::Equal } = tokens.next().token_type { // `=`
                let value = parse_value(tokens); // ...
                if let TokSemicolon = tokens.next().token_type { // `;`
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

fn parse_value(tokens: &mut TokenIter) -> ASTValue {
    let next_token = tokens.next().token_type;
    if let TokNumber { has_decimal, whole, decimal } = next_token {
        return ASTValue::Number { left: whole, right: decimal, decimal: has_decimal};
    }
    if let TokString { content } = next_token {
        return ASTValue::String { content };
    }
    dbg!(next_token);
    todo!();
}

fn parse_error(error: &str, tokens: &TokenIter) -> ! {
    println!(
        "ERROR {error} at ({}, {}) in {}",
        tokens.last.position.0,
        tokens.last.position.1,
        tokens.last.filename,
    );
    std::process::exit(1);
}
