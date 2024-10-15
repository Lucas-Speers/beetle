use std::{char, iter::Peekable, process::exit, str::Chars};

use anyhow::Result;


/// The most basic building blocks of a program
#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    /// The (Line, Column) of the token
    position: (u64, u64),
    /// The filename where the token originated
    filename: String,
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

#[derive(Debug, Clone)]
pub enum TokenType {
    TokNone,
    TokComment,
    TokSemicolon,
    TokSymbol {
        symbol: Symbol,
    },
    TokIndentifier {
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

#[derive(Debug, Clone)]
pub enum Symbol {
    Addition,
    Subtraction,
    // TODO: () {} [] ! ? . < > <= >= etc
}

impl Symbol {
    pub fn from(chars: &mut Peekable<Chars>, position: &mut (u64, u64)) -> Option<Symbol> {
        if let Some(char) = chars.peek() {
            let symbol: Symbol;
            match char {
                '+' => symbol = Symbol::Addition,
                '-' => symbol = Symbol::Subtraction,
                ')' => symbol = Symbol::Addition,
                _ => return None
            }
            chars.neext(position);
            Some(symbol)
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
                    token.token_type = TokIndentifier { name: String::new() }
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
            TokSymbol { symbol } => {
                tokens.push(token.clone());
                token.token_type = TokNone;
            }
            TokIndentifier { name } => {
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
            TokNumber { has_decimal, whole, decimal } => {
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
                    token_error(position, filename, "cannot parse number")
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

// #[derive(PartialEq, Clone)]
// #[derive(Debug)]
// pub enum Token {
//     Func,
//     Import,
//     Let,
//     If,
//     Else,
//     SetEqual,
//     IsEqual,
//     Semicolon,
//     Comma,
//     Period,
//     Colon,
//     NameSpace,
//     LessThan,
//     GreaterThan,
//     LeftParentheses,
//     RightParentheses,
//     LeftBracket,
//     RightBracket,
//     LeftCurlyBracket,
//     RightCurlyBracket,
//     Identifier(String),
//     NumLiteral(String),
//     StringLiteral(String),
// }

// pub fn tokenize(input: &str) -> Result<Vec<Token>> {

//     let mut chars = input.chars().peekable();
//     let mut tokens: Vec<Token> = Vec::new();

//     while let Some(char) = chars.next() {
//         // match whitespace
//         if char.is_whitespace() {
//             continue;
//         }
//         // match numbers
//         if char.is_ascii_digit() {
//             let mut lit_number = String::from(char);
//             while let Some(char) = chars.next_if(|c| c.is_ascii_digit()) {
//                 lit_number.push(char);
//             }
//             tokens.push(Token::NumLiteral(lit_number));
//             continue;
//         }
//         // match sybols
//         match char {
//             '/' => {
//                 if let Some('/') = chars.peek() {
//                     while let Some(_) = chars.next_if(|c| *c != '\n') {}
//                 } else {
//                     // TODO division operator
//                 }
//                 continue;
//             }
//             '"' => { // TODO: string escaping
//                 let mut string_literal = String::new();
//                 while let Some(char) = chars.next() {
//                     if char == '"' {
//                         break;
//                     }
//                     string_literal.push(char);
//                 }
//                 tokens.push(Token::StringLiteral(string_literal));
//                 continue;
//             }
//             '(' => {
//                 tokens.push(Token::LeftParentheses);
//                 continue;
//             }
//             ')' => {
//                 tokens.push(Token::RightParentheses);
//                 continue;
//             }
//             '{' => {
//                 tokens.push(Token::LeftCurlyBracket);
//                 continue;
//             }
//             '}' => {
//                 tokens.push(Token::RightCurlyBracket);
//                 continue;
//             }
//             '[' => {
//                 tokens.push(Token::LeftBracket);
//                 continue;
//             }
//             ']' => {
//                 tokens.push(Token::RightBracket);
//                 continue;
//             }
//             ';' => {
//                 tokens.push(Token::Semicolon);
//                 continue;
//             }
//             '=' => {
//                 if let Some('=') = chars.peek() {
//                     tokens.push(Token::IsEqual);
//                     chars.next();
//                     continue;
//                 }
//                 tokens.push(Token::SetEqual);
//                 continue;
//             }
//             ':' => {
//                 if let Some(':') = chars.peek() {
//                     tokens.push(Token::NameSpace);
//                     chars.next();
//                     continue;
//                 }
//                 tokens.push(Token::Colon);
//                 continue;
//             }
//             '<' => {
//                 tokens.push(Token::LessThan);
//                 continue;
//             }
//             '>' => {
//                 tokens.push(Token::GreaterThan);
//                 continue;
//             }
//             ',' => {
//                 tokens.push(Token::Comma);
//                 continue;
//             }
//             '.' => {
//                 tokens.push(Token::Period);
//                 continue;
//             }
//             _ => ()
//         }
//         // match identifiers/keywords
//         let mut identifier = String::from(char);
//         while let Some(char) = chars.next_if(|c| c.is_alphabetic()) {
//             identifier.push(char);
//         }
//         tokens.push(
//             match identifier.as_str() {
//                 "func" => Token::Func,
//                 "import" => Token::Import,
//                 "let" => Token::Let,
//                 "if" => Token::If,
//                 "else" => Token::Else,
//                 _ => Token::Identifier(identifier)
//             }
//         );
//     }

//     Ok(tokens)
// }