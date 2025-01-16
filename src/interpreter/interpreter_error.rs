use std::fmt::Display;

pub type InterpResult<T> = std::result::Result<T, InterpError>;

pub enum InterpError {
    VarNotFound(String),
    FuncNotFound(String),
    IncorectArgs,
    NoOperation,
}

impl Display for InterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpError::VarNotFound(var) => write!(f, "Cannot find variable: {var}"),
            InterpError::FuncNotFound(func) => write!(f, "Cannot find function: {func}"),
            InterpError::IncorectArgs => write!(f, "Incorect arguments passed to function"),
            InterpError::NoOperation => write!(f, "No operation found"),
        }
    }
}