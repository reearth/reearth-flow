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
    /// Python-style slice: `expr[start:stop:step]` — all parts optional
    Slice {
        target: Box<Expr>,
        start: Option<Box<Expr>>,
        stop: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },
    /// function call: `value("key")`
    FuncCall {
        name: String,
        args: Vec<Expr>,
    },
    /// method call: `expr.method(args)`
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
}
