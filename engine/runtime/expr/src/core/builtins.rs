pub mod array;
pub mod map;
pub mod math;
mod regex;
pub mod str;
mod url;

pub use math::builtin_math;
pub use regex::builtin_regex;
pub use url::builtin_url;

use crate::core::error::{InnerError, InnerResult};
use crate::core::value::Value;

pub(super) type MethodFn = fn(&[Value]) -> InnerResult<Value>;

pub(super) fn expect_str(v: &Value) -> InnerResult<&str> {
    match v {
        Value::String(s) => Ok(s.as_str()),
        other => Err(InnerError::new(format!("expected string, got {}", other.type_name()))),
    }
}

pub(super) fn expect_int(v: &Value) -> InnerResult<i64> {
    match v {
        Value::Int(n) => Ok(*n),
        other => Err(InnerError::new(format!("expected int, got {}", other.type_name()))),
    }
}

/// Check that the number of user-visible arguments (i.e. `args` minus the
/// implicit receiver at index 0) is within `[min, max]`.
pub(super) fn expect_arity(args: &[Value], min: usize, max: usize) -> InnerResult<()> {
    let n = args.len().saturating_sub(1);
    if n >= min && n <= max {
        return Ok(());
    }
    let msg = if min == max {
        format!("expected {min} argument(s), got {n}")
    } else {
        format!("expected {min} to {max} argument(s), got {n}")
    };
    Err(InnerError::new(msg))
}
