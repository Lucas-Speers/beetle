use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Display}, io::{self, Write}, process, rc::Rc};

use interpreter_error::{InterpError, InterpResult};
use variables::{deep_copy, VarRef, VarType, Variable};

use crate::ast::{ASTValue, ASTree, Function, FunctionDecleration};

mod interpreter_error;
mod operations;
mod variables;


type VariableScope = HashMap<String, VarRef>;

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
        let global_var_scope = VariableScope::new();
        return CodeState { functions, global_var_scope, ret: false, brk: false, con: false };
    }
    fn variable_from_ast(&mut self, value: &ASTValue, local_scope: &VariableScope) -> InterpResult<VarRef> {
        return match value {
            ASTValue::Number { whole, decimal, negative  } => {
                match decimal {
                    Some(_) => todo!(),
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
            ASTValue::None => Ok(Variable::None.into()),
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
            "debug" => {
                for arg in args {
                    print!("{:p}: {}\n", arg.as_ref(), arg.borrow());
                }
                Variable::None.into()
            }
            "print" => {
                for arg in args {
                    print!("{}", arg.borrow());
                }
                println!();
                Variable::None.into()
            }
            "input" => {
                if args.len() >= 2 {
                    return Err(InterpError::IncorectArgs);
                }
                if args.len() == 1 {
                    print!("{}", args[0].borrow());
                    io::stdout().flush();
                }
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
                variables::deep_copy(&args[0])
            }
            "push" => {
                if args.len() != 2 {
                    return Err(InterpError::IncorectArgs);
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    l.push(deep_copy(&args[1]));
                } else {return Err(InterpError::IncorectType(VarType::List, args[0].borrow().to_type()));}

                Variable::None.into()
            }
            "pop" => {
                if args.len() != 1 {
                    return Err(InterpError::IncorectArgs);
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    return Ok(l.pop());
                } else {return Err(InterpError::IncorectType(VarType::List, args[0].borrow().to_type()));}
            }
            "insert" => {
                if args.len() != 3 {
                    return Err(InterpError::IncorectArgs);
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    if let Variable::Int(i) = *args[1].borrow_mut() {
                        l.insert(i as usize, deep_copy(&args[2]));
                    } else {return Err(InterpError::IncorectType(VarType::List, args[1].borrow().to_type()));}
                } else {return Err(InterpError::IncorectType(VarType::List, args[0].borrow().to_type()));}

                Variable::None.into()
            }
            "remove" => {
                if args.len() != 2 {
                    return Err(InterpError::IncorectArgs);
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    if let Variable::Int(i) = *args[1].borrow_mut() {
                        return Ok(Some(l.remove(i as usize)));
                    } else {return Err(InterpError::IncorectType(VarType::List, args[1].borrow().to_type()));}
                } else {return Err(InterpError::IncorectType(VarType::List, args[0].borrow().to_type()));}

                Variable::None.into()
            }
            "type" => {
                if args.len() != 1 {
                    return Err(InterpError::IncorectArgs);
                }
                Variable::Type(args[0].borrow().to_type()).into()
            }
            "int" => {
                if args.len() != 1 {
                    return Err(InterpError::IncorectArgs);
                }
                if let Variable::String(s) = &*args[0].borrow() {
                    return Ok(Some(Variable::Int(s.parse().unwrap()).into()));
                }
                Variable::None.into()
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
                        Some(x) => *x.borrow_mut() = value.borrow().clone(),
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
                    if let Variable::List(list) = &*list_ref.borrow() {
                        let mut loop_scope = current_scope.clone();
                        for i in list {
                            loop_scope.insert(var.to_string(), Rc::clone(i));
                            let ret_value = self.run_ast_tree(body, &loop_scope)?;
                            if self.ret {return Ok(ret_value);}
                            if self.brk {self.brk = false;break;}
                            if self.con {self.con = false;}
                        }
                    } else {
                        return Err(InterpError::IncorectType(VarType::List, list_ref.borrow().to_type()));
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