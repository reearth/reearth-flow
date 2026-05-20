mod core;

/// Unpack a fixed number of arguments from a method `args: &[Value]` slice,
/// binding each to a named variable. Generates an arity error if the count
/// doesn't match.
///
/// Usage:
/// - `unpack_args!(args =>)` — expect 0 arguments
/// - `unpack_args!(args => x)` — expect 1, bind to `x: &Value`
/// - `unpack_args!(args => x, y)` — expect 2, bind to `x`, `y`
#[macro_export]
macro_rules! unpack_args {
    ($args:expr =>) => {
        if !$args.is_empty() {
            return Err($crate::InnerError::new(format!(
                "expected 0 argument(s), got {}",
                $args.len()
            )));
        }
    };
    ($args:expr => $($var:ident),+) => {
        let [$($var),+] = $args else {
            return Err($crate::InnerError::new(format!(
                "expected {} argument(s), got {}",
                [$(stringify!($var)),+].len(),
                $args.len()
            )));
        };
    };
}

pub use core::error::{Error, InnerError, InnerResult, Result};
pub use core::eval::{default_env, Env};
pub use core::value::{ImmutableObject, NativeFn, Value};

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
    let value = eval(expr, env)?;
    core::eval::str_cast(value).map_err(|e| Error::EvalString { msg: e.msg })
}

/// Opaque handle to a compiled expression. Internals are not part of the public API.
#[derive(Debug, Clone)]
pub struct CompiledExpr(core::ast::Expr);
