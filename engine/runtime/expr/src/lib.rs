mod core;

pub use core::error::{Error, InnerError, InnerResult, Result};
pub use core::eval::{bool_cast, default_env, str_cast, Env};
pub use core::value::{ImmutableObject, NativeFn, Value};

pub(crate) fn expect_arity(name: &str, args: &[Value], min: usize, max: usize) -> InnerResult<()> {
    let n = args.len();
    if n >= min && n <= max {
        return Ok(());
    }
    let msg = if min == max {
        format!("{name}() expected {min} argument(s), got {n}")
    } else {
        format!("{name}() expected {min} to {max} argument(s), got {n}")
    };
    Err(InnerError::new(msg))
}

/// Compile an expression string into an opaque [`CompiledExpr`].
pub fn compile(input: &str) -> Result<CompiledExpr> {
    core::parser::parse(input).map(CompiledExpr)
}

/// Evaluate a compiled expression against an [`Env`].
pub fn eval(expr: &CompiledExpr, env: &mut Env) -> Result<Value> {
    core::eval::eval(&expr.0, env)
}

/// Opaque handle to a compiled expression. Internals are not part of the public API.
#[derive(Debug, Clone)]
pub struct CompiledExpr(core::ast::Expr);
