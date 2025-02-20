use std::fmt::Display;

use crate::ast::Op;

use super::VarType;

pub type InterpResult<T> = std::result::Result<T, InterpError>;

pub struct InterpError(pub InterpErrorType);

pub enum InterpErrorType {
    VarNotFound(String),
    FuncNotFound(String),
    IncorectArgs,
    NoOperation(VarType, VarType, Op),
    IncorectType(VarType, VarType),
}

impl Display for InterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            InterpErrorType::VarNotFound(var) => write!(f, "Cannot find variable: {var}"),
            InterpErrorType::FuncNotFound(func) => write!(f, "Cannot find function: {func}"),
            InterpErrorType::IncorectArgs => write!(f, "Incorect arguments passed to function"),
            InterpErrorType::NoOperation(x, y, op) => write!(f, "No operation found for {op:?} of {x} and {y}"),
            InterpErrorType::IncorectType(t1, t2) => write!(f, "Expected type {t1}, got type {t2}"),
        }
    }
}