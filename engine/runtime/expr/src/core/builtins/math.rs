use crate::core::error::InnerError;
use crate::core::value::{Module, NativeFn, Value};

fn unary_float(name: &'static str, f: fn(f64) -> f64) -> Value {
    Value::Fn(NativeFn::new(move |args| match args {
        [x] => Ok(Value::Float(f(x.as_f64()?))),
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
    // math.round is away-from-zero which is natural in GIS context
    m.insert("round".into(), unary_float("round", f64::round));
    m.insert(
        "log".into(),
        Value::Fn(NativeFn::new(|args| match args {
            [x] => Ok(Value::Float(x.as_f64()?.ln())),
            [x, base] => Ok(Value::Float(x.as_f64()?.log(base.as_f64()?))),
            _ => Err(InnerError::new(format!(
                "math.log() expected 1 or 2 arguments, got {}",
                args.len()
            ))),
        })),
    );
    m.insert("log2".into(), unary_float("log2", f64::log2));
    m.insert("log10".into(), unary_float("log10", f64::log10));
    m.insert("radians".into(), unary_float("radians", f64::to_radians));
    m.insert("pi".into(), Value::Float(std::f64::consts::PI));
    Value::module(m)
}
