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
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
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
    /// `lvalue = value` — assigns value to lvalue in current scope; evaluates to value.
    /// lvalue must be a Var, Index, or (in the future) a field-access expression.
    Assign {
        lvalue: Box<Expr>,
        value: Box<Expr>,
    },
    /// `lvalue op= rhs` — reads lvalue, applies op, writes result back; evaluates to new value.
    /// op is one of Add, Sub, Mul, Div, FloorDiv. lvalue is evaluated once (no double-evaluation).
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

#[cfg(test)]
pub mod test_util {
    use super::*;

    /// Construct an `Expr` with a dummy span for use in test expectations.
    pub fn e(kind: ExprKind) -> Expr {
        Expr::new(Span { start: 0, end: 0 }, kind)
    }

    /// Recursively compare two expression trees, ignoring spans.
    pub fn exprs_eq(a: &Expr, b: &Expr) -> bool {
        kinds_eq(&a.kind, &b.kind)
    }

    fn opt_box_eq(a: &Option<Box<Expr>>, b: &Option<Box<Expr>>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => exprs_eq(a, b),
            _ => false,
        }
    }

    fn vec_eq(a: &[Expr], b: &[Expr]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| exprs_eq(x, y))
    }

    fn pair_vec_eq(a: &[(Expr, Expr)], b: &[(Expr, Expr)]) -> bool {
        a.len() == b.len()
            && a.iter()
                .zip(b)
                .all(|((ak, av), (bk, bv))| exprs_eq(ak, bk) && exprs_eq(av, bv))
    }

    fn kinds_eq(a: &ExprKind, b: &ExprKind) -> bool {
        match (a, b) {
            (ExprKind::Null, ExprKind::Null) => true,
            (ExprKind::Bool(a), ExprKind::Bool(b)) => a == b,
            (ExprKind::Int(a), ExprKind::Int(b)) => a == b,
            (ExprKind::Float(a), ExprKind::Float(b)) => a == b,
            (ExprKind::Str(a), ExprKind::Str(b)) => a == b,
            (ExprKind::Var(a), ExprKind::Var(b)) => a == b,
            (ExprKind::Array(a), ExprKind::Array(b)) => vec_eq(a, b),
            (ExprKind::Index(at, ak), ExprKind::Index(bt, bk)) => {
                exprs_eq(at, bt) && exprs_eq(ak, bk)
            }
            (
                ExprKind::Slice {
                    target: at,
                    start: as_,
                    stop: ao,
                    step: ap,
                },
                ExprKind::Slice {
                    target: bt,
                    start: bs,
                    stop: bo,
                    step: bp,
                },
            ) => {
                exprs_eq(at, bt) && opt_box_eq(as_, bs) && opt_box_eq(ao, bo) && opt_box_eq(ap, bp)
            }
            (
                ExprKind::FuncCall { name: an, args: aa },
                ExprKind::FuncCall { name: bn, args: ba },
            ) => an == bn && vec_eq(aa, ba),
            (
                ExprKind::MethodCall {
                    receiver: ar,
                    method: am,
                    args: aa,
                },
                ExprKind::MethodCall {
                    receiver: br,
                    method: bm,
                    args: ba,
                },
            ) => exprs_eq(ar, br) && am == bm && vec_eq(aa, ba),
            (ExprKind::Unary(ao, ae), ExprKind::Unary(bo, be)) => ao == bo && exprs_eq(ae, be),
            (ExprKind::Binary(al, ao, ar), ExprKind::Binary(bl, bo, br)) => {
                ao == bo && exprs_eq(al, bl) && exprs_eq(ar, br)
            }
            (
                ExprKind::Assign {
                    lvalue: al,
                    value: av,
                },
                ExprKind::Assign {
                    lvalue: bl,
                    value: bv,
                },
            ) => exprs_eq(al, bl) && exprs_eq(av, bv),
            (
                ExprKind::CompoundAssign {
                    lvalue: al,
                    op: ao,
                    rhs: ar,
                },
                ExprKind::CompoundAssign {
                    lvalue: bl,
                    op: bo,
                    rhs: br,
                },
            ) => ao == bo && exprs_eq(al, bl) && exprs_eq(ar, br),
            (ExprKind::Block(a), ExprKind::Block(b)) => vec_eq(a, b),
            (ExprKind::Map(a), ExprKind::Map(b)) => pair_vec_eq(a, b),
            (
                ExprKind::If {
                    cond: ac,
                    then: at,
                    else_: ae,
                },
                ExprKind::If {
                    cond: bc,
                    then: bt,
                    else_: be,
                },
            ) => exprs_eq(ac, bc) && exprs_eq(at, bt) && exprs_eq(ae, be),
            (ExprKind::While { cond: ac, body: ab }, ExprKind::While { cond: bc, body: bb }) => {
                exprs_eq(ac, bc) && exprs_eq(ab, bb)
            }
            (
                ExprKind::ForIn {
                    var: av,
                    iterable: ai,
                    body: ab,
                },
                ExprKind::ForIn {
                    var: bv,
                    iterable: bi,
                    body: bb,
                },
            ) => av == bv && exprs_eq(ai, bi) && exprs_eq(ab, bb),
            _ => false,
        }
    }
}
