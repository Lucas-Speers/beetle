use std::{collections::HashMap, io::{self, BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, ops::DerefMut, process, rc::Rc};

use interpreter_error::{InterpError, InterpResult, InterpErrorType::*};
use variables::{deep_copy, VarRef, VarType, Variable};

use crate::ast::{ASTValue, ASTree, ASTreeType, Function, FunctionDecleration, Op};

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
    tcp_listener: Option<TcpListener>,
    tcp_stream: Option<TcpStream>,
}

impl CodeState {
    pub fn new(functions: Vec<FunctionDecleration>) -> Self {
        let global_var_scope = VariableScope::new();
        return CodeState { functions, global_var_scope, ret: false, brk: false, con: false, tcp_listener: None, tcp_stream: None };
    }
    fn variable_from_ast(&mut self, value: &ASTValue, local_scope: &VariableScope, position: (usize, u64, u64)) -> InterpResult<VarRef> {
        return match value {
            ASTValue::Int(i) => {Ok(Variable::Int(*i).into())}
            ASTValue::Float(f) => {Ok(Variable::Float(*f).into())}
            ASTValue::Bool(bool) => Ok(Variable::Bool(*bool).into()),
            ASTValue::String(content) => Ok(Variable::String(content.to_owned().into()).into()),
            ASTValue::Char(content) => Ok(Variable::Char(*content).into()),
            ASTValue::Function(Function { name, args }) => {
                let args = &self.variable_from_asts(&args[..], &local_scope, position)?;
                self.run_function(name, args, position)
            },
            ASTValue::Variable(name) => {
                if local_scope.contains_key(name) {
                    Ok(local_scope.get(name).unwrap().clone())
                } else if self.global_var_scope.contains_key(name) {
                    Ok(self.global_var_scope.get(name).unwrap().clone())
                } else {
                    Err(InterpError(position, VarNotFound(name.to_owned())))
                }
            },
            ASTValue::Operation(var1, var2, op) => {
                let x = &self.variable_from_ast(var1, local_scope, position)?;
                let y = &self.variable_from_ast(var2, local_scope, position)?;
                if let Some(v) = operations::variable_operation(Rc::clone(x), Rc::clone(y), *op) {
                    Ok(v)
                }
                else {
                    Err(InterpError(position, NoOperation(x.borrow().to_type(), y.borrow().to_type(), *op)))
                }
            },
            ASTValue::List(vec) => Ok(Variable::List(self.variable_from_asts(&vec, local_scope, position)?).into()),
            ASTValue::Hash(hash) => {
                let mut new_hash = HashMap::new();

                for (k, v) in hash {
                    new_hash.insert(k.to_owned(), self.variable_from_ast(v, local_scope, position)?);
                }

                Ok(Variable::Hash(Box::new(new_hash)).into())
            },
            ASTValue::None => Ok(Variable::None.into()),
        };
    }
    fn variable_from_asts(&mut self, values: &[ASTValue], local_scope: &VariableScope, position: (usize, u64, u64)) -> InterpResult<Vec<VarRef>> {
        values.iter().map(|v| self.variable_from_ast(v, local_scope, position)).collect()
    }
    fn get_function(&self, name: &str, position: (usize, u64, u64)) -> InterpResult<FunctionDecleration> {
        let valid_functions: Vec<&FunctionDecleration> = self.functions.iter().filter(|f| f.name == name).collect();
        if valid_functions.len() == 0 {
            return Err(InterpError(position, FuncNotFound(name.to_owned())));
        }
        return Ok(valid_functions[0].clone());
    }
    fn built_in_funtion(&mut self, function_name: &str, args: &Vec<VarRef>, position: (usize, u64, u64)) -> InterpResult<Option<VarRef>> {
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
            "printsl" => {
                for arg in args {
                    print!("{}", arg.borrow());
                }
                Variable::None.into()
            }
            "input" => {
                if args.len() >= 2 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if args.len() == 1 {
                    print!("{}", args[0].borrow());
                    io::stdout().flush().unwrap();
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
                    return Err(InterpError(position, IncorectArgs));
                }
                variables::deep_copy(&args[0])
            }
            "push" => {
                if args.len() != 2 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    l.push(deep_copy(&args[1]));
                } else {return Err(InterpError(position, IncorectType(VarType::List, args[0].borrow().to_type())));}

                Variable::None.into()
            }
            "pop" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    return Ok(l.pop());
                } else {return Err(InterpError(position, IncorectType(VarType::List, args[0].borrow().to_type())));}
            }
            "insert" => {
                if args.len() != 3 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    if let Variable::Int(i) = *args[1].borrow_mut() {
                        l.insert(i as usize, deep_copy(&args[2]));
                    } else {return Err(InterpError(position, IncorectType(VarType::List, args[1].borrow().to_type())));}
                }

                todo!()
            }
            "remove" => {
                if args.len() != 2 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    if let Variable::Int(i) = *args[1].borrow_mut() {
                        return Ok(Some(l.remove(i as usize)));
                    }
                    return Err(InterpError(position, IncorectType(VarType::Int, args[1].borrow().to_type())));
                }

                todo!()
            }
            "set" => {
                if args.len() != 3 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    if let Variable::Int(i) = *args[1].borrow() {
                        l[i as usize] = deep_copy(&args[2]);
                        return Ok(Some(Variable::None.into()));
                    } else {return Err(InterpError(position, IncorectType(VarType::List, args[1].borrow().to_type())));}
                }

                if let Variable::String(ref mut l) = *args[0].borrow_mut() {
                    if let Variable::Int(i) = *args[1].borrow() {
                        if let Variable::Char(c) = *args[2].borrow() {
                            l.replace_range((i as usize)..(i as usize + 1), &c.to_string());
                            return Ok(Some(Variable::None.into()));
                        } else {return Err(InterpError(position, IncorectType(VarType::Char, args[2].borrow().to_type())));}
                    } else {return Err(InterpError(position, IncorectType(VarType::List, args[1].borrow().to_type())));}
                }

                todo!()
            }
            "type" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                Variable::Type(args[0].borrow().to_type()).into()
            }
            "int" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::String(s) = &*args[0].borrow() {
                    return Ok(Some(Variable::Int(s.parse().unwrap()).into()));
                }
                if let Variable::Char(c) = &*args[0].borrow() {
                    return Ok(Some(Variable::Int(c.to_string().parse().unwrap()).into()));
                }
                Variable::None.into()
            }
            "str" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::Int(i) = &*args[0].borrow() {
                    return Ok(Some(Variable::String(i.to_string()).into()));
                }
                Variable::None.into()
            }
            "len" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::List(l) = &*args[0].borrow() {
                    return Ok(Some(Variable::Int(l.len() as i64).into()))
                }
                if let Variable::String(l) = &*args[0].borrow() {
                    return Ok(Some(Variable::Int(l.len() as i64).into()))
                }
                todo!()
            }
            "range" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::Int(l) = &*args[0].borrow() {
                    return Ok(Some(
                        Variable::List(
                            (0..*l)
                            .into_iter()
                            .map(|x| Variable::Int(x).into())
                            .collect()
                        ).into()))
                }
                todo!()
            }
            "contains" => {
                if args.len() != 2 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::List(ref mut l) = *args[0].borrow_mut() {
                    Variable::Bool(l.contains(&args[1])).into()
                } else {todo!()}
            }
            "tcp_bind" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::String(ref s) = *args[0].borrow() {
                    self.tcp_listener = Some(TcpListener::bind(s).unwrap());
                } else {todo!()}
                Variable::None.into()
            }
            "tcp_unbind" => {
                if args.len() != 0 {
                    return Err(InterpError(position, IncorectArgs));
                }
                self.tcp_listener = None;
                Variable::None.into()
            }
            "tcp_listen" => {
                if args.len() != 0 {
                    return Err(InterpError(position, IncorectArgs));
                }
                let mut request = String::new();
                let (incoming, _addr) = self.tcp_listener.as_mut().unwrap().accept().unwrap();
                let mut reader = BufReader::new(&incoming);
                reader.read_line(&mut request).unwrap();
                self.tcp_stream = Some(incoming);
                Variable::String(request).into()
            }
            "tcp_write" => {
                if args.len() != 1 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::String(ref s) = *args[0].borrow() {
                    self.tcp_stream.as_mut().unwrap().write_all(s.as_bytes()).unwrap();
                    self.tcp_stream = None;
                }
                Variable::None.into()
            }
            "split" => {
                if args.len() != 2 {
                    return Err(InterpError(position, IncorectArgs));
                }
                if let Variable::String(ref s) = *args[0].borrow() {
                    if let Variable::String(ref d) = *args[1].borrow() {
                        return Ok(Some(Variable::List(s.split(d).map(|a| Variable::String(a.to_owned()).into()).collect()).into()));
                    }
                }
                todo!()
            }
            _ => return Ok(None),
        }))
    }
    pub fn run_function(&mut self, function_name: &str, args: &Vec<VarRef>, position: (usize, u64, u64)) -> InterpResult<VarRef> {
        if let Some(value) = self.built_in_funtion(function_name, args, position)? {return Ok(value);}
        let function = self.get_function(function_name, position)?;
        let mut function_scope = VariableScope::new();
        if function.args.len() != args.len() {return Err(InterpError(function.position, IncorectArgs));}
        for arg in 0..args.len() {
            function_scope.insert(function.args[arg].clone(), Rc::clone(&args[arg]));
        }
        self.ret = false;
        self.brk = false;
        self.con = false;
        let return_value = self.run_ast_tree(&function.body, &function_scope);
        self.ret = false;
        self.brk = false;
        self.con = false;
        return_value
    }
    fn run_ast_tree(&mut self, body: &Vec<ASTree>, scope: &HashMap<String, VarRef>) -> InterpResult<VarRef> {
        let mut current_scope = clone_scope(scope);
        let mut condition_failed = false;
        for ast in body {
            let position = ast.0;
            match &ast.1 {
                ASTreeType::Let { variable, value } => {
                    // println!("ASTreeType::Let");
                    current_scope.insert(variable.to_owned(), self.variable_from_ast(&value, &current_scope, position)?);
                },
                ASTreeType::Assign { variable, indexes, value } => {
                    // println!("ASTreeType::Assign");
                    // value to be put into the variable
                    let value = self.variable_from_ast(&value, &current_scope, position)?;
                    
                    match current_scope.get(&variable.to_owned()) {
                        Some(x) => { // original varialbe
                            let mut changing_var = Rc::clone(x);
                            for i in indexes {
                                let indecie = self.variable_from_ast(i, &current_scope, position)?;

                                let var_type = changing_var.borrow().to_type();
                                if let VarType::Hash = var_type { // special rules if it's a hashmap
                                    let new_var;
                                    {
                                        let mut hash = changing_var.borrow_mut();
                                        if let Variable::Hash(h) = hash.deref_mut() {
                                            if let Variable::String(ref s) = indecie.borrow().clone() {
                                                if let Some(v) = h.get(s) {
                                                    new_var = Rc::clone(v);
                                                } else {
                                                    new_var = Variable::None.into();
                                                    h.insert(s.to_owned(), Rc::clone(&new_var));
                                                }
                                            } else {return Err(InterpError(position, NoOperation(value.borrow().to_type(), indecie.borrow().to_type(), Op::Indexing)));}
                                        } else {unreachable!()}
                                    }
                                    changing_var = new_var;
                                    break;
                                }

                                if let Some(new_var) = operations::variable_operation(changing_var, Rc::clone(&indecie), Op::Indexing) {
                                    changing_var = Rc::clone(&new_var);
                                } else {
                                    return Err(InterpError(position, NoOperation(value.borrow().to_type(), indecie.borrow().to_type(), Op::Indexing)));
                                }
                            }
                            *changing_var.borrow_mut() = value.borrow().clone();
                        },
                        None => return Err(InterpError(position, VarNotFound(variable.to_owned()))),
                    }
                },
                ASTreeType::Function(Function { name, args }) => {
                    // println!("ASTreeType::Function");
                    let args = &self.variable_from_asts(&args[..], &current_scope, position)?;
                    let _ret = self.run_function(name, args, position)?;
                    self.ret = false;
                    self.brk = false;
                    self.con = false;
                },
                ASTreeType::If { condition, body } => {
                    // println!("ASTreeType::If");
                    if self.variable_from_ast(condition, &current_scope, position)?.borrow().to_bool() {
                        condition_failed = false;
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.con {return Ok(ret_value);}
                    } else {condition_failed = true}
                },
                ASTreeType::ElseIf { condition, body } => {
                    // println!("ASTreeType::ElseIf");
                    if condition_failed && self.variable_from_ast(condition, &current_scope, position)?.borrow().to_bool() {
                        condition_failed = false;
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.con {return Ok(ret_value);}
                    }
                },
                ASTreeType::Else { body } => {
                    // println!("ASTreeType::Else");
                    if condition_failed {
                        condition_failed = false;
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.con {return Ok(ret_value);}
                    }
                },
                ASTreeType::While { condition, body } => {
                    // println!("ASTreeType::While");
                    while self.variable_from_ast(condition, &current_scope, position)?.borrow().to_bool() {
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.brk {self.brk = false;break;}
                        if self.con {self.con = false;}
                    }
                },
                ASTreeType::Loop { body } => {
                    // println!("ASTreeType::Loop");
                    loop {
                        let ret_value = self.run_ast_tree(body, &current_scope)?;
                        if self.ret {return Ok(ret_value);}
                        if self.brk {self.brk = false;break;}
                        if self.con {self.con = false;}
                    }
                },
                ASTreeType::Return(value) => {
                    // println!("ASTreeType::Return");
                    self.ret = true;
                    return self.variable_from_ast(value, &current_scope, position);
                },
                ASTreeType::For(var, ast_list, body) => {
                    // println!("ASTreeType::For");
                    let list_ref = self.variable_from_ast(ast_list, &current_scope, position)?;
                    if let Variable::List(list) = &*list_ref.borrow() {
                        let mut loop_scope = current_scope.clone();
                        for i in list {
                            loop_scope.insert(var.to_owned(), Rc::clone(i));
                            let ret_value = self.run_ast_tree(body, &loop_scope)?;
                            if self.ret {return Ok(ret_value);}
                            if self.brk {self.brk = false;break;}
                            if self.con {self.con = false;}
                        }
                    } else {
                        return Err(InterpError(position, IncorectType(VarType::List, list_ref.borrow().to_type())));
                    };
                },
                ASTreeType::Break => {
                    // println!("ASTreeType::Break");
                    self.brk = true;
                    return Ok(Variable::None.into());
                },
                ASTreeType::Continue => {
                    // println!("ASTreeType::Continue");
                    self.con = true;
                    return Ok(Variable::None.into());
                },
            }
        }

        Ok(Variable::None.into())
    }
}