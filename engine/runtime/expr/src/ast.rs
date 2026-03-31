#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    In,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<Expr>),
    /// bare variable: `feature`, `workerArtifactPath`
    Var(String),
    /// index access: `expr[expr]`
    Index(Box<Expr>, Box<Expr>),
    /// namespaced function call: `file::join_path(a, b)`
    FuncCall { ns: String, name: String, args: Vec<Expr> },
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
}
