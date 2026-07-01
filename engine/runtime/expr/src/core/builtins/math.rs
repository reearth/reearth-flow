use crate::core::error::{eval_error, Result};
use crate::core::value::{Module, NativeFn, Value};
use crate::expect_arity;

fn float_to_int(name: &str, x: f64) -> Result<i64> {
    if !x.is_finite() {
        return Err(eval_error(format!(
            "{name}: cannot convert non-finite value to int"
        )));
    }
    if x.fract() != 0.0 {
        return Err(eval_error(format!(
            "{name}: float value has fractional part"
        )));
    }
    if x < i64::MIN as f64 || x >= i64::MAX as f64 {
        return Err(eval_error(format!("{name}: float value out of i64 range")));
    }
    Ok(x as i64)
}

fn unary_float(name: &'static str, f: fn(f64) -> f64) -> Value {
    let full_name = format!("math.{name}");
    Value::Fn(NativeFn::new(move |args| {
        expect_arity(&full_name, args, 1, 1)?;
        Ok(Value::Float(f(args[0].as_f64()?)))
    }))
}

fn unary_bool(name: &'static str, f: fn(f64) -> bool) -> Value {
    let full_name = format!("math.{name}");
    Value::Fn(NativeFn::new(move |args| {
        expect_arity(&full_name, args, 1, 1)?;
        Ok(Value::Bool(f(args[0].as_f64()?)))
    }))
}

pub fn builtin_math() -> Value {
    let mut m = Module::new();
    m.insert("pi".into(), Value::Float(std::f64::consts::PI));
    m.insert("e".into(), Value::Float(std::f64::consts::E));
    m.insert("inf".into(), Value::Float(f64::INFINITY));
    m.insert("nan".into(), Value::Float(f64::NAN));

    m.insert(
        "abs".into(),
        Value::Fn(NativeFn::new(|args| {
            expect_arity("math.abs", args, 1, 1)?;
            match &args[0] {
                Value::Int(n) => n
                    .checked_abs()
                    .map(Value::Int)
                    .ok_or_else(|| eval_error("math.abs: integer overflow")),
                _ => Ok(Value::Float(args[0].as_f64()?.abs())),
            }
        })),
    );
    m.insert(
        "floor".into(),
        Value::Fn(NativeFn::new(|args| {
            expect_arity("math.floor", args, 1, 1)?;
            if let Value::Int(_) = &args[0] {
                return Ok(args[0].clone());
            }
            float_to_int("math.floor", args[0].as_f64()?.floor()).map(Value::Int)
        })),
    );
    m.insert(
        "ceil".into(),
        Value::Fn(NativeFn::new(|args| {
            expect_arity("math.ceil", args, 1, 1)?;
            if let Value::Int(_) = &args[0] {
                return Ok(args[0].clone());
            }
            float_to_int("math.ceil", args[0].as_f64()?.ceil()).map(Value::Int)
        })),
    );
    // math.round is away-from-zero which is natural in GIS context
    m.insert(
        "round".into(),
        Value::Fn(NativeFn::new(|args| {
            expect_arity("math.round", args, 1, 1)?;
            if let Value::Int(_) = &args[0] {
                return Ok(args[0].clone());
            }
            float_to_int("math.round", args[0].as_f64()?.round()).map(Value::Int)
        })),
    );
    m.insert("sqrt".into(), unary_float("sqrt", f64::sqrt));
    m.insert("exp".into(), unary_float("exp", f64::exp));

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

    m.insert("sin".into(), unary_float("sin", f64::sin));
    m.insert("cos".into(), unary_float("cos", f64::cos));
    m.insert("tan".into(), unary_float("tan", f64::tan));
    m.insert("asin".into(), unary_float("asin", f64::asin));
    m.insert("acos".into(), unary_float("acos", f64::acos));
    m.insert("atan".into(), unary_float("atan", f64::atan));
    m.insert(
        "atan2".into(),
        Value::Fn(NativeFn::new(|args| {
            expect_arity("math.atan2", args, 2, 2)?;
            Ok(Value::Float(args[0].as_f64()?.atan2(args[1].as_f64()?)))
        })),
    );
    m.insert("radians".into(), unary_float("radians", f64::to_radians));
    m.insert("degrees".into(), unary_float("degrees", f64::to_degrees));

    m.insert("is_inf".into(), unary_bool("is_inf", |x| x.is_infinite()));
    m.insert("is_nan".into(), unary_bool("is_nan", |x| x.is_nan()));
    m.insert(
        "is_finite".into(),
        unary_bool("is_finite", |x| x.is_finite()),
    );

    Value::module(m)
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::assert_eval;
    use crate::core::value::Value;

    #[test]
    fn test_abs() {
        assert_eval("math.abs(-3)", &[], Value::Int(3));
        assert_eval("math.abs(-2.5)", &[], Value::Float(2.5));
    }

    #[test]
    fn test_floor() {
        assert_eval("math.floor(2.9)", &[], Value::Int(2));
        assert_eval("math.floor(-2.1)", &[], Value::Int(-3));
    }

    #[test]
    fn test_ceil() {
        assert_eval("math.ceil(2.1)", &[], Value::Int(3));
        assert_eval("math.ceil(-2.9)", &[], Value::Int(-2));
    }

    #[test]
    fn test_round() {
        assert_eval("math.round(2.4)", &[], Value::Int(2));
        assert_eval("math.round(2.6)", &[], Value::Int(3));
    }

    #[test]
    fn test_log() {
        assert_eval("math.log(math.e)", &[], Value::Float(1.0));
        assert_eval("math.log(8.0, 2.0)", &[], Value::Float(3.0));
    }

    #[test]
    fn test_atan2() {
        // verifies (y, x) argument order
        assert_eval(
            "math.atan2(1.0, 0.0)",
            &[],
            Value::Float(std::f64::consts::FRAC_PI_2),
        );
        assert_eval("math.atan2(0.0, 1.0)", &[], Value::Float(0.0));
    }
}
