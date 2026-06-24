mod core;

pub use core::env::Env;
pub use core::error::{eval_error, Error, Result};
pub use core::eval::{default_env, env_bind, env_remove};
pub use core::value::{ClosureValue, FromValue, ImmutableObject, NativeFn, TypeValue, Value};

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

/// Evaluate a compiled expression, converting the result to `T` via [`FromValue`].
pub fn eval<T>(expr: &CompiledExpr, env: &Env) -> std::result::Result<T, T::Error>
where
    T: FromValue,
    T::Error: From<Error>,
{
    let child = core::env::new_frame(Some(env.clone()));
    let before = core::value::LIVE_ALLOC.with(|c| c.get());
    let v = core::eval::eval(&expr.0, &child).map_err(T::Error::from)?;
    let result = core::value::convert_value::<T>(v)?;
    drop(child);
    let after = core::value::LIVE_ALLOC.with(|c| c.get());
    if after != before {
        panic!(
            "expr: {} TrackedRc allocation(s) still live after eval; \
             intermediate cyclic reference detected",
            after.wrapping_sub(before)
        );
    }
    Ok(result)
}

/// Evaluate a compiled expression, returning the raw [`Value`].
pub fn eval_unsafe(expr: &CompiledExpr, env: &Env) -> Result<Value> {
    core::eval::eval(&expr.0, env)
}

/// Opaque handle to a compiled expression. Internals are not part of the public API.
#[derive(Debug, Clone)]
pub struct CompiledExpr(core::ast::Expr);
