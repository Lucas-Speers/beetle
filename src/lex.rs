use std::{char, iter::Peekable, process::exit, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    /// The (Line, Column) of the token
    pub position: (u64, u64),
    /// The filename where the token originated
    // pub filename: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Semicolon,
    Identifier(String),
    Number {
        whole: u64,
        decimal: Option<u64>,
    },
    String(String),

    Addition,
    Subtraction,
    Division,
    LeftParren,
    RightParren,
    LeftCurly,
    RightCurly,
    Comma,
    Equal,
    DoubleEqual,
    // TODO: [] ! ? . < > <= >= etc
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

pub struct Tokenizer {
    input: Vec<char>,
    filename: String,
    pub tokens: Vec<Token>,
    index: usize,
    position: (u64, u64),
}

impl Tokenizer {
    pub fn new(input: &str, filename: &str) -> Self {
        Tokenizer { input: input.chars().collect(), filename: filename.into(), tokens: Vec::new(), index: 0, position: (1,1) }
    }
    fn get_next(&mut self) -> char {
        let char = self.input[self.index];
        self.index += 1;
        if char == '\n' {self.position.0 += 1; self.position.1 = 0;}
        else {self.position.1 += 1;}
        return char;
    }
    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token { token_type, position: self.position });
    }
    pub fn generate(&mut self) {
        loop {
            let current_char = self.input[self.index];
            // ignore whitespace
            if current_char.is_whitespace() {
                self.get_next();
                continue;
            }
            // check for comments
            if current_char == '/' && self.input[self.index+1] == '/' {
                loop {
                    if self.get_next() == '\n' {break;} // ignore the rest of the line
                }
                continue;
            }
            if current_char == '=' && self.input[self.index+1] == '=' {
                self.add_token(TokenType::DoubleEqual);
                self.get_next();
                continue;
            }
            match current_char {
                ';' => self.add_token(TokenType::Semicolon),
                '+' => self.add_token(TokenType::Addition),
                '-' => self.add_token(TokenType::Subtraction),
                '/' => self.add_token(TokenType::Division),
                '(' => self.add_token(TokenType::LeftParren),
                ')' => self.add_token(TokenType::RightParren),
                '{' => self.add_token(TokenType::LeftCurly),
                '}' => self.add_token(TokenType::RightCurly),
                ',' => self.add_token(TokenType::Comma),
                '=' => self.add_token(TokenType::Equal),
                _ => todo!()
            }
        }
    }
}
