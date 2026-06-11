mod core;

pub use core::env::Env;
pub use core::error::{eval_error, Error, Result};
pub use core::eval::{default_env, env_bind};
pub use core::value::{ClosureValue, ImmutableObject, NativeFn, Value};

pub fn expect_arity(name: &str, args: &[Value], min: usize, max: usize) -> Result<()> {
    let n = args.len();
    if n >= min && n <= max {
        return Ok(());
    }
    let msg = if min == max {
        format!("{name}() expected {min} argument(s), got {n}")
    } else {
        format!("{name}() expected {min} to {max} argument(s), got {n}")
    };
    Err(core::error::eval_error(msg))
}

/// Compile an expression string into an opaque [`CompiledExpr`].
pub fn compile(input: &str) -> Result<CompiledExpr> {
    core::parser::parse(input).map(CompiledExpr)
}

/// Evaluate a compiled expression against an [`Env`].
pub fn eval(expr: &CompiledExpr, env: &Env) -> Result<Value> {
    core::eval::eval(&expr.0, env)
}

/// Opaque handle to a compiled expression. Internals are not part of the public API.
#[derive(Debug, Clone)]
pub struct CompiledExpr(core::ast::Expr);
