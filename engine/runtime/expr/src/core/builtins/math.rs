use crate::core::value::{Module, NativeFn, Value};
use crate::expect_arity;

fn unary_float(name: &'static str, f: fn(f64) -> f64) -> Value {
    let full_name = format!("math.{name}");
    Value::Fn(NativeFn::new(move |args| {
        expect_arity(&full_name, args, 1, 1)?;
        Ok(Value::Float(f(args[0].as_f64()?)))
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
        Value::Fn(NativeFn::new(|args| {
            expect_arity("math.log", args, 1, 2)?;
            let x = args[0].as_f64()?;
            if args.len() == 1 {
                Ok(Value::Float(x.ln()))
            } else {
                Ok(Value::Float(x.log(args[1].as_f64()?)))
            }
        })),
    );
    m.insert("log2".into(), unary_float("log2", f64::log2));
    m.insert("log10".into(), unary_float("log10", f64::log10));
    m.insert("radians".into(), unary_float("radians", f64::to_radians));
    m.insert("pi".into(), Value::Float(std::f64::consts::PI));
    Value::module(m)
}
