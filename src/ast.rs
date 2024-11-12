use std::iter::Peekable;

use crate::lex::{Token, Symbol, TokenType::{self, *}};

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
    Parren {
        values: Vec<ASTree>,
    },
    FunctionCall {
        callee: String,
        args: Vec<ASTree>,
    },
    FunctionDecleration {
        name: String,
        args: Vec<(String, String)>,
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
    fn next(&mut self) -> Option<Token> {
        if self.index == self.vec.len() {None}
        else {
            let value = self.vec[self.index].clone();
            self.last = value.clone();
            self.index += 1;
            Some(value)
        }
    }
    fn peek(&mut self) -> Option<Token> {
        if self.index == self.vec.len() {None}
        else {
            let value = self.vec[self.index].clone();
            Some(value)
        }
    }
}

pub fn ast_from_tokens(tokens_vec: Vec<Token>) {
    let mut tokens = TokenIter::new(tokens_vec);
    let ast: Vec<ASTree> = Vec::new();
    loop {
        println!("AST: {:?}", parse_expresion(&mut tokens));
    }
}


/// expected_token may be `;` or `)` and so on
fn parse_any_until(tokens: &mut TokenIter, expected_tokens: Vec<Symbol>) -> ASTree {
    let mut current_exp;
    loop {
        current_exp = parse_expresion(tokens);
        if let Some(next) = tokens.peek() { // if there is a next token
            if let TokSymbol { symbol } = &next.token_type {
                if expected_tokens.contains(&symbol) { // if it is the expected end token
                    tokens.next(); // consume the last token
                    return current_exp;
                }
            }
            // TODO chain operators
            return current_exp;
        }
        panic!("Reached EOF error");
    }
}

fn parse_expresion(tokens: &mut TokenIter) -> ASTree {
    let next_token = tokens.peek().expect("end of tokens");
    dbg!(&next_token.token_type);
    match &next_token.token_type {
        TokNumber { .. } => parse_number(tokens),
        TokIdentifier { .. } => parse_identifier(tokens),
        TokString { .. } => parse_string(tokens),
        _ => todo!()
    }
}

fn parse_number(tokens: &mut TokenIter) -> ASTree {
    if let TokNumber { has_decimal, whole, decimal } = tokens.next().unwrap().token_type {
        return ASTree::NumberLiteral { is_float: has_decimal, whole, decimal};
    }
    panic!("Expected number");
}

fn parse_string(tokens: &mut TokenIter) -> ASTree {
    if let TokString { content } = tokens.next().unwrap().token_type {
        return ASTree::StringLiteral { string: content.clone() };
    }
    panic!("expected string");
}

fn parse_parren(tokens: &mut TokenIter) -> Vec<ASTree> {
    if let TokSymbol { symbol: Symbol::LeftParren } = tokens.next().unwrap().token_type {
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
    if let TokIdentifier { name } = tokens.next().unwrap().token_type {
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

fn parse_error(error: &str, tokens: TokenIter) {
    
}
