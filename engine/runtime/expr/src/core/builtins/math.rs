use crate::core::error::InnerError;
use crate::core::value::{Module, NativeFn, Value};

pub fn builtin_math() -> Value {
    let mut m = Module::new();
    m.insert(
        "sin".into(),
        Value::Fn(NativeFn::new(|args| match args {
            [Value::Float(x)] => Ok(Value::Float(x.sin())),
            [Value::Int(x)] => Ok(Value::Float((*x as f64).sin())),
            [v] => Err(InnerError::new(format!(
                "math.sin() expected float, got {}",
                v.type_name()
            ))),
            _ => Err(InnerError::new(format!(
                "math.sin() expected 1 argument, got {}",
                args.len()
            ))),
        })),
    );
    m.insert("pi".into(), Value::Float(std::f64::consts::PI));
    Value::module(m)
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::run;
    use crate::core::value::Value;

    #[test]
    fn test_math_sin() {
        let v = run("math.sin(0.0)", &[]);
        assert!(matches!(v, Value::Float(f) if f == 0.0));
    }

    #[test]
    fn test_math_pi() {
        let v = run("math.pi", &[]);
        assert!(matches!(v, Value::Float(f) if (f - std::f64::consts::PI).abs() < 1e-10));
    }
}
