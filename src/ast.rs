use std::{collections::HashMap, path::PathBuf, process::exit};

use crate::lex::{Token, TokenType::{self, *}};

#[derive(Debug, Clone)]
pub struct FunctionDecleration {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<ASTree>,
    pub position: (usize, u64, u64),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<ASTValue>,
}

#[derive(Debug, Clone)]
pub struct ASTree(
    pub (usize, u64, u64),
    pub ASTreeType
);

#[derive(Debug, Clone)]
pub enum ASTreeType {
    Let {
        variable: String,
        value: ASTValue,
    },
    Assign {
        variable: String,
        indexes: Vec<ASTValue>,
        value: ASTValue,
    },
    Function(Function),
    If {
        condition: ASTValue,
        body: Vec<ASTree>,
    },
    ElseIf {
        condition: ASTValue,
        body: Vec<ASTree>,
    },
    Else {
        body: Vec<ASTree>,
    },
    While {
        condition: ASTValue,
        body: Vec<ASTree>,
    },
    Loop {
        body: Vec<ASTree>,
    },
    For(String, ASTValue, Vec<ASTree>),
    Return(ASTValue),
    Break,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Equality,
    NotEquality,
    Indexing,
    And,
    Or,
}

impl Op {
    fn precidence(&self) -> u8 {
        match self {
            Op::And | Op::Or => 0,
            Op::Equality | Op::NotEquality => 1,
            Op::Addition | Op::Subtraction => 2,
            Op::Multiplication | Op::Division => 3,
            Op::Indexing => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ASTValue {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Function(Function),
    Variable(String),
    Operation(Box<ASTValue>, Box<ASTValue>, Op),
    List(Vec<ASTValue>),
    Hash(HashMap<String, ASTValue>),
    None,
}

pub struct ASTParser {
    tokens: Vec<Token>,
    index: usize,
}

impl ASTParser {
    pub fn new(tokens: Vec<Token>) -> Self{
        ASTParser { tokens, index: 0 }
    }

    fn get_position(&self) -> (usize, u64, u64) {
        self.tokens[self.index-1].position
    }

    fn ast_tree(&self, t: ASTreeType) -> ASTree {
        ASTree(self.get_position(), t)
    }

    fn next(&mut self) -> TokenType {
        let t = self.tokens[self.index].token_type.clone();
        self.index += 1;
        // println!(" ->{:?}", t);
        t
    }
    
    fn peek(&self, i: usize) -> TokenType {
        let t = self.tokens[self.index+i].token_type.clone();
        // println!("                                  peeked: {:?}", t);
        t
    }
    
    fn has_more(&self) -> bool {
        self.index < self.tokens.len()
    }
    
    pub fn parse_all(&mut self) -> (Vec<String>, Vec<FunctionDecleration>) {
        let imports = self.parse_imports();
        let functions = self.parse_functions();
        (imports, functions)
    }
    
    fn parse_imports(&mut self) -> Vec<String> {
        let mut imported_files = Vec::new();
        loop { // check all the imports
            if self.has_more() {
                let token = self.peek(0);
                if let Identifier(name) = token {
                    if name == "import" {
                        self.next();
                        if let StringToken(content) = self.next() {
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
    
    fn parse_functions(&mut self) -> Vec<FunctionDecleration> {
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
        if self.next() != Identifier("func".to_owned()) {self.parse_error("Expected `func` keyword")}
        if let Identifier(name) = self.next() {
            let position = self.get_position();
            let args = self.parse_function_params();
            let body = self.parse_fuction_body();
            return FunctionDecleration { name, args, body, position };
        }
        else {
            self.parse_error("Expected function name");
        }
    }
    
    fn parse_function_params(&mut self) -> Vec<String> {
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
        let mut expresions = Vec::new();
        if self.next() != LeftCurly {self.parse_error("Expected `{`")}
        loop {
            if  self.peek(0) == RightCurly {
                self.next();
                return expresions;
            }
            expresions.push(self.parse_line());
        }
    }

    fn parse_line(&mut self) -> ASTree {
        if let Identifier(name) = self.peek(0) {
            match name.as_str() {
                "let" => return self.parse_let(),
                "return" => return self.parse_return(),
                "while" => return self.parse_while(),
                "loop" => return self.parse_loop(),
                "for" => return self.parse_for(),
                "break" => return self.parse_break(),
                "continue" => return self.parse_continue(),
                "if" => {
                    return self.parse_if()
                },
                "else" => {
                    if self.peek(1) == Identifier("if".to_owned()) {
                        return self.parse_else_if();
                    }
                    return self.parse_else()
                },
                _ => {
                    if (self.peek(1) == Equal) | (self.peek(1) == LeftBracket) { // x[] = y
                        return self.parse_assignment();
                    }
                    if self.peek(1) == LeftParren { // x()
                        let function = self.parse_function_call();
                        if self.next() != Semicolon {self.parse_error("Expected semicolon")}
                        return self.ast_tree(ASTreeType::Function(function));
                    }
                    self.parse_error("not implemented");
                },
            }
        }
        self.parse_error("Expected identifier");
    }
    
    fn parse_assignment(&mut self) -> ASTree {
        if let Identifier(variable) = self.next() {
            // indexing
            let mut indexes = Vec::new();
            while self.peek(0) == LeftBracket {
                self.next();
                indexes.push(self.parse_value());
                if self.next() != RightBracket {self.parse_error("Expected `]`")}
            }
            if self.next() != Equal {self.parse_error("Expected `=`")}
            let value = self.parse_value();
            if self.next() != Semicolon {self.parse_error("Expected `;`")}
            return self.ast_tree(ASTreeType::Assign { variable, indexes, value });
        }
        else {self.parse_error("Expected variable name")}
    }
    
    fn parse_let(&mut self) -> ASTree {
        if self.next() != Identifier("let".to_owned()) {self.parse_error("Expected `let`")}
        if let Identifier(variable) = self.next() {
            if self.next() != Equal {self.parse_error("Expected `=`")}
            let value = self.parse_value();
            if self.next() != Semicolon {self.parse_error("Expected `;`")}
            return self.ast_tree(ASTreeType::Let { variable, value });
        }
        else {self.parse_error("Expected variable name")}
    }

    fn parse_if(&mut self) -> ASTree {
        if self.next() != Identifier("if".to_owned()) {self.parse_error("Expected `if`")}
        if self.next() != LeftParren {self.parse_error("Expected `(`")}
        let condition = self.parse_value();
        if self.next() != RightParren {self.parse_error("Expected `)`")}
        let body = self.parse_fuction_body();

        return self.ast_tree(ASTreeType::If { condition, body });
    }
   
    fn parse_else_if(&mut self) -> ASTree {
        if self.next() != Identifier("else".to_owned()) {self.parse_error("Expected `else`")}
        if self.next() != Identifier("if".to_owned()) {self.parse_error("Expected `if`")}
        if self.next() != LeftParren {self.parse_error("Expected `(`")}
        let condition = self.parse_value();
        if self.next() != RightParren {self.parse_error("Expected `)`")}
        let body = self.parse_fuction_body();

        return self.ast_tree(ASTreeType::ElseIf { condition, body });
    }

    fn parse_else(&mut self) -> ASTree {
        if self.next() != Identifier("else".to_owned()) {self.parse_error("Expected `else`")}
        let body = self.parse_fuction_body();

        return self.ast_tree(ASTreeType::Else { body });
    }
    
    fn parse_while(&mut self) -> ASTree {
        if self.next() != Identifier("while".to_owned()) {self.parse_error("Expected `while`")}
        if self.next() != LeftParren {self.parse_error("Expected `(`")}
        let condition = self.parse_value();
        if self.next() != RightParren {self.parse_error("Expected `)`")}
        let body = self.parse_fuction_body();

        return self.ast_tree(ASTreeType::While { condition, body });
    }
    
    fn parse_loop(&mut self) -> ASTree {
        if self.next() != Identifier("loop".to_owned()) {self.parse_error("Expected `loop`")}
        let body = self.parse_fuction_body();

        return self.ast_tree(ASTreeType::Loop { body });
    }

    fn parse_for(&mut self) -> ASTree {
        if self.next() != Identifier("for".to_owned()) {self.parse_error("Expected `for`")}
        if let Identifier(x) = self.next() {
            if self.next() != Identifier("in".to_owned()) {self.parse_error("Expected `in`")}
            let list = self.parse_value();
            let body = self.parse_fuction_body();
            return self.ast_tree(ASTreeType::For(x, list, body));
        } else {self.parse_error("Expected variable name after `for`")}
    }

    fn parse_return(&mut self) -> ASTree {
        if self.next() != Identifier("return".to_owned()) {self.parse_error("Expected `return`")}
        let value = self.parse_value();
        if self.next() != Semicolon {self.parse_error("Expected `;`")}

        return self.ast_tree(ASTreeType::Return(value));
    }

    fn parse_break(&mut self) -> ASTree {
        if self.next() != Identifier("break".to_owned()) {self.parse_error("Expected `break`")}
        if self.next() != Semicolon {self.parse_error("Expected `;`")}

        return self.ast_tree(ASTreeType::Break);
    }

    fn parse_continue(&mut self) -> ASTree {
        if self.next() != Identifier("continue".to_owned()) {self.parse_error("Expected `continue`")}
        if self.next() != Semicolon {self.parse_error("Expected `;`")}

        return self.ast_tree(ASTreeType::Continue);
    }
    
    fn parse_function_call(&mut self) -> Function {
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
        fn expect_value(values: &Vec<ASTValue>, operations: &Vec<Op>) {
            if values.len() == operations.len() {return;}
            println!("error in parsing value");
            exit(1);
        }
        fn expect_operation(values: &Vec<ASTValue>, operations: &Vec<Op>) {
            if values.len()-1 == operations.len() {return;}
            println!("error in parsing operation");
            exit(1);
        }
        let mut values = Vec::new();
        let mut operations = Vec::new();
        loop {
            match self.peek(0) {
                Identifier(name) => {
                    expect_value(&values, &operations);
                    if name == "true" {self.next();values.push(ASTValue::Bool(true));}
                    else if name == "false" {self.next();values.push(ASTValue::Bool(false));}
                    else if name == "none" {self.next();values.push(ASTValue::None);}
                    else if self.peek(1) == LeftParren {
                        values.push(ASTValue::Function(self.parse_function_call()));
                    }
                    else {
                        self.next();
                        values.push(ASTValue::Variable(name));
                    }

                    // indexing
                    while self.peek(0) == LeftBracket {
                        self.next();
                        let v = values.pop().unwrap();
                        values.push(ASTValue::Operation(Box::new(v), Box::new(self.parse_value()), Op::Indexing));
                        if self.next() != RightBracket {self.parse_error("Expected `]`")}
                    }
                },
                Int(i) => {
                    expect_value(&values, &operations);
                    self.next();
                    values.push(ASTValue::Int(i));
                },
                Float(f) => {
                    expect_value(&values, &operations);
                    self.next();
                    values.push(ASTValue::Float(f));
                },
                StringToken(content) => {
                    expect_value(&values, &operations);
                    self.next();
                    values.push(ASTValue::String(content));

                    // indexing
                    while self.peek(0) == LeftBracket {
                        self.next();
                        let v = values.pop().unwrap();
                        values.push(ASTValue::Operation(Box::new(v), Box::new(self.parse_value()), Op::Indexing));
                        if self.next() != RightBracket {self.parse_error("Expected `]`")}
                    }
                },
                CharToken(content) => {
                    expect_value(&values, &operations);
                    self.next();
                    values.push(ASTValue::Char(content));
                },
                Addition => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::Addition);
                },
                Subtraction => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::Subtraction);
                },
                Multiplication => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::Multiplication);
                },
                Division => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::Division);
                },
                And => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::And);
                },
                Or => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::Or);
                },
                LeftParren => todo!(),
                LeftCurly => {
                    if let StringToken(_) = self.peek(0) {
                        expect_value(&values, &operations);
                    } else {break;}
                    self.next();
                    let mut new_hashmap = HashMap::new();
                    loop {
                        if let RightCurly = self.peek(0) {self.next(); break;}
                        if let StringToken(s) = self.next() {
                            if let Equal = self.next() {
                                let value = self.parse_value();
                                new_hashmap.insert(s, value);
                            }
                        }
                        if let Comma = self.peek(0) {self.next();}
                    }
                    values.push(ASTValue::Hash(new_hashmap));
                }
                LeftBracket => {
                    expect_value(&values, &operations);
                    self.next();
                    let mut v = Vec::new();
                    loop {
                        if self.peek(0) == RightBracket {self.next();break;}
                        v.push(self.parse_value());
                        if self.peek(0) == Comma {self.next();}
                    }
                    values.push(ASTValue::List(v));
                },
                Colon => todo!(),
                Equal => todo!(),
                DoubleEqual => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::Equality);
                },
                NotEqual => {
                    expect_operation(&values, &operations);
                    self.next();
                    operations.push(Op::NotEquality);
                },
                Semicolon | RightParren | RightCurly | RightBracket | Comma => break,
            }
        }
        
        'outer: loop {
            if operations.len() == 0 {return values[0].clone();}
            let max = operations.iter().map(|op| op.precidence()).max().unwrap();
            for i in 0..operations.len() {
                if operations[i].precidence() == max {
                    let new_value = ASTValue::Operation(Box::new(values[i].clone()), Box::new(values[i+1].clone()), operations[i].clone());
                    values[i] = new_value;
                    values.remove(i+1);
                    operations.remove(i);
                    continue 'outer;
                }
            }
        }
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
