use crate::core::error::InnerError;
use crate::core::value::{Module, NativeFn, Value};

fn unary_float(name: &'static str, f: fn(f64) -> f64) -> Value {
    Value::Fn(NativeFn::new(move |args| match args {
        [Value::Float(x)] => Ok(Value::Float(f(*x))),
        [Value::Int(x)] => Ok(Value::Float(f(*x as f64))),
        [v] => Err(InnerError::new(format!(
            "math.{name}() expected float, got {}",
            v.type_name()
        ))),
        _ => Err(InnerError::new(format!(
            "math.{name}() expected 1 argument, got {}",
            args.len()
        ))),
    }))
}

pub fn builtin_math() -> Value {
    let mut m = Module::new();
    m.insert("sin".into(), unary_float("sin", f64::sin));
    m.insert("cos".into(), unary_float("cos", f64::cos));
    m.insert("floor".into(), unary_float("floor", f64::floor));
    m.insert("round".into(), unary_float("round", f64::round));
    m.insert("log".into(), unary_float("log", f64::ln));
    m.insert("radians".into(), unary_float("radians", f64::to_radians));
    m.insert("pi".into(), Value::Float(std::f64::consts::PI));
    Value::module(m)
}
