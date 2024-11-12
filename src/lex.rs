use std::{char, iter::Peekable, process::exit, str::Chars};


/// The most basic building blocks of a program
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    /// The (Line, Column) of the token
    pub position: (u64, u64),
    /// The filename where the token originated
    pub filename: String,
}

/// hacky way to update the position of the current char
pub trait CustomNext {
    fn neext(&mut self, position: &mut (u64, u64)) -> Option<char>;
}
impl CustomNext for Peekable<Chars<'_>> {
    fn neext(&mut self, position: &mut (u64, u64)) -> Option<char> {
        let value = self.next();
        if let Some(char) = value {
            // update the line/column of the token
            if char == '\n' {
                position.0 += 1;
                position.1 = 1;
            } else {
                position.1 += 1;
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    TokNone,
    TokComment,
    TokSemicolon,
    TokSymbol {
        symbol: Symbol,
    },
    TokIdentifier {
        name: String,
    },
    TokNumber {
        has_decimal: bool,
        whole: u64,
        decimal: u64,
    },
    TokString {
        content: String,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Addition,
    Subtraction,
    Division,
    LeftParren,
    RightParren,
    // TODO: {} [] ! ? . < > <= >= etc
}

impl Symbol {
    pub fn from(chars: &mut Peekable<Chars>, position: &mut (u64, u64)) -> Option<Symbol> {
        if let Some(char) = chars.peek() {
            let symbol: Symbol;
            match char {
                '+' => {
                    chars.neext(position);
                    return Some(Symbol::Addition);
                },
                '-' => {
                    chars.neext(position);
                    return Some(Symbol::Subtraction);
                },
                '(' => {
                    chars.neext(position);
                    return Some(Symbol::LeftParren);
                },
                ')' => {
                    chars.neext(position);
                    return Some(Symbol::RightParren);
                },
                '/' => {
                    chars.neext(position);
                    if let Some( next ) = chars.peek() {
                        if *next == '/' {
                            loop {
                                if let Some('\n') = chars.neext(position) {
                                    return None;
                                }
                            }
                        }
                    }
                    return Some(Symbol::Division)
                }
                _ => return None
            }
        } else {
            None
        }
        
    }
}

fn token_error(position: (u64, u64), filename: &str, error: &str) -> ! {
    println!(
        "ERROR: Parsing error at ({}, {}) in file '{}' {}",
        position.0, position.1,
        filename,
        error,
    );
    exit(1)
}

pub fn tokenize(input: &str, filename: &str) -> Vec<Token> {
    use TokenType::*;
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();
    let mut position: (u64, u64) = (1,1);
    let mut token = Token {
        token_type: TokNone,
        position,
        filename: filename.to_owned()
    };
    while let Some(&char) = iter.peek() {
        
        // the main state machine
        match &mut token.token_type {
            TokNone => {
                if char.is_whitespace() {iter.neext(&mut position);continue;}
                token.position = position;
                if char == '"' {
                    iter.neext(&mut position);
                    token.token_type = TokString{content: String::new()};
                } else if char.is_ascii_digit() {
                    println!("char: {char}");
                    token.token_type = TokNumber { has_decimal: false, whole: 0, decimal: 0 }
                } else if char == ';' {
                    iter.neext(&mut position);
                    token.token_type = TokSemicolon;
                } else if let Some(symbol) = Symbol::from(&mut iter, &mut position) {
                    token.token_type = TokSymbol { symbol };
                } else {
                    token.token_type = TokIdentifier { name: String::new() }
                }
            },
            TokComment => {
                iter.neext(&mut position);
                if char == '\n' {
                    token.token_type = TokNone;
                }
            }
            TokSemicolon => {
                tokens.push(token.clone());
                token.token_type = TokNone;
            }
            TokSymbol { symbol: _ } => {
                tokens.push(token.clone());
                token.token_type = TokNone;
            }
            TokIdentifier { name } => {
                if char.is_whitespace() {
                    iter.neext(&mut position);
                    tokens.push(token.clone());
                    token.token_type = TokNone;
                } else if let Some(symbol) = Symbol::from(&mut iter, &mut position) {
                    tokens.push(token.clone());
                    token.token_type = TokSymbol { symbol };
                } else {
                    iter.neext(&mut position);
                    name.push(char);
                }
            },
            TokNumber { has_decimal, whole: _, decimal: _ } => {
                if char.is_ascii_digit() {
                    iter.neext(&mut position);
                    // TODO
                } else if char == '.' {
                    iter.neext(&mut position);
                    *has_decimal = true;
                } else if char == ';' {
                    iter.neext(&mut position);
                    token.token_type = TokSemicolon;
                } else if let Some(symbol) = Symbol::from(&mut iter, &mut position) {
                    tokens.push(token.clone());
                    token.token_type = TokSymbol { symbol };
                } else {
                    tokens.push(token.clone());
                    token.token_type = TokNone;
                }
                
            },
            TokString { content } => {
                iter.neext(&mut position);
                if char == '"' {
                    tokens.push(token.clone());
                    token.token_type = TokNone;
                } else {
                    content.push(char);
                }
            },
        }
    }
    // end of file stuff
    // match token.token_type
    
    tokens
}

