

pub enum ASTree {
    NumberLiteral {
        is_float: bool,
        value: String,
    },
    VarriableName {
        name: String,
        generics: Vec<String>,
    },
    Operator {
        op: String,
        lhs: Box<ASTree>,
        rhs: Box<ASTree>,
    },
    FunctionCall {
        callee: String,
        args: Vec<ASTree>,
    },
    FunctionDecleration {
        name: String,
        args: Vec<(String, String)>,
    }
}