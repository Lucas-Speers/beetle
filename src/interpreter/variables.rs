use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq)]
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
    pub fn to_bool(&self) -> bool {
        match self {
            Variable::None => false,
            Variable::Bool(bool) => *bool,
            Variable::Int(int) => *int != 0,
            Variable::Float(float) => *float != 0.0,
            Variable::Char(char) => *char as u32 != 0,
            Variable::String(string) => !string.is_empty(),
            Variable::Type(_) => true,
            Variable::List(vec) => !vec.is_empty(),
        }
    }
    pub fn to_type(&self) -> VarType {
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

pub type VarRef = Rc<RefCell<Variable>>;

impl From<Variable> for Rc<RefCell<Variable>> {
    fn from(value: Variable) -> Self {
        Rc::new(RefCell::new(value))
    }
}

pub fn deep_copy(item: &VarRef) -> VarRef {
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