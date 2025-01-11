use std::{collections::HashMap, fmt::Display, process::exit};

use crate::ast::{self, ASTValue, ASTree, FunctionDecleration};

#[derive(Debug, Clone)]
pub enum Variable {
    Int(i64),
    Float(f64),
    String(String),
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Int(int) => int.fmt(f),
            Variable::Float(float) => float.fmt(f),
            Variable::String(string) => string.fmt(f),
        }
    }
}

type VariableScope = HashMap<String, Variable>;

pub struct CodeState {
    functions: Vec<FunctionDecleration>,
    global_var_scope: VariableScope,
}

impl CodeState {
    pub fn new(functions: Vec<FunctionDecleration>) -> Self {
        let mut global_var_scope = VariableScope::new();
        global_var_scope.insert(String::from("test"), Variable::Int(5));
        return CodeState { functions, global_var_scope };
    }
    fn variable_from_ast(&self, value: &ASTValue, local_scope: &VariableScope) -> Variable {
        return match value {
            ASTValue::Number { left, right, decimal, negative } => {
                match decimal {
                    false => Variable::Int(*left as i64),
                    true => todo!(),
                }
            },
            ASTValue::String { content } => Variable::String(content.to_string()),
            ASTValue::Function { name, args } => todo!(),
            ASTValue::Variable { name } => {
                if local_scope.contains_key(name) {
                    local_scope.get(name).unwrap().clone()
                } else if self.global_var_scope.contains_key(name) {
                    self.global_var_scope.get(name).unwrap().clone()
                } else {
                    interpreter_error("Variable not found");
                }
            },
        };
    }
    fn variable_from_asts(&self, values: &[ASTValue], local_scope: &VariableScope) -> Vec<Variable> {
        values.iter().map(|v| self.variable_from_ast(v, local_scope)).collect()
    }
    fn get_function(&self, name: String) -> FunctionDecleration {
        let valid_functions: Vec<&FunctionDecleration> = self.functions.iter().filter(|f| f.name == name).collect();
        if valid_functions.len() == 0 {
            interpreter_error("Function name not found");
        }
        return valid_functions[0].clone();
    }
    pub fn built_in_funtion(&mut self, function_name: &str, args: Vec<Variable>) -> bool {
        match function_name {
            "print" => {
                for arg in args {
                    print!("{arg}");
                }
                println!();
            }
            _ => return false,
        }
        return true;
    }
    pub fn run_function(&mut self, function_name: String, args: Vec<Variable>) {
        if self.built_in_funtion(&function_name, args) {return;}
        let function = self.get_function(function_name);
        let mut function_scope = VariableScope::new();
        for ast in &function.body {
            println!("Running: {ast:?}");
            match ast {
                ASTree::Let { variable, value } => {
                    function_scope.insert(variable.to_string(), self.variable_from_ast(&value, &function_scope));
                },
                ASTree::Assign { variable, value } => {
                    let value = self.variable_from_ast(&value, &function_scope);
                    match function_scope.get_mut(&variable.to_string()) {
                        Some(x) => *x = value,
                        None => interpreter_error("Variable not found"),
                    }
                },
                ASTree::Function { name, args } => self.run_function(name.to_string(), self.variable_from_asts(&args[..], &function_scope)),
            }
        }
    }
}

fn interpreter_error(error: &str) -> ! {
    println!("ERROR: {error}");
    exit(1);
}