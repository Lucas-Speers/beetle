use std::iter::Peekable;

use crate::lex::{Token, Symbol, TokenType::*};

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
        generics: Vec<String>,
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

pub fn ast_from_tokens(tokens_vec: Vec<Token>) {
    let mut tokens = tokens_vec.iter().cloned().peekable();
    let ast: Vec<ASTree> = Vec::new();
    loop {
        println!("AST: {:?}", parse_expresion(&mut tokens));
    }
}

fn parse_expresion<T: Iterator<Item = Token>>(tokens: &mut Peekable<T>) -> ASTree {
    let next_token = tokens.peek().expect("end of tokens");
    println!("{:?}", next_token.token_type);
    match &next_token.token_type {
        TokNumber { .. } => parse_number(tokens),
        TokIdentifier { .. } => parse_identifier(tokens),
        TokString { .. } => parse_string(tokens),
        _ => todo!()
    }
}

fn parse_number<T: Iterator<Item = Token>>(tokens: &mut Peekable<T>) -> ASTree {
    if let TokNumber { has_decimal, whole, decimal } = tokens.next().unwrap().token_type {
        return ASTree::NumberLiteral { is_float: has_decimal, whole, decimal};
    }
    unreachable!();
}

fn parse_string<T: Iterator<Item = Token>>(tokens: &mut Peekable<T>) -> ASTree {
    if let TokString { content } = tokens.next().unwrap().token_type {
        return ASTree::StringLiteral { string: content.clone() };
    }
    unreachable!();
}

fn parse_parren<T: Iterator<Item = Token>>(tokens: &mut Peekable<T>) -> ASTree {
    if let TokSymbol { symbol: Symbol::LeftParren } = tokens.next().unwrap().token_type {}

    todo!()
}


fn parse_identifier<T: Iterator<Item = Token>>(tokens: &mut Peekable<T>) -> ASTree {
    if let TokIdentifier { name } = tokens.next().unwrap().token_type {
        // check if the next char is a `(`
        if let Some(&ref next_token) = tokens.peek() {
            if let TokSymbol { symbol: Symbol::LeftParren } = next_token.token_type {
                if let ASTree::Parren { values } = parse_parren(tokens) {
                    return ASTree::FunctionCall { callee: name, args: values };
                }
                unreachable!();
            }
        }
        return ASTree::VarriableName { name, generics: Vec::new() };
    }
    unreachable!();
}
