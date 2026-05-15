mod core;

pub use core::error::{Error, Result};
pub use core::eval::{default_env, Env};
pub use core::value::{NativeFn, Value};

/// Compile an expression string into an opaque [`CompiledExpr`].
pub fn compile(input: &str) -> Result<CompiledExpr> {
    core::parser::parse(input).map(CompiledExpr)
}

/// Evaluate a compiled expression against an [`Env`].
pub fn eval(expr: &CompiledExpr, env: &mut Env) -> Result<Value> {
    core::eval::eval(&expr.0, env)
}

/// Evaluate and coerce the result to a `String` via the `str()` builtin.
pub fn eval_string(expr: &CompiledExpr, env: &mut Env) -> Result<String> {
    let wrapped = CompiledExpr(core::ast::Expr::new(
        expr.0.span,
        core::ast::ExprKind::FuncCall {
            name: "str".to_string(),
            args: vec![expr.0.clone()],
        },
    ));
    match eval(&wrapped, env)? {
        Value::String(s) => Ok(s),
        v => Err(Error::EvalString {
            msg: format!("str() must return a string, got {v:?}"),
        }),
    }
}

/// Opaque handle to a compiled expression. Internals are not part of the public API.
#[derive(Debug, Clone)]
pub struct CompiledExpr(core::ast::Expr);
