use std::fmt::Display;

use crate::ast::Op;

use super::VarType;

pub type InterpResult<T> = std::result::Result<T, InterpError>;

pub enum InterpError {
    VarNotFound(String),
    FuncNotFound(String),
    IncorectArgs,
    NoOperation(VarType, VarType, Op),
}

impl Display for InterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpError::VarNotFound(var) => write!(f, "Cannot find variable: {var}"),
            InterpError::FuncNotFound(func) => write!(f, "Cannot find function: {func}"),
            InterpError::IncorectArgs => write!(f, "Incorect arguments passed to function"),
            InterpError::NoOperation(x, y, op) => write!(f, "No operation found for {op:?} of {x} and {y}"),
        }
    }
}