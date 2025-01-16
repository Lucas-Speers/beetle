use std::{char, iter::Peekable, process::exit, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    /// The (Line, Column) of the token
    pub position: (u64, u64),
    // The filename where the token originated
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
    StringToken(String),

    Addition,
    Subtraction,
    Multiplication,
    Division,
    LeftParren,
    RightParren,
    LeftCurly,
    RightCurly,
    Colon,
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
    tokens: Vec<Token>,
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
    pub fn generate(&mut self) -> Vec<Token> {
        loop {
            if self.input.len() == self.index {break;}
            let current_char = self.input[self.index];
            println!("{current_char}");

            // ignore whitespace
            if current_char.is_whitespace() {
                self.get_next();
                continue;
            }

            // if not last char
            if self.input.len() > self.index+1 {
                // check for comments
                if current_char == '/' && self.input[self.index+1] == '/' {
                    loop {
                        if self.get_next() == '\n' {break;} // ignore the rest of the line
                    }
                    continue;
                }
                
                // check for `==`
                if current_char == '=' && self.input[self.index+1] == '=' {
                    self.add_token(TokenType::DoubleEqual);
                    self.get_next();
                    self.get_next();
                    continue;
                }
            }

            // strings
            if current_char == '"' {
                self.get_next();
                let mut s = String::new();
                loop {
                    let next_char = self.get_next();
                    if next_char == '\\' {} // todo
                    if next_char == '"' {
                        break;
                    }
                    s.push(next_char);
                }
                self.add_token(TokenType::StringToken(s));
                continue;
            }

            // numbers
            if current_char.is_ascii_digit() {
                let mut whole = self.get_next().to_digit(10).unwrap() as u64;
                loop {
                    let next_char = self.input[self.index];
                    if !next_char.is_ascii_digit() {break;}
                    whole *= 10;
                    whole += next_char.to_digit(10).unwrap() as u64;
                    self.get_next();
                }

                let decimal = None;
                if self.input[self.index] == '.' {todo!()}

                self.add_token(TokenType::Number { whole, decimal });
                continue;
            }

            match current_char {
                ';' => self.add_token(TokenType::Semicolon),
                '+' => self.add_token(TokenType::Addition),
                '-' => self.add_token(TokenType::Subtraction),
                '*' => self.add_token(TokenType::Multiplication),
                '/' => self.add_token(TokenType::Division),
                '(' => self.add_token(TokenType::LeftParren),
                ')' => self.add_token(TokenType::RightParren),
                '{' => self.add_token(TokenType::LeftCurly),
                '}' => self.add_token(TokenType::RightCurly),
                ':' => self.add_token(TokenType::Colon),
                ',' => self.add_token(TokenType::Comma),
                '=' => self.add_token(TokenType::Equal),
                c => {
                    let mut name = String::new();
                    loop {
                        if self.input.len() == self.index+1 {break;}
                        let next_char = self.input[self.index];
                        
                        if next_char.is_whitespace() {break;}
                        if vec![';','+','-','*','/','(',')','{','}',':',',','='].contains(&next_char) {break;}
                        
                        self.get_next();
                        name.push(next_char);
                    }
                    self.add_token(TokenType::Identifier(name));
                    continue;
                }
            }
            self.get_next();
        }
        return self.tokens.clone();
    }
}
