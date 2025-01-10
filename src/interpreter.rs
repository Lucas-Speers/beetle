use std::{collections::HashMap, process::exit};

use crate::ast::{self, ASTValue, ASTree, FunctionDecleration};

pub enum Variable {
    Int(i64),
    Float(f64),
    String(String),
}

impl Variable {
    fn from_astvalue(value: &ASTValue) -> Self {
        return match value {
            ASTValue::Number { left, right, decimal, negative } => {
                match decimal {
                    false => Self::Int(*left as i64),
                    true => todo!(),
                }
            },
            ASTValue::String { content } => Self::String(content.to_string()),
            ASTValue::Function { func } => todo!(),
            ASTValue::Variable { name } => todo!(),
        };
    }
    fn from_vec_astvalue(values: &[ASTValue]) -> Vec<Self> {
        values.iter().map(|v| Self::from_astvalue(v)).collect()
    }
}

type VariableScope = HashMap<String, Variable>;

pub struct CodeState {
    global_var_scope: VariableScope,
}

impl CodeState {
    pub fn new() -> Self {
        let mut global_var_scope = VariableScope::new();
        global_var_scope.insert(String::from("test"), Variable::Int(5));
        return CodeState { global_var_scope };
    }
    fn get_function(&self, name: String, functions: &Vec<FunctionDecleration>) -> usize {
        for function in 0..functions.len() {
            if functions[function].name == name {
                return function;
            }
        }
        todo!();
    }
    pub fn run_function(&mut self, function_name: String, args: Vec<Variable>, functions: &Vec<FunctionDecleration>) {
        let function = &functions[self.get_function(function_name, functions)];
        let mut function_scope = VariableScope::new();
        for ast in &function.body {
            println!("Running: {ast:?}");
            match ast {
                ASTree::Let { variable, value } => {
                    function_scope.insert(variable.to_string(), Variable::from_astvalue(value));
                },
                ASTree::Assign { variable, value } => {
                    match function_scope.get_mut(&variable.to_string()) {
                        Some(x) => *x = Variable::from_astvalue(value),
                        None => interpreter_error("Variable not found"),
                    }
                },
                ASTree::Function { name, args } => self.run_function(name.to_string(), Variable::from_vec_astvalue(&args[..]), functions),
            }
        }
    }
}

fn interpreter_error(error: &str) -> ! {
    print!("ERROR: {error}");
    exit(1);
}