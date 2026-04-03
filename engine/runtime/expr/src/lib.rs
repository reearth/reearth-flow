pub mod core;
pub mod flow;

pub use core::error::{Error, Result};
pub use core::eval::Context;
pub use core::value::Value;

/// Compile an expression string into an opaque [`CompiledExpr`].
pub fn compile(input: &str) -> Result<CompiledExpr> {
    core::parser::parse(input).map(CompiledExpr)
}

/// Evaluate a compiled expression against a [`Context`].
pub fn eval(expr: &CompiledExpr, ctx: &Context) -> Result<Value> {
    core::eval::eval(&expr.0, ctx)
}

/// Evaluate and coerce the result to a `String` via the `str()` builtin.
pub fn eval_string(expr: &CompiledExpr, ctx: &Context) -> Result<String> {
    let wrapped = CompiledExpr(core::ast::Expr::FuncCall {
        name: "str".to_string(),
        args: vec![expr.0.clone()],
    });
    match eval(&wrapped, ctx)? {
        Value::String(s) => Ok(s),
        v => Err(Error::Eval {
            msg: format!("str() must return a string, got {v:?}"),
        }),
    }
}

/// Opaque handle to a compiled expression. Internals are not part of the public API.
#[derive(Debug, Clone)]
pub struct CompiledExpr(core::ast::Expr);
