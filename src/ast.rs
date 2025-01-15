use std::{iter::Peekable, path::PathBuf};

use crate::lex::{Token, TokenType::{self, *}};

#[derive(Debug, Clone)]
pub struct FunctionDecleration {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<ASTree>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<ASTValue>,
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
    Function(Function),
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
    Function(Function),
    Variable {
        name: String,
    }
}

pub struct ASTParser {
    tokens: Vec<Token>,
    index: usize,
}

impl ASTParser {
    pub fn new(tokens: Vec<Token>) -> Self{
        ASTParser { tokens, index: 0 }
    }

    fn next(&mut self) -> TokenType {
        let t = self.tokens[self.index].token_type.clone();
        self.index += 1;
        println!(" ->{:?}", t);
        t
    }
    fn peek(&self, i: usize) -> TokenType {
        let t = self.tokens[self.index+i].token_type.clone();
        t
    }
    fn has_more(&self) -> bool {
        self.index < self.tokens.len()
    }
    
    pub fn parse_all(&mut self) -> (Vec<PathBuf>, Vec<FunctionDecleration>) {
        println!("parse_all");
        let imports = self.parse_imports();
        let functions = self.parse_functions();
        dbg!(imports, functions)
    }
    
    fn parse_imports(&mut self) -> Vec<PathBuf> {
        println!("parse_imports");
        let mut imported_files = Vec::new();
        loop { // check all the imports
            if self.has_more() {
                let token = self.peek(0);
                if let Identifier(name) = token {
                    if name == "import" {
                        self.next();
                        if let StringToken(content) = self.next() {
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
    
    fn parse_functions(&mut self) -> Vec<FunctionDecleration> {
        println!("parse_functions");
        let mut functions = Vec::new();
        while self.has_more() {
            if let Identifier(name) = self.peek(0) {
                if name == "func" {
                    functions.push(self.parse_function_decleration());
                }
                else {
                    self.parse_error("Unexpected token in file");
                }
            }
        }
        functions
    }
    
    fn parse_function_decleration(&mut self) -> FunctionDecleration {
        println!("parse_function_decleration");
        if self.next() != Identifier("func".to_owned()) {self.parse_error("Expected `func` keyword")}
        if let Identifier(name) = self.next() {
            let args = self.parse_function_params();
            let body = self.parse_fuction_body();
            return FunctionDecleration { name, args, body };
        }
        else {
            self.parse_error("Expected function name");
        }
    }
    
    fn parse_function_params(&mut self) -> Vec<String> {
        println!("parse_function_params");
        let mut params = Vec::new();
        if self.next() != LeftParren {self.parse_error("Expected `(`")}
        loop {
            if self.peek(0) == RightParren {self.next();return params;}

            if let Identifier(name) = self.next() {params.push(name);}
            else {self.parse_error("Expected variable name");}
            
            if self.peek(0) == Comma {self.next();}
            else {
                if self.next() != RightParren {self.parse_error("Expected `)`")}
                return params;
            }
        }
    }
    
    fn parse_fuction_body(&mut self) -> Vec<ASTree> {
        println!("parse_fuction_body");
        let mut expresions = Vec::new();
        if self.next() != LeftCurly {self.parse_error("Expected `{`")}
        loop {
            println!("loop, {:?}", self.peek(0));
            if  self.peek(0) == RightCurly {
                self.next();
                return expresions;
            }
            if let Identifier(name) = self.peek(0) {
                if name == "let" {
                    expresions.push(self.parse_let());
                }
                else { // variable/function names
                    if self.peek(1) == Equal {
                        expresions.push(self.parse_assignment());
                    } else if self.peek(1) == LeftParren {
                        expresions.push(ASTree::Function(self.parse_function_call()));
                        if self.next() != Semicolon {self.parse_error("Expected semicolon")}
                    }
                    else {
                        self.parse_error("not implemented");
                    }
                }
            }
        }
    }
    
    fn parse_assignment(&mut self) -> ASTree {
        println!("parse_assignment");
        if let Identifier(variable) = self.next() {
            if self.next() != Equal {self.parse_error("Expected `=`")}
            let value = self.parse_value();
            if self.next() != Semicolon {self.parse_error("Expected `;`")}
            return ASTree::Assign { variable, value };
        }
        else {self.parse_error("Expected variable name")}
    }
    
    fn parse_let(&mut self) -> ASTree {
        println!("parse_let");
        if self.next() != Identifier("let".to_owned()) {self.parse_error("Expected `let`")}
        if let Identifier(variable) = self.next() {
            if self.next() != Equal {self.parse_error("Expected `=`")}
            let value = self.parse_value();
            if self.next() != Semicolon {self.parse_error("Expected `;`")}
            return ASTree::Let { variable, value };
        }
        else {self.parse_error("Expected variable name")}
    }
    
    fn parse_function_call(&mut self) -> Function {
        println!("parse_function_call");
        if let Identifier(name) = self.next() {
            if self.next() != LeftParren {self.parse_error("Expected `(` after function name")}
            let mut values = Vec::new();
            loop {
                if self.peek(0) == RightParren {self.next();break;}
                values.push(self.parse_value());
                if self.peek(0) == Comma {self.next();}
            }
            return Function { name, args: values };
        }
        self.parse_error("expected function name");
    }
    
    fn parse_value(&mut self) -> ASTValue {
        println!("parse_value");
        let next_token = self.peek(0);
        if let Number { whole, decimal } = next_token {
            self.next();
            return ASTValue::Number { whole, decimal, negative: false }; // TODO
        }
        if let StringToken(content) = next_token {
            self.next();
            return ASTValue::String { content };
        }
        if let Identifier(name) = next_token {
            if self.peek(1) == LeftParren {
                return ASTValue::Function(self.parse_function_call());
            }
            self.next();
            return ASTValue::Variable { name };
        }
        dbg!(next_token);
        todo!();
    }
    
    fn parse_error(&mut self, error: &str) -> ! {
        println!(
            "ERROR {error} at ({}, {})",
            self.tokens[self.index].position.0,
            self.tokens[self.index].position.1,
        );
        std::process::exit(1);
    }
}
