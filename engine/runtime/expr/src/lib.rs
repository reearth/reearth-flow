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

/// Opaque handle to a compiled expression. Internals are not part of the public API.
#[derive(Debug, Clone)]
pub struct CompiledExpr(core::ast::Expr);
