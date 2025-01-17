use std::process::exit;

use crate::ast::Op;

use super::Variable;

pub fn variable_operation(var1: &Variable, var2: &Variable, op: Op) -> Option<Variable> {
    variable_operation_tracked(var1, var2, op, true)
}

fn variable_operation_tracked(var1: &Variable, var2: &Variable, op: Op, first: bool) -> Option<Variable> {
    if op == Op::Indexing {return indexing(var1, var2);}
    match (var1, var2) {
        // anything with a `None` does not change
        (Variable::None, x) => Some(x.clone()),
        (x, Variable::None) => Some(x.clone()),

        (Variable::Bool(x), Variable::Bool(y)) => todo!(),

        (Variable::Bool(x), Variable::Int(y)) => todo!(),
        (Variable::Bool(x), Variable::Float(y)) => todo!(),
        
        (Variable::Int(x), Variable::Int(y)) => int_operation(x, y, op),
        
        (Variable::Int(x), Variable::Float(y)) => todo!(),

        (Variable::Float(x), Variable::Float(y)) => float_operation(x, y, op),
        
        (Variable::String(x), Variable::String(y)) => string_operation(x, y, op),
        (_, Variable::String(y)) => todo!(),
        (Variable::String(x), _) => todo!(),

        _ => {
            if first {return variable_operation_tracked(var2, var1, op, false);}
            else {exit(1)}
        }
    }
}

fn indexing(var1: &Variable, var2: &Variable) -> Option<Variable> {
    Some(match (var1, var2) {
        (Variable::List(x), Variable::Int(i)) => x[*i as usize].clone(),
        (Variable::String(x), Variable::Int(i)) => Variable::Char(x.chars().nth(*i as usize).unwrap()),
        _ => return None,
    })
}

fn int_operation(x: &i64, y: &i64, op: Op) -> Option<Variable> {
    Some(match op {
        Op::Addition => Variable::Int(x+y),
        Op::Subtraction => Variable::Int(x-y),
        Op::Multiplication => Variable::Int(x*y),
        Op::Division => Variable::Int(x/y), // TODO: y is zero
        Op::Equality => Variable::Bool(x==y),
        Op::Indexing => return None,
    })
}

fn float_operation(x: &f64, y: &f64, op: Op) -> Option<Variable> {
    Some(match op {
        Op::Addition => Variable::Float(x+y),
        Op::Subtraction => Variable::Float(x-y),
        Op::Multiplication => Variable::Float(x*y),
        Op::Division => Variable::Float(x/y),
        Op::Equality => Variable::Bool(x==y),
        Op::Indexing => return None,
    })
}

fn string_operation(x: &str, y: &str, op: Op) -> Option<Variable> {
    Some(match op {
        Op::Addition => {
            let mut new_string = x.to_owned();
            new_string.push_str(y);
            Variable::String(new_string)
        },
        Op::Equality => Variable::Bool(x==y),
        _ => return None
    })
}