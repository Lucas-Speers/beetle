use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Display}, process::{self, exit}, rc::Rc};

use interpreter_error::{InterpError, InterpResult};

use crate::ast::{self, ASTValue, ASTree, Function, FunctionDecleration};

mod interpreter_error;
mod operations;

#[derive(Debug, Clone, Copy)]
pub enum VarType {
    None,
    Bool,
    Int,
    Float,
    Char,
    String,
    Type,
    List,
}

impl Display for VarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarType::None => write!(f, "None"),
            VarType::Bool => write!(f, "Bool"),
            VarType::Int => write!(f, "Int"),
            VarType::Float => write!(f, "Float"),
            VarType::Char => write!(f, "Char"),
            VarType::String => write!(f, "String"),
            VarType::Type => write!(f, "Type"),
            VarType::List => write!(f, "List"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Variable {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Type(VarType),
    List(Vec<VarRef>),
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::None => Display::fmt("None", f),
            Variable::Bool(bool) => Display::fmt(bool, f),
            Variable::Int(int) => Display::fmt(int, f),
            Variable::Float(float) => Display::fmt(float, f),
            Variable::Char(char) => Display::fmt(char, f),
            Variable::String(string) => Display::fmt(string, f),
            Variable::Type(var_type) => Display::fmt(var_type, f),
            Variable::List(vec) => vec.iter().map(|v| Display::fmt(&v.borrow(), f).and(write!(f, " "))).collect(),
        }
    }
}

impl Variable {
    fn to_bool(&self) -> bool {
        match self {
            Variable::None => false,
            Variable::Bool(bool) => *bool,
            Variable::Int(int) => *int != 0,
            Variable::Float(float) => *float != 0.0,
            Variable::Char(char) => *char as u32 != 0,
            Variable::String(string) => !string.is_empty(),
            Variable::Type(var_type) => true,
            Variable::List(vec) => !vec.is_empty(),
        }
    }
    fn to_type(&self) -> VarType {
        match self {
            Variable::None => VarType::None,
            Variable::Bool(_) => VarType::Bool,
            Variable::Int(_) => VarType::Int,
            Variable::Float(_) => VarType::Float,
            Variable::Char(_) => VarType::Char,
            Variable::String(_) => VarType::String,
            Variable::Type(_) => VarType::Type,
            Variable::List(_) => VarType::List,
        }
    }
}

type VarRef = Rc<RefCell<Variable>>;
type VariableScope = HashMap<String, VarRef>;

impl From<Variable> for Rc<RefCell<Variable>> {
    fn from(value: Variable) -> Self {
        Rc::new(RefCell::new(value))
    }
}

fn deep_copy(item: &VarRef) -> VarRef {
    match *item.borrow_mut() {
        Variable::None => Variable::None,
        Variable::Bool(x) => Variable::Bool(x),
        Variable::Int(x) => Variable::Int(x),
        Variable::Float(x) => Variable::Float(x),
        Variable::Char(x) => Variable::Char(x),
        Variable::String(ref x) => Variable::String(x.clone()),
        Variable::Type(var_type) => Variable::Type(var_type),
        Variable::List(ref vec) => Variable::List(vec.iter().map(|i| deep_copy(i)).collect()),
    }.into()
}

fn clone_scope(scope: &VariableScope) -> VariableScope {
    let mut new_scope = VariableScope::new();
    for (name, item) in scope.iter() {
        new_scope.insert(name.into(), Rc::clone(&item));
    }

    new_scope
}

pub struct CodeState {
    functions: Vec<FunctionDecleration>,
    global_var_scope: VariableScope,
    ret: bool,
    brk: bool,
    con: bool,
}

impl CodeState {
    pub fn new(functions: Vec<FunctionDecleration>) -> Self {
        let mut global_var_scope = VariableScope::new();
        return CodeState { functions, global_var_scope, ret: false, brk: false, con: false };
    }
    fn variable_from_ast(&mut self, value: &ASTValue, local_scope: &VariableScope) -> InterpResult<VarRef> {
        return match value {
            ASTValue::Number { whole, decimal, negative  } => {
                match decimal {
                    Some(x) => todo!(),
                    None => Ok(Variable::Int( if *negative {-(*whole as i64)} else {*whole as i64}).into()),
                }
            },
            ASTValue::Bool(bool) => Ok(Variable::Bool(*bool).into()),
            ASTValue::String(content) => Ok(Variable::String(content.to_string().into()).into()),
            ASTValue::Function(Function { name, args }) => {
                let args = &self.variable_from_asts(&args[..], &local_scope)?;
                self.run_function(name.to_string(), args)
            },
            ASTValue::Variable(name) => {
                if local_scope.contains_key(name) {
                    Ok(local_scope.get(name).unwrap().clone())
                } else if self.global_var_scope.contains_key(name) {
                    Ok(self.global_var_scope.get(name).unwrap().clone())
                } else {
                    Err(InterpError::VarNotFound(name.to_string()))
                }
            },
            ASTValue::Operation(var1, var2, op) => {
                let x = &self.variable_from_ast(var1, local_scope)?;
                let y = &self.variable_from_ast(var2, local_scope)?;
                if let Some(v) = operations::variable_operation(Rc::clone(x), Rc::clone(y), *op) {
                    Ok(v)
                }
                else {
                    Err(InterpError::NoOperation(x.borrow().to_type(), y.borrow().to_type(), *op))
                }
            },
            ASTValue::List(vec) => Ok(Variable::List(self.variable_from_asts(&vec, local_scope)?).into()),
        };
    }
    fn variable_from_asts(&mut self, values: &[ASTValue], local_scope: &VariableScope) -> InterpResult<Vec<VarRef>> {
        values.iter().map(|v| self.variable_from_ast(v, local_scope)).collect()
    }
    fn get_function(&self, name: String) -> InterpResult<FunctionDecleration> {
        let valid_functions: Vec<&FunctionDecleration> = self.functions.iter().filter(|f| f.name == name).collect();
        if valid_functions.len() == 0 {
            return Err(InterpError::FuncNotFound(name));
        }
        return Ok(valid_functions[0].clone());
    }
    pub fn built_in_funtion(&mut self, function_name: &str, args: &Vec<VarRef>) -> InterpResult<Option<VarRef>> {
        Ok(Some(match function_name {
            "print" => {
                for arg in args {
                    print!("{}", arg.borrow());
                }
                println!();
                Variable::None.into()
            }
            "input" => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("failed to readline");
                if input.chars().nth(input.len()-1) == Some('\n') {input.pop();}
                if input.chars().nth(input.len()-1) == Some('\r') {input.pop();}

                Variable::String(input).into()
            }
            "exit" => {
                process::exit(0);
            }
            "copy" => {
                if args.len() != 1 {
                    return Err(InterpError::IncorectArgs);
                }
                deep_copy(&args[0])
            }
            _ => return Ok(None),
        }))
    }
    pub fn run_function(&mut self, function_name: String, args: &Vec<VarRef>) -> InterpResult<VarRef> {
        if let Some(value) = self.built_in_funtion(&function_name, args)? {return Ok(value);}
        let function = self.get_function(function_name)?;
        let mut function_scope = VariableScope::new();
        if function.args.len() != args.len() {return Err(InterpError::IncorectArgs);}
        for arg in 0..args.len() {
            function_scope.insert(function.args[arg].clone(), Rc::clone(&args[arg]));
        }
        self.ret = false;
        self.brk = false;
        self.con = false;
        self.run_ast_tree(&function.body, &function_scope)
    }
    fn run_ast_tree(&mut self, body: &Vec<ASTree>, scope: &HashMap<String, VarRef>) -> InterpResult<VarRef> {
        let mut current_scope = clone_scope(scope);
        let mut condition_failed = false;
        for ast in body {
            match ast {
                ASTree::Let { variable, value } => {
                    current_scope.insert(variable.to_string(), self.variable_from_ast(&value, &current_scope)?);
                },
                ASTree::Assign { variable, value } => {
                    let value = self.variable_from_ast(&value, &current_scope)?;
                    match current_scope.get_mut(&variable.to_string()) {
                        Some(x) => *x = value,
                        None => return Err(InterpError::VarNotFound(variable.to_string())),
                    }
                },
                ASTree::Function(Function { name, args }) => _ = {
                    let args = &self.variable_from_asts(&args[..], &current_scope)?;
                    self.run_function(name.to_string(), args)?
                },
                ASTree::If { condition, body } => {
                    if self.variable_from_ast(condition, &current_scope)?.borrow().to_bool() {
                        condition_failed = false;
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.con {return Ok(ret_value);}
                    } else {condition_failed = true}
                },
                ASTree::ElseIf { condition, body } => {
                    if condition_failed && self.variable_from_ast(condition, &current_scope)?.borrow().to_bool() {
                        condition_failed = false;
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.con {return Ok(ret_value);}
                    }
                },
                ASTree::Else { body } => {
                    if condition_failed {
                        condition_failed = false;
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.con {return Ok(ret_value);}
                    }
                },
                ASTree::While { condition, body } => {
                    if self.variable_from_ast(condition, &current_scope)?.borrow().to_bool() {
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.brk {self.brk = false;break;}
                        if self.con {self.con = false;}
                    }
                },
                ASTree::Loop { body } => {
                    loop {
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.brk {self.brk = false;break;}
                        if self.con {self.con = false;}
                    }
                },
                ASTree::Return(value) => {
                    self.ret = true;
                    return self.variable_from_ast(value, &current_scope);
                },
                ASTree::For(var, ast_list, body) => {
                    let list_ref = self.variable_from_ast(ast_list, &current_scope)?;
                    let x = if let Variable::List(list) = &*list_ref.borrow() {
                        let mut loop_scope = current_scope.clone();
                        for i in list {
                            loop_scope.insert(var.to_string(), Rc::clone(i));
                            let ret_value = self.run_ast_tree(body, &loop_scope)?;
                            if self.ret {return Ok(ret_value);}
                            if self.brk {self.brk = false;break;}
                            if self.con {self.con = false;}
                        }
                    } else {
                        return Err(InterpError::NotAList(list_ref.borrow().to_type()));
                    };
                },
                ASTree::Break => {
                    self.brk = true;
                    return Ok(Variable::None.into());
                },
                ASTree::Continue => {
                    self.con = true;
                    return Ok(Variable::None.into());
                },
            }
        }

        Ok(Variable::None.into())
    }
}

fn interpreter_error(error: &str) -> ! {
    println!("ERROR: {error}");
    exit(1);
}