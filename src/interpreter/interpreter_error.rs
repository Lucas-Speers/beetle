use std::fmt::Display;

use crate::ast::Op;

use super::VarType;

pub type InterpResult<T> = std::result::Result<T, InterpError>;

pub struct InterpError(
    pub (usize, u64, u64),
    pub InterpErrorType,
);

pub enum InterpErrorType {
    VarNotFound(String),
    FuncNotFound(String),
    IncorrectArgs,
    NoOperation(VarType, VarType, Op),
    IncorrectType(VarType, VarType),
}

impl Display for InterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "At {:?}: ", self.0)?;
        match &self.1 {
            InterpErrorType::VarNotFound(var) => write!(f, "Cannot find variable: {var}"),
            InterpErrorType::FuncNotFound(func) => write!(f, "Cannot find function: {func}"),
            InterpErrorType::IncorrectArgs => write!(f, "Incorrect arguments passed to function"),
            InterpErrorType::NoOperation(x, y, op) => write!(f, "No operation found for {op:?} of {x} and {y}"),
            InterpErrorType::IncorrectType(t1, t2) => write!(f, "Expected type {t1}, got type {t2}"),
        }
    }
}