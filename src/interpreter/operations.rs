use std::rc::Rc;

use crate::ast::Op;

use super::{VarRef, VarType, Variable};

pub fn variable_operation(var1: VarRef, var2: VarRef, op: Op) -> Option<VarRef> {
    let types = (var1.borrow().to_type(), var2.borrow().to_type());
    if op == Op::Indexing {return indexing(types, var1, var2);}
    match types {

        (VarType::Bool, VarType::Bool) => {
            if let (Variable::Bool(x), Variable::Bool(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                bool_operation(x, y, op)
            } else {unreachable!()}
        },
        
        (VarType::Int, VarType::Int) => {
            if let (Variable::Int(x), Variable::Int(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                int_operation(x, y, op)
            } else {unreachable!()}
        },
        
        (VarType::Int, VarType::Float) => {
            if let (Variable::Int(x), Variable::Float(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                float_operation(x as f64, y, op)
            } else {unreachable!()}
        },

        (VarType::Float, VarType::Int) => {
            if let (Variable::Float(x), Variable::Int(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                float_operation(x, y as f64, op)
            } else {unreachable!()}
        },

        (VarType::Float, VarType::Float) => {
            if let (Variable::Float(x), Variable::Float(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                float_operation(x, y, op)
            } else {unreachable!()}
        },
        
        (VarType::String, VarType::String) => {
            if let (Variable::String(x), Variable::String(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                string_operation(&x, &y, op)
            } else {unreachable!()}
        },

        (VarType::Char, VarType::Char) => {
            if let (Variable::Char(x), Variable::Char(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                char_operation(x, y, op)
            } else {unreachable!()}
        },

        (VarType::Type, VarType::Type) => {
            if let (Variable::Type(x), Variable::Type(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                type_operation(x, y, op)
            } else {unreachable!()}
        },
        
        _ => None
    }
}

fn indexing(types: (VarType, VarType), var1: VarRef, var2: VarRef) -> Option<VarRef> {
    Some(match types {
        (VarType::List, VarType::Int) => {
            if let (Variable::List(x), Variable::Int(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                Rc::clone(&x[y as usize])
            } else {unreachable!()}
        },
        (VarType::String, VarType::Int) => {
            if let (Variable::String(x), Variable::Int(y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                Variable::Char(x.chars().nth(y as usize).unwrap()).into()
            } else {unreachable!()}
        },
        (VarType::Hash, VarType::String) => {
            if let (Variable::Hash(x), Variable::String(ref y)) = (var1.borrow().clone(), var2.borrow().clone()) {
                if let Some(x) = x.get(y) {return Some(Rc::clone(x));}
                else {
                    return Some(Variable::None.into());
                }
            } else {unreachable!()}
        },
        _ => return None,
    })
}

fn int_operation(x: i64, y: i64, op: Op) -> Option<VarRef> {
    Some(match op {
        Op::Addition => Variable::Int(x+y).into(),
        Op::Subtraction => Variable::Int(x-y).into(),
        Op::Multiplication => Variable::Int(x*y).into(),
        Op::Division => Variable::Int(x/y).into(), // TODO: y is zero
        Op::Equality => Variable::Bool(x==y).into(),
        Op::NotEquality => Variable::Bool(x!=y).into(),
        Op::Indexing => return None,
        Op::And => Variable::Int(x&y).into(),
        Op::Or => Variable::Int(x|y).into(),
        Op::Modulus => Variable::Int(x%y).into(),
        Op::LessThan => Variable::Bool(x<y).into(),
        Op::GreaterThan => Variable::Bool(x>y).into(),
    })
}

fn float_operation(x: f64, y: f64, op: Op) -> Option<VarRef> {
    Some(match op {
        Op::Addition => Variable::Float(x+y).into(),
        Op::Subtraction => Variable::Float(x-y).into(),
        Op::Multiplication => Variable::Float(x*y).into(),
        Op::Division => Variable::Float(x/y).into(),
        Op::Equality => Variable::Bool(x==y).into(),
        Op::NotEquality => Variable::Bool(x!=y).into(),
        Op::Indexing => return None,
        Op::And => return None,
        Op::Or => return None,
        Op::Modulus => Variable::Float(x%y).into(),
        Op::LessThan => Variable::Bool(x<y).into(),
        Op::GreaterThan => Variable::Bool(x>y).into(),
    })
}

fn string_operation(x: &str, y: &str, op: Op) -> Option<VarRef> {
    Some(match op {
        Op::Addition => {
            let mut new_string = x.to_owned();
            new_string.push_str(y);
            Variable::String(new_string).into()
        },
        Op::Equality => Variable::Bool(x==y).into(),
        Op::NotEquality => Variable::Bool(x!=y).into(),
        _ => return None,
    })
}

fn bool_operation(x: bool, y: bool, op: Op) -> Option<VarRef> {
    Some(match op {
        Op::And => Variable::Bool(x&y).into(),
        Op::Or => Variable::Bool(x|y).into(),
        Op::Equality => Variable::Bool(x==y).into(),
        Op::NotEquality => Variable::Bool(x!=y).into(),
        _ => return None,
    })
}

fn char_operation(x: char, y: char, op: Op) -> Option<VarRef> {
    Some(match op {
        Op::Equality => Variable::Bool(x==y).into(),
        Op::NotEquality => Variable::Bool(x!=y).into(),
        _ => return None,
    })
}

fn type_operation(x: VarType, y: VarType, op: Op) -> Option<VarRef> {
    Some(match op {
        Op::Equality => Variable::Bool(x==y).into(),
        Op::NotEquality => Variable::Bool(x!=y).into(),
        _ => return None,
    })
}