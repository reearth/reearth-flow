use crate::core::error::InnerResult;
use crate::core::eval::{default_env, env_bind, eval, eval_eq};
use crate::core::parser::parse;
use crate::core::value::Value;
use crate::Result;

pub(crate) fn values_equal(a: &Value, b: &Value) -> InnerResult<bool> {
    eval_eq(a.clone(), b.clone())
}

pub(crate) fn try_run(input: &str, vars: &[(&str, Value)]) -> Result<Value> {
    let env = default_env();
    for (k, v) in vars {
        env_bind(&env, *k, v.clone());
    }
    eval(&parse(input).unwrap(), &env)
}

pub(crate) fn run(input: &str, vars: &[(&str, Value)]) -> Value {
    try_run(input, vars).unwrap()
}

#[track_caller]
pub(crate) fn assert_val(a: &Value, b: &Value) {
    assert!(
        values_equal(a, b).expect("values_equal failed"),
        "\nleft:  {a:?}\nright: {b:?}"
    );
}

#[track_caller]
pub(crate) fn assert_eval(input: &str, vars: &[(&str, Value)], expected: Value) {
    assert_val(&run(input, vars), &expected);
}
