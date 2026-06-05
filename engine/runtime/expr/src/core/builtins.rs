pub mod array;
pub mod map;
pub mod math;
mod regex;
pub mod str;
mod url;

pub use math::builtin_math;
pub use regex::builtin_regex;
pub use url::builtin_url;

use crate::core::error::InnerResult;
use crate::core::value::Value;

pub(super) type MethodFn = fn(&[Value]) -> InnerResult<Value>;
