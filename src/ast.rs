use std::iter::Peekable;

use crate::lex::{Token, Symbol, TokenType::{self, *}};

pub struct FunctionDecleration {
    name: String,
    args: Vec<(String, String)>,
    body: Vec<ASTree>,
}

#[derive(Debug)]
pub enum ASTree {
    NumberLiteral {
        is_float: bool,
        whole: u64,
        decimal: u64,
    },
    StringLiteral {
        string: String,
    },
    VarriableName {
        name: String,
    },
    Operator {
        op: String,
        lhs: Box<ASTree>,
        rhs: Box<ASTree>,
    },
    FunctionCall {
        callee: String,
        args: Vec<ASTree>,
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
    fn has_more(&self) -> bool {
        self.index != self.vec.len()
    }
}

pub fn ast_from_tokens(tokens_vec: Vec<Token>) {
    let mut tokens = TokenIter::new(tokens_vec);
    parse_all(&mut tokens);
}

fn parse_all(tokens: &mut TokenIter) {
    println!("parse_all");
    let imports = parse_imports(tokens);
    parse_functions(tokens);
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
    dbg!(imported_files)
}

fn parse_functions(tokens: &mut TokenIter) {
    println!("parse_function");
    while tokens.has_more() {
        dbg!(tokens.peek_expect());
        if let TokIdentifier { name } = tokens.next().token_type {
            if name != "func" {parse_error("Expected `func` keyword", tokens)}
            parse_function_decleration(tokens);
        } else {
            parse_error("Expected function decleration", tokens);
        }
    }
}

fn parse_function_decleration(tokens: &mut TokenIter) {
    println!("parse_function_decleration");
    if let TokIdentifier { name } = tokens.next().token_type {
        if tokens.next().token_type != (TokSymbol { symbol: Symbol::LeftParren }) {parse_error("Expected `(` after function name", tokens)}
        let args = parse_function_params(tokens);
        dbg!(args);
        if tokens.next().token_type != (TokSymbol { symbol: Symbol::LeftCurly }) {parse_error("Expected `{` for the function code block", tokens)}
        let code = parse_fuction_body(tokens);
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
    let expresions = Vec::new();

    loop {
        if let TokSymbol { symbol: Symbol::RightCurly } = tokens.peek_expect().token_type {
            tokens.next();
            return expresions;
        }
        tokens.next();
    }
}





/// expected tokens may be `;` or `)` and so on
fn parse_any_until(tokens: &mut TokenIter, expected_tokens: Vec<Symbol>) -> ASTree {
    println!("parse_any_until");
    // let mut current_exp;
    loop {
        todo!();
    }
}

fn parse_expresion(tokens: &mut TokenIter) -> ASTree {
    let next_token = tokens.peek().expect("end of tokens");
    dbg!(&next_token.token_type);
    match &next_token.token_type {
        TokNumber { .. } => parse_number(tokens),
        TokIdentifier { .. } => parse_identifier(tokens),
        TokString { .. } => parse_string(tokens),
        _ => {
            tokens.next();
            parse_expresion(tokens)
        },
    }
}

fn parse_number(tokens: &mut TokenIter) -> ASTree {
    if let TokNumber { has_decimal, whole, decimal } = tokens.next().token_type {
        return ASTree::NumberLiteral { is_float: has_decimal, whole, decimal};
    }
    panic!("Expected number");
}

fn parse_string(tokens: &mut TokenIter) -> ASTree {
    if let TokString { content } = tokens.next().token_type {
        return ASTree::StringLiteral { string: content.clone() };
    }
    panic!("expected string");
}

fn parse_parren(tokens: &mut TokenIter) -> Vec<ASTree> {
    if let TokSymbol { symbol: Symbol::LeftParren } = tokens.next().token_type {
        let mut items = Vec::new();
        loop {
            items.push(parse_any_until(tokens, vec![Symbol::RightParren]));
            // consume any commas
            // break on `)`
            if let TokSymbol { symbol: Symbol::LeftParren } = tokens.peek().unwrap().token_type {
            
            }
        }
    }

    panic!("Expected parentheses")
}


fn parse_identifier(tokens: &mut TokenIter) -> ASTree {
    if let TokIdentifier { name } = tokens.next().token_type {
        // check if the next char is a `(`
        if let Some(token) = tokens.peek() {
            if let TokSymbol { symbol: Symbol::LeftParren } = token.token_type {
                return ASTree::FunctionCall { callee: name, args: parse_parren(tokens) };
            }
        }
        return ASTree::VarriableName { name };
    }
    panic!("Expected identifier");
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
