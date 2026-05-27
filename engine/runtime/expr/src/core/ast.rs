#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    In,
    NotIn,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
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
    /// attribute access: `expr.attr`
    Attribute {
        receiver: Box<Expr>,
        attr: String,
    },
    /// call expression: `expr(args)`
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// `lvalue = value` — assigns value to lvalue in current scope; evaluates to value.
    /// lvalue must be a Var, Index, or (in the future) a field-access expression.
    Assign {
        lvalue: Box<Expr>,
        value: Box<Expr>,
    },
    /// `lvalue op= rhs` — reads lvalue, applies op, writes result back; lvalue is evaluated once.
    CompoundAssign {
        lvalue: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
    /// `{ e1; e2; e3 }` — sequence expression; evaluates each, returns last
    Block(Vec<Expr>),
    /// `{ key: value, ... }` — map literal; key is any expr (must eval to string at runtime)
    Map(Vec<(Expr, Expr)>),
    /// `if cond { then } [else { else_ }]` — else_ is Null if omitted
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    /// `while cond { body }` — evaluates to Null
    While {
        cond: Box<Expr>,
        body: Box<Expr>,
    },
    /// `for var in iterable { body }` — list/map/string iteration, evaluates to Null
    ForIn {
        var: String,
        iterable: Box<Expr>,
        body: Box<Expr>,
    },
    /// `return [expr]` — exits the current script (or future closure) with a value
    Return(Option<Box<Expr>>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(span: Span, kind: ExprKind) -> Self {
        Self { span, kind }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[cfg(test)]
pub mod test_util {
    use super::*;

    /// Construct an `Expr` with a dummy span for use in test expectations.
    pub fn e(kind: ExprKind) -> Expr {
        Expr::new(Span { start: 0, end: 0 }, kind)
    }
}
