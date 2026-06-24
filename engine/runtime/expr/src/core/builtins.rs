pub mod list;
pub mod dict;
mod itertools;
mod json;
pub mod math;
mod regex;
pub mod str;
mod url;

pub use itertools::builtin_itertools;
pub use json::builtin_json;
pub use math::builtin_math;
pub use regex::regex_type_value;
pub use url::url_type_value;

use crate::core::error::Result;
use crate::core::value::Value;

pub(super) type MethodFn = fn(&[Value]) -> Result<Value>;
