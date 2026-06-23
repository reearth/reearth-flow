pub mod array;
mod itertools;
mod json;
pub mod map;
pub mod math;
mod regex;
pub mod str;
mod url;

pub use itertools::builtin_itertools;
pub use json::builtin_json;
pub use math::builtin_math;
pub use regex::builtin_regex;
pub use url::builtin_url;

use crate::core::error::Result;
use crate::core::value::Value;

pub(super) type MethodFn = fn(&[Value]) -> Result<Value>;
