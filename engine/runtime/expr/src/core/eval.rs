use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::ast::{BinOp, Expr, ExprKind, UnaryOp};
use super::builtins::{builtin_itertools, builtin_json, builtin_math, regex_type_value};
use super::builtins::{dict as dict_methods, list as list_methods, str as str_methods};
use super::env::{new_frame, Env};
use super::error::{eval_error, Error, Result, POS_UNSET};
use super::value::{format_float, ClosureValue, NativeFn, TypeValue, Value};
use crate::expect_arity;

struct DepthCounter {
    depth: Cell<usize>,
    limit: usize,
    label: &'static str,
}

thread_local! {
    static AST_DEPTH: DepthCounter = const { DepthCounter {
        depth: Cell::new(0),
        #[cfg(debug_assertions)]
        limit: 64,
        #[cfg(not(debug_assertions))]
        limit: 1024,
        label: "AST",
    } };
    static CALL_DEPTH: DepthCounter = const { DepthCounter {
        depth: Cell::new(0),
        #[cfg(debug_assertions)]
        limit: 32,
        #[cfg(not(debug_assertions))]
        limit: 512,
        label: "call",
    } };
}

struct DepthGuard {
    counter: &'static std::thread::LocalKey<DepthCounter>,
}

impl DepthGuard {
    fn enter(counter: &'static std::thread::LocalKey<DepthCounter>) -> Result<Self> {
        let (depth, limit, label) = counter.with(|c| {
            let v = c.depth.get() + 1;
            c.depth.set(v);
            (v, c.limit, c.label)
        });
        if depth > limit {
            counter.with(|c| c.depth.set(c.depth.get() - 1));
            Err(eval_error(format!(
                "expression exceeds maximum {label} depth ({limit})"
            )))
        } else {
            Ok(DepthGuard { counter })
        }
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        self.counter.with(|c| c.depth.set(c.depth.get() - 1));
    }
}

/// Walk the frame chain and return a clone of the first binding found, or None.
pub(crate) fn env_get(env: &Env, name: &str) -> Option<Value> {
    let frame = env.borrow();
    if let Some(v) = frame.bindings.get(name) {
        return Some(v.clone());
    }
    let parent = frame.parent.clone();
    drop(frame);
    parent.as_ref().and_then(|p| env_get(p, name))
}

/// Walk the frame chain and update the first frame that owns `name`.
/// If not found anywhere, create the binding in the innermost frame.
/// Root frames (no parent) are the immutable builtin frame and are never written to.
fn env_set_upward(env: &Env, name: String, val: Value) {
    let mut cursor = Rc::clone(env);
    loop {
        let parent = {
            let frame = cursor.borrow();
            if frame.parent.is_some() && frame.bindings.contains_key(&name) {
                drop(frame);
                cursor.borrow_mut().bindings.insert(name, val);
                return;
            }
            frame.parent.clone()
        };
        match parent {
            Some(p) => cursor = p,
            None => break,
        }
    }
    env.borrow_mut().bindings.insert(name, val);
}

/// Always create/overwrite the binding in the innermost frame (`let` semantics).
fn env_set_local(env: &Env, name: String, val: Value) {
    env.borrow_mut().bindings.insert(name, val);
}

/// Insert a value into the innermost frame. Intended for seeding an env from
/// Rust (e.g. `default_env`, test helpers, external callers).
pub fn env_bind(env: &Env, name: impl Into<String>, val: Value) {
    env.borrow_mut().bindings.insert(name.into(), val);
}

/// Remove a binding from the innermost frame.
pub fn env_remove(env: &Env, name: &str) {
    env.borrow_mut().bindings.remove(name);
}

fn int_resolve_method(recv: Value, attr: &str) -> Result<NativeFn> {
    let Value::Int(n) = recv else {
        return Err(eval_error("int method called on non-int receiver"));
    };
    match attr {
        "bit_length" => Ok(NativeFn::new(move |args| {
            expect_arity("int.bit_length", args, 0, 0)?;
            if n < 0 {
                return Err(eval_error(
                    "bit_length() not supported for negative integers",
                ));
            }
            Ok(Value::Int((i64::BITS - n.leading_zeros()) as i64))
        })),
        _ => Err(eval_error(format!("int has no attribute '{attr}'"))),
    }
}

thread_local! {
    static NULL_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("nullType", None));
    static BOOL_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("bool", Some(NativeFn::new(builtin_bool))));
    static INT_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("int", Some(NativeFn::new(builtin_int))).with_method_resolver(int_resolve_method));
    static FLOAT_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("float", Some(NativeFn::new(builtin_float))));
    static STR_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("str", Some(NativeFn::new(builtin_str))).with_method_resolver(str_methods::resolve_method));
    static LIST_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("list", Some(NativeFn::new(builtin_list))).with_method_resolver(list_methods::resolve_method));
    static DICT_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("dict", Some(NativeFn::new(builtin_dict))).with_method_resolver(dict_methods::resolve_method));
    static FN_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("function", None));
    static MODULE_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("module", None));
    static TYPE_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new("type", Some(NativeFn::new(builtin_type))));
}

thread_local! {
    static BUILTIN_ENV: Env = {
        let env = new_frame(None);
        env_bind(&env, "str", Value::Type(STR_TYPE.with(Rc::clone)));
        env_bind(&env, "int", Value::Type(INT_TYPE.with(Rc::clone)));
        env_bind(&env, "float", Value::Type(FLOAT_TYPE.with(Rc::clone)));
        env_bind(&env, "bool", Value::Type(BOOL_TYPE.with(Rc::clone)));
        env_bind(&env, "list", Value::Type(LIST_TYPE.with(Rc::clone)));
        env_bind(&env, "dict", Value::Type(DICT_TYPE.with(Rc::clone)));
        env_bind(&env, "Regex", Value::Type(regex_type_value()));
        env_bind(&env, "math", builtin_math());
        env_bind(&env, "print", Value::Fn(NativeFn::new(builtin_print)));
        env_bind(&env, "type", Value::Type(TYPE_TYPE.with(Rc::clone)));
        env_bind(&env, "len", Value::Fn(NativeFn::new(builtin_len)));
        env_bind(&env, "itertools", builtin_itertools());
        env_bind(&env, "json", builtin_json());
        env_bind(&env, "range", Value::Fn(NativeFn::new(builtin_range)));
        env
    };
}

pub fn default_env() -> Env {
    BUILTIN_ENV.with(|base| new_frame(Some(Rc::clone(base))))
}

pub fn eval(expr: &Expr, env: &Env) -> Result<Value> {
    match eval_inner(expr, env) {
        Err(Error::Return(v)) => Ok(v),
        other => other,
    }
}

/// Unified callable dispatch: invokes a NativeFn, Type constructor, or Closure by value.
pub(crate) fn call_value(f: Value, args: Vec<Value>) -> Result<Value> {
    let _guard = DepthGuard::enter(&CALL_DEPTH)?;
    match f {
        Value::Fn(native) => native.call(&args),
        Value::Type(tv) => tv.call_ctor(&args),
        Value::Closure(cl) => {
            if args.len() != cl.params.len() {
                return Err(eval_error(format!(
                    "closure expects {} argument(s), got {}",
                    cl.params.len(),
                    args.len()
                )));
            }
            let captured = cl
                .captured
                .upgrade()
                .ok_or_else(|| eval_error("closure called after its defining scope was dropped"))?;
            let call_env = new_frame(Some(captured));
            for (param, arg) in cl.params.iter().zip(args) {
                env_set_local(&call_env, param.clone(), arg);
            }
            match eval_inner(&cl.body, &call_env) {
                Err(Error::Return(v)) => Ok(v),
                other => other,
            }
        }
        other => Err(eval_error(format!(
            "value of type {} is not callable",
            other.type_name()
        ))),
    }
}

pub(crate) fn eval_eq(a: Value, b: Value) -> Result<bool> {
    match call_value(Value::Fn(NativeFn::new(eq_op)), vec![a, b])? {
        Value::Bool(b) => Ok(b),
        _ => Err(eval_error("__eq__ must return a bool")),
    }
}

fn eq_op(args: &[Value]) -> Result<Value> {
    let [a, b] = args else {
        return Err(eval_error("== requires two operands"));
    };
    if let Value::Object(rc) = a {
        return rc.call_method("__eq__", std::slice::from_ref(b));
    }
    match (a, b) {
        (Value::List(a), Value::List(b)) => list_methods::eq_inner(a, b).map(Value::Bool),
        (Value::Dict(a), Value::Dict(b)) => dict_methods::eq_inner(a, b).map(Value::Bool),
        _ => Ok(Value::Bool(primitive_eq(a, b))),
    }
}

fn resolve_attr(recv: Value, attr: &str) -> Result<Value> {
    match recv {
        Value::Module(m) => m
            .get(attr)
            .cloned()
            .ok_or_else(|| eval_error(format!("module has no attribute '{attr}'"))),
        Value::Type(tv) => match tv.resolve_method {
            Some(f) => {
                let attr = attr.to_string();
                Ok(Value::Fn(NativeFn::new(move |args| {
                    let [recv, rest @ ..] = args else {
                        return Err(eval_error(format!(
                            "unbound method '{attr}' requires an instance as first argument"
                        )));
                    };
                    f(recv.clone(), &attr)?.call(rest)
                })))
            }
            None => Err(eval_error(format!(
                "type '{}' has no attribute '{attr}'",
                tv.name
            ))),
        },
        Value::Object(rc) => {
            if let Some(result) = rc.get_property(attr) {
                return result;
            }
            let attr = attr.to_string();
            Ok(Value::Fn(NativeFn::new(move |args| {
                rc.call_method(&attr, args)
            })))
        }
        recv => {
            let tv = type_of(&recv);
            match tv.resolve_method {
                Some(f) => f(recv, attr).map(Value::Fn),
                None => Err(eval_error(format!("{} has no attribute '{attr}'", tv.name))),
            }
        }
    }
}

pub(super) fn value_add(left: Value, right: Value) -> Result<Value> {
    if let Value::Object(rc) = &left {
        return rc.call_method("__add__", &[right]);
    }
    match (left, right) {
        (Value::String(a), Value::String(b)) => Ok(Value::String(a + b.as_str())),
        (Value::List(a), Value::List(b)) => {
            let mut new_vec = a.borrow().clone();
            new_vec.extend(b.borrow().iter().cloned());
            Ok(Value::list(new_vec))
        }
        (a, b) => match coerce_numeric(&a, &b) {
            Ok((Numeric::Int(a), Numeric::Int(b))) => {
                a.checked_add(b).map(Value::Int).ok_or_else(int_overflow)
            }
            Ok((Numeric::Float(a), Numeric::Float(b))) => Ok(Value::Float(a + b)),
            Ok(_) => unreachable!(),
            Err(_) => Err(binop_type_error("+", &a, &b)),
        },
    }
}

// Returns a NativeFn for a binary operator. args[0]=left, args[1]=right.
fn resolve_op(op: &BinOp) -> NativeFn {
    match op {
        BinOp::Add => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            value_add(left, right)
        }),
        BinOp::Sub => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__sub__", &[right]);
            }
            numeric_op(
                left,
                right,
                |a, b| a.checked_sub(b).ok_or_else(int_overflow),
                |a, b| a - b,
            )
        }),
        BinOp::Mul => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__mul__", &[right]);
            }
            numeric_op(
                left,
                right,
                |a, b| a.checked_mul(b).ok_or_else(int_overflow),
                |a, b| a * b,
            )
        }),
        BinOp::Div => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__div__", &[right]);
            }
            match coerce_numeric(&left, &right) {
                Ok((Numeric::Int(a), Numeric::Int(b))) => {
                    if b == 0 {
                        return Err(eval_error("division by zero"));
                    }
                    Ok(Value::Float(a as f64 / b as f64))
                }
                Ok((Numeric::Float(a), Numeric::Float(b))) => {
                    if b == 0.0 {
                        return Err(eval_error("division by zero"));
                    }
                    Ok(Value::Float(a / b))
                }
                Ok(_) => unreachable!(),
                Err(_) => Err(binop_type_error("/", &left, &right)),
            }
        }),
        BinOp::FloorDiv => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__floordiv__", &[right]);
            }
            match coerce_numeric(&left, &right) {
                Ok((Numeric::Int(a), Numeric::Int(b))) => {
                    if b == 0 {
                        return Err(eval_error("division by zero"));
                    }
                    let d = a.checked_div(b).ok_or_else(int_overflow)?;
                    let r = a.checked_rem(b).ok_or_else(int_overflow)?;
                    let floored = if r != 0 && (r < 0) != (b < 0) {
                        d.checked_sub(1).ok_or_else(int_overflow)?
                    } else {
                        d
                    };
                    Ok(Value::Int(floored))
                }
                Ok((Numeric::Float(a), Numeric::Float(b))) => {
                    if b == 0.0 {
                        return Err(eval_error("division by zero"));
                    }
                    Ok(Value::Float((a / b).floor()))
                }
                Ok(_) => unreachable!(),
                Err(_) => Err(binop_type_error("//", &left, &right)),
            }
        }),
        BinOp::Mod => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__mod__", &[right]);
            }
            match coerce_numeric(&left, &right) {
                Ok((Numeric::Int(a), Numeric::Int(b))) => {
                    if b == 0 {
                        return Err(eval_error("modulo by zero"));
                    }
                    if b == -1 {
                        return Ok(Value::Int(0));
                    }
                    let r = a % b;
                    let result = if r != 0 && (r < 0) != (b < 0) {
                        r + b
                    } else {
                        r
                    };
                    Ok(Value::Int(result))
                }
                Ok((Numeric::Float(a), Numeric::Float(b))) => {
                    if b == 0.0 {
                        return Err(eval_error("modulo by zero"));
                    }
                    let r = a % b;
                    let result = if r != 0.0 && (r < 0.0) != (b < 0.0) {
                        r + b
                    } else {
                        r
                    };
                    Ok(Value::Float(result))
                }
                Ok(_) => unreachable!(),
                Err(_) => Err(binop_type_error("%", &left, &right)),
            }
        }),
        BinOp::Pow => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__pow__", &[right]);
            }
            match coerce_numeric(&left, &right) {
                Ok((Numeric::Int(a), Numeric::Int(b))) => {
                    if b < 0 {
                        Ok(Value::Float((a as f64).powf(b as f64)))
                    } else if b > u32::MAX as i64 {
                        Err(eval_error("exponent too large"))
                    } else {
                        a.checked_pow(b as u32)
                            .map(Value::Int)
                            .ok_or_else(int_overflow)
                    }
                }
                Ok((Numeric::Float(a), Numeric::Float(b))) => Ok(Value::Float(a.powf(b))),
                Ok(_) => unreachable!(),
                Err(_) => Err(binop_type_error("**", &left, &right)),
            }
        }),
        BinOp::Eq => NativeFn::new(eq_op),
        BinOp::Ne => NativeFn::new(|args| match eq_op(args)? {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(eval_error("__eq__ must return a bool")),
        }),
        BinOp::Lt => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__lt__", &[right]);
            }
            compare_values(left, right, |o| o == std::cmp::Ordering::Less)
        }),
        BinOp::Le => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__le__", &[right]);
            }
            compare_values(left, right, |o| o != std::cmp::Ordering::Greater)
        }),
        BinOp::Gt => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__gt__", &[right]);
            }
            compare_values(left, right, |o| o == std::cmp::Ordering::Greater)
        }),
        BinOp::Ge => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__ge__", &[right]);
            }
            compare_values(left, right, |o| o != std::cmp::Ordering::Less)
        }),
        BinOp::In => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            contains_inner(left, right).map(Value::Bool)
        }),
        BinOp::NotIn => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            contains_inner(left, right).map(|b| Value::Bool(!b))
        }),
        BinOp::BitAnd => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__and__", &[right]);
            }
            let (a, b) = bitwise_args(&left, &right)?;
            Ok(Value::Int(a & b))
        }),
        BinOp::BitOr => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__or__", &[right]);
            }
            let (a, b) = bitwise_args(&left, &right)?;
            Ok(Value::Int(a | b))
        }),
        BinOp::BitXor => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__xor__", &[right]);
            }
            let (a, b) = bitwise_args(&left, &right)?;
            Ok(Value::Int(a ^ b))
        }),
        BinOp::Shl => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__lshift__", &[right]);
            }
            let (a, b) = bitwise_args(&left, &right)?;
            if b >= 63 {
                return Err(eval_error(format!(
                    "left shift amount {b} out of range [0, 62]"
                )));
            }
            let result = a
                .checked_shl(b as u32)
                .filter(|&v| v >= 0)
                .ok_or_else(|| eval_error("left shift result overflows 63-bit integer"))?;
            Ok(Value::Int(result))
        }),
        BinOp::Shr => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__rshift__", &[right]);
            }
            let (a, b) = bitwise_args(&left, &right)?;
            if b >= 63 {
                // shr should not overflow, same as Python
                return Ok(Value::Int(0));
            }
            Ok(Value::Int(a >> b))
        }),
        BinOp::And | BinOp::Or => {
            unreachable!("and/or are short-circuited in eval_inner before resolve_op is called")
        }
    }
}

fn contains_inner(left: Value, right: Value) -> Result<bool> {
    match right {
        Value::List(arr) => {
            let arr = arr.borrow();
            for v in arr.iter() {
                if eval_eq(v.clone(), left.clone())? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::String(haystack) => match left {
            Value::String(s) => Ok(haystack.contains(s.as_str())),
            l => Err(eval_error(format!(
                "'in' not supported between {} and string",
                l.type_name()
            ))),
        },
        Value::Dict(map) => match left {
            Value::String(key) => Ok(map.borrow().contains_key(&key)),
            l => Err(eval_error(format!(
                "'in' not supported between {} and dict",
                l.type_name()
            ))),
        },
        Value::Object(rc) => rc
            .call_method("__contains__", &[left])
            .and_then(|v| match v {
                Value::Bool(b) => Ok(b),
                other => Err(eval_error(format!(
                    "__contains__ must return bool, got {}",
                    other.type_name()
                ))),
            }),
        r => Err(eval_error(format!(
            "'in' not supported between {} and {}",
            left.type_name(),
            r.type_name()
        ))),
    }
}

// Returns a NativeFn for a unary operator. args[0] is the operand.
fn resolve_unary_op(op: &UnaryOp) -> NativeFn {
    match op {
        UnaryOp::Not => NativeFn::new(|args| {
            let val = unary_arg(args)?;
            Ok(Value::Bool(!val.is_truthy()))
        }),
        UnaryOp::Neg => NativeFn::new(|args| {
            let val = unary_arg(args)?;
            match val {
                Value::Int(n) => n
                    .checked_neg()
                    .map(Value::Int)
                    .ok_or_else(|| eval_error("integer overflow")),
                Value::Float(f) => Ok(Value::Float(-f)),
                v => Err(eval_error(format!("cannot negate {}", v.type_name()))),
            }
        }),
    }
}

fn binary_args(args: &[Value]) -> Result<(Value, Value)> {
    let [a, b] = args else {
        return Err(eval_error("binary operator requires two operands"));
    };
    Ok((a.clone(), b.clone()))
}

fn unary_arg(args: &[Value]) -> Result<&Value> {
    args.first()
        .ok_or_else(|| eval_error("unary operator requires one operand"))
}

fn bitwise_args(a: &Value, b: &Value) -> Result<(i64, i64)> {
    let to_bits = |v: &Value| match v {
        Value::Int(n) if *n >= 0 => Ok(*n),
        Value::Int(_) => Err(eval_error("bitwise operands must be non-negative integers")),
        other => Err(eval_error(format!(
            "bitwise operands must be non-negative integers, got {}",
            other.type_name()
        ))),
    };
    Ok((to_bits(a)?, to_bits(b)?))
}

// Recursion entrypoint for AST expression evaluation.
fn eval_inner(expr: &Expr, env: &Env) -> Result<Value> {
    let pos = expr.span.start;
    let _depth = DepthGuard::enter(&AST_DEPTH)?;
    eval_node(expr, env).map_err(|mut e| {
        if let Error::Eval { pos: ref mut p, .. } = e {
            if *p == POS_UNSET {
                *p = pos;
            }
        }
        e
    })
}

fn eval_node(expr: &Expr, env: &Env) -> Result<Value> {
    let pos = expr.span.start;
    match &expr.kind {
        ExprKind::Null => Ok(Value::Null),
        ExprKind::Bool(b) => Ok(Value::Bool(*b)),
        ExprKind::Int(n) => Ok(Value::Int(*n)),
        ExprKind::Float(f) => Ok(Value::Float(*f)),
        ExprKind::Str(s) => Ok(Value::String(s.clone())),
        ExprKind::Array(items) => {
            let mut values = Vec::with_capacity(items.len());
            for e in items {
                values.push(eval_inner(e, env)?);
            }
            Ok(Value::list(values))
        }
        ExprKind::Map(entries) => {
            let mut map = IndexMap::new();
            for (k, v) in entries {
                let key = eval_inner(k, env)?;
                let key_str = match key {
                    Value::String(s) => s,
                    other => {
                        return Err(Error::Eval {
                            pos,
                            msg: format!("dict key must be a string, got {}", other.type_name()),
                        })
                    }
                };
                map.insert(key_str, eval_inner(v, env)?);
            }
            Ok(Value::dict(map))
        }
        ExprKind::Var(name) => env_get(env, name.as_str()).ok_or_else(|| Error::Eval {
            pos,
            msg: format!("'{name}' is not defined"),
        }),
        ExprKind::Index(target, key) => {
            let target = eval_inner(target, env)?;
            let key = eval_inner(key, env)?;
            eval_index(target, key)
        }
        ExprKind::Slice {
            target,
            start,
            stop,
            step,
        } => {
            let target = eval_inner(target, env)?;
            let start = start.as_deref().map(|e| eval_inner(e, env)).transpose()?;
            let stop = stop.as_deref().map(|e| eval_inner(e, env)).transpose()?;
            let step = step.as_deref().map(|e| eval_inner(e, env)).transpose()?;
            eval_slice(target, start, stop, step)
        }
        ExprKind::Unary(op, e) => {
            let val = eval_inner(e, env)?;
            call_value(Value::Fn(resolve_unary_op(op)), vec![val])
        }
        ExprKind::Attribute { receiver, attr } => {
            let recv = eval_inner(receiver, env)?;
            resolve_attr(recv, attr)
        }
        ExprKind::Call { callee, args } => {
            let f = eval_inner(callee, env)?;
            let evaled = args
                .iter()
                .map(|a| eval_inner(a, env))
                .collect::<Result<Vec<_>>>()?;
            call_value(f, evaled)
        }
        ExprKind::Binary(left, op, right) => {
            match op {
                BinOp::And => {
                    let l = eval_inner(left, env)?;
                    if !l.is_truthy() {
                        return Ok(l);
                    }
                    return eval_inner(right, env);
                }
                BinOp::Or => {
                    let l = eval_inner(left, env)?;
                    if l.is_truthy() {
                        return Ok(l);
                    }
                    return eval_inner(right, env);
                }
                _ => {}
            }
            let left = eval_inner(left, env)?;
            let right = eval_inner(right, env)?;
            call_value(Value::Fn(resolve_op(op)), vec![left, right])
        }
        ExprKind::Assign { lvalue, value } => {
            let v = eval_inner(value, env)?;
            eval_assign_lvalue(lvalue, v.clone(), env, false)?;
            Ok(v)
        }
        ExprKind::Let { lvalue, value } => {
            let v = eval_inner(value, env)?;
            eval_assign_lvalue(lvalue, v.clone(), env, true)?;
            Ok(v)
        }
        ExprKind::CompoundAssign { lvalue, op, rhs } => {
            let new_val = eval_compound_assign(lvalue, op, rhs, env, pos)?;
            Ok(new_val)
        }
        ExprKind::Block(exprs) => {
            let mut result = Value::Null;
            for e in exprs {
                result = eval_inner(e, env)?;
            }
            Ok(result)
        }
        ExprKind::If { cond, then, else_ } => {
            let c = eval_inner(cond, env)?;
            if c.is_truthy() {
                eval_inner(then, env)
            } else {
                eval_inner(else_, env)
            }
        }
        // No iteration cap — see docs/design.md#no-while-iteration-limit
        ExprKind::While { cond, body } => {
            loop {
                let c = eval_inner(cond, env)?;
                if !c.is_truthy() {
                    break;
                }
                eval_inner(body, env)?;
            }
            Ok(Value::Null)
        }
        ExprKind::ForIn {
            var,
            iterable,
            body,
        } => {
            let iter_val = eval_inner(iterable, env)?;
            let items = collect_iterable(iter_val, pos)?;
            for item in items {
                eval_assign_lvalue(var, item, env, false)?;
                eval_inner(body, env)?;
            }
            Ok(Value::Null)
        }
        ExprKind::Return(expr) => {
            let val = match expr {
                Some(e) => eval_inner(e, env)?,
                None => Value::Null,
            };
            Err(Error::Return(val))
        }
        ExprKind::Fn { params, body } => Ok(Value::Closure(ClosureValue {
            params: params.clone(),
            body: Rc::new(*body.clone()),
            // TODO: currently capture whole parent frame weakly,
            // should do escape analysis for per-variable capturing and support optional strong capturing
            captured: Rc::downgrade(env),
        })),
    }
}

fn collect_iterable(value: Value, pos: usize) -> Result<Vec<Value>> {
    match value {
        Value::List(rc) => Ok(rc.borrow().clone()),
        Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        Value::Dict(rc) => Ok(rc
            .borrow()
            .keys()
            .map(|k| Value::String(k.clone()))
            .collect()),
        Value::Object(rc) => match rc.call_method("__iter__", &[])? {
            Value::List(arr) => Ok(arr.borrow().clone()),
            v => Err(Error::Eval {
                pos,
                msg: format!("__iter__ must return a list, got {}", v.type_name()),
            }),
        },
        v => Err(Error::Eval {
            pos,
            msg: format!("{} is not iterable", v.type_name()),
        }),
    }
}

/// Assign `value` to `lvalue`.
/// `local = true` uses `let` semantics (always bind in innermost frame).
/// `local = false` uses walk-upward semantics (find existing binding or create local).
fn eval_assign_lvalue(lvalue: &Expr, value: Value, env: &Env, local: bool) -> Result<()> {
    let pos = lvalue.span.start;
    match &lvalue.kind {
        ExprKind::Var(name) => {
            if local {
                env_set_local(env, name.clone(), value);
            } else {
                env_set_upward(env, name.clone(), value);
            }
            Ok(())
        }
        ExprKind::Index(target, key) => {
            let container = eval_inner(target, env)?;
            let key_val = eval_inner(key, env)?;
            match (container, &key_val) {
                (Value::List(rc), Value::Int(i)) => {
                    let len = rc.borrow().len();
                    let idx = list_methods::resolve_index(*i, len).ok_or_else(|| Error::Eval {
                        pos,
                        msg: format!("list index {} out of range (len {})", i, len),
                    })?;
                    rc.borrow_mut()[idx] = value;
                }
                (Value::Dict(rc), Value::String(k)) => {
                    rc.borrow_mut().insert(k.clone(), value);
                }
                (c, k) => {
                    return Err(Error::Eval {
                        pos,
                        msg: format!(
                            "cannot index-assign {} with {}",
                            c.type_name(),
                            k.type_name()
                        ),
                    })
                }
            }
            Ok(())
        }
        ExprKind::Array(targets) => {
            let items = collect_iterable(value, pos)?;
            if items.len() != targets.len() {
                return Err(Error::Eval {
                    pos,
                    msg: format!(
                        "unpack mismatch (expected {}, got {})",
                        targets.len(),
                        items.len()
                    ),
                });
            }
            for (target, item) in targets.iter().zip(items) {
                eval_assign_lvalue(target, item, env, local)?;
            }
            Ok(())
        }
        _ => Err(Error::Eval {
            pos,
            msg: "invalid assignment target".into(),
        }),
    }
}

fn eval_compound_assign(
    lvalue: &Expr,
    op: &BinOp,
    rhs: &Expr,
    env: &Env,
    pos: usize,
) -> Result<Value> {
    let rhs_val = eval_inner(rhs, env)?;
    let f = resolve_op(op);
    match &lvalue.kind {
        ExprKind::Var(name) => {
            let current = env_get(env, name).ok_or_else(|| Error::Eval {
                pos,
                msg: format!("undefined variable '{name}'"),
            })?;
            let new_val = call_value(Value::Fn(f.clone()), vec![current, rhs_val])?;
            env_set_upward(env, name.clone(), new_val.clone());
            Ok(new_val)
        }
        ExprKind::Index(target, key) => {
            let container = eval_inner(target, env)?;
            let key_val = eval_inner(key, env)?;
            match (container, &key_val) {
                (Value::List(rc), Value::Int(i)) => {
                    let len = rc.borrow().len();
                    let idx = list_methods::resolve_index(*i, len).ok_or_else(|| Error::Eval {
                        pos,
                        msg: format!("list index {} out of range (len {})", i, len),
                    })?;
                    let current = rc.borrow()[idx].clone();
                    let new_val = call_value(Value::Fn(f.clone()), vec![current, rhs_val])?;
                    rc.borrow_mut()[idx] = new_val.clone();
                    Ok(new_val)
                }
                (Value::Dict(rc), Value::String(k)) => {
                    let current = rc.borrow().get(k.as_str()).cloned().unwrap_or(Value::Null);
                    let new_val = call_value(Value::Fn(f.clone()), vec![current, rhs_val])?;
                    rc.borrow_mut().insert(k.clone(), new_val.clone());
                    Ok(new_val)
                }
                (c, k) => Err(Error::Eval {
                    pos,
                    msg: format!(
                        "cannot index-assign {} with {}",
                        c.type_name(),
                        k.type_name()
                    ),
                }),
            }
        }
        _ => Err(Error::Eval {
            pos,
            msg: "invalid assignment target".into(),
        }),
    }
}

fn eval_index(target: Value, key: Value) -> Result<Value> {
    match (target, key) {
        (Value::Dict(map), Value::String(k)) => map
            .borrow()
            .get(&k)
            .cloned()
            .ok_or_else(|| eval_error(format!("dict key '{k}' not found"))),
        (Value::List(arr), Value::Int(i)) => {
            let arr = arr.borrow();
            let len = arr.len();
            list_methods::resolve_index(i, len)
                .map(|pos| arr[pos].clone())
                .ok_or_else(|| eval_error(format!("list index {i} out of range (len {len})")))
        }
        (Value::String(s), Value::Int(i)) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len();
            list_methods::resolve_index(i, len)
                .map(|pos| Value::String(chars[pos].to_string()))
                .ok_or_else(|| eval_error(format!("string index {i} out of range (len {len})")))
        }
        (Value::Object(rc), key) => rc.call_method("__getitem__", &[key]),
        (target, key) => Err(eval_error(format!(
            "cannot index {} with {}",
            target.type_name(),
            key.type_name()
        ))),
    }
}

fn as_slice_index(v: Value, what: &str) -> Result<i64> {
    match v {
        Value::Int(n) => Ok(n),
        v => Err(eval_error(format!(
            "slice {what} must be an integer, got {}",
            v.type_name()
        ))),
    }
}

fn slice_indices(len: usize, start: Option<i64>, stop: Option<i64>, step: i64) -> Vec<usize> {
    let n = len as i64;
    let normalize = |i: i64, clamp_lo: i64, clamp_hi: i64| -> i64 {
        let i = if i < 0 { i + n } else { i };
        i.clamp(clamp_lo, clamp_hi)
    };
    let (start, stop) = if step > 0 {
        let s = start.map(|i| normalize(i, 0, n)).unwrap_or(0);
        let e = stop.map(|i| normalize(i, 0, n)).unwrap_or(n);
        (s, e)
    } else {
        let s = start.map(|i| normalize(i, -1, n - 1)).unwrap_or(n - 1);
        let e = stop.map(|i| normalize(i, -1, n - 1)).unwrap_or(-1);
        (s, e)
    };
    let mut indices = Vec::new();
    let mut i = start;
    if step > 0 {
        while i < stop {
            indices.push(i as usize);
            i = i.saturating_add(step);
        }
    } else {
        while i > stop {
            indices.push(i as usize);
            i = i.saturating_add(step);
        }
    }
    indices
}

fn eval_slice(
    target: Value,
    start: Option<Value>,
    stop: Option<Value>,
    step: Option<Value>,
) -> Result<Value> {
    let step = match step {
        None => 1i64,
        Some(v) => as_slice_index(v, "step")?,
    };
    if step == 0 {
        return Err(eval_error("slice step cannot be zero"));
    }
    let start = start.map(|v| as_slice_index(v, "start")).transpose()?;
    let stop = stop.map(|v| as_slice_index(v, "stop")).transpose()?;

    match target {
        Value::List(arr) => {
            let arr = arr.borrow();
            let indices = slice_indices(arr.len(), start, stop, step);
            Ok(Value::list(
                indices.into_iter().map(|i| arr[i].clone()).collect(),
            ))
        }
        Value::String(s) => {
            let chars: Vec<char> = s.chars().collect();
            let indices = slice_indices(chars.len(), start, stop, step);
            Ok(Value::String(
                indices.into_iter().map(|i| chars[i]).collect(),
            ))
        }
        v => Err(eval_error(format!("cannot slice {}", v.type_name()))),
    }
}

pub(crate) enum Numeric {
    Int(i64),
    Float(f64),
}

pub(crate) fn coerce_numeric(a: &Value, b: &Value) -> Result<(Numeric, Numeric)> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok((Numeric::Int(*a), Numeric::Int(*b))),
        (Value::Int(a), Value::Float(b)) => Ok((Numeric::Float(*a as f64), Numeric::Float(*b))),
        (Value::Float(a), Value::Int(b)) => Ok((Numeric::Float(*a), Numeric::Float(*b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok((Numeric::Float(*a), Numeric::Float(*b))),
        (a, b) => Err(eval_error(format!(
            "cannot apply numeric op to {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn int_overflow() -> Error {
    eval_error("integer overflow")
}

fn numeric_op(
    left: Value,
    right: Value,
    int_op: impl Fn(i64, i64) -> Result<i64>,
    float_op: impl Fn(f64, f64) -> f64,
) -> Result<Value> {
    match coerce_numeric(&left, &right)? {
        (Numeric::Int(a), Numeric::Int(b)) => Ok(Value::Int(int_op(a, b)?)),
        (Numeric::Float(a), Numeric::Float(b)) => Ok(Value::Float(float_op(a, b))),
        _ => unreachable!(),
    }
}

fn binop_type_error(op: &str, l: &Value, r: &Value) -> Error {
    eval_error(format!(
        "'{op}' not supported between {} and {}",
        l.type_name(),
        r.type_name()
    ))
}

fn compare_values(
    left: Value,
    right: Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value> {
    let ord = match coerce_numeric(&left, &right) {
        Ok((Numeric::Int(a), Numeric::Int(b))) => a.cmp(&b),
        Ok((Numeric::Float(a), Numeric::Float(b))) => match a.partial_cmp(&b) {
            Some(ord) => ord,
            None => return Ok(Value::Bool(false)),
        },
        Ok(_) => unreachable!(),
        Err(_) => match (&left, &right) {
            (Value::String(a), Value::String(b)) => a.as_str().cmp(b.as_str()),
            (Value::List(a), Value::List(b)) => compare_arrays(&a.borrow(), &b.borrow())?,
            _ => {
                return Err(eval_error(format!(
                    "cannot compare {} and {}",
                    left.type_name(),
                    right.type_name()
                )))
            }
        },
    };
    Ok(Value::Bool(pred(ord)))
}

fn compare_arrays(a: &[Value], b: &[Value]) -> Result<std::cmp::Ordering> {
    for (x, y) in a.iter().zip(b.iter()) {
        let ord = match compare_values(x.clone(), y.clone(), |o| o == std::cmp::Ordering::Less)? {
            Value::Bool(true) => std::cmp::Ordering::Less,
            _ => {
                match compare_values(x.clone(), y.clone(), |o| o == std::cmp::Ordering::Greater)? {
                    Value::Bool(true) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            }
        };
        if ord != std::cmp::Ordering::Equal {
            return Ok(ord);
        }
    }
    Ok(a.len().cmp(&b.len()))
}

fn primitive_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y)) => *x == (*y as f64),
        (Value::Fn(a), Value::Fn(b)) => a.ptr_eq(b),
        (Value::Type(a), Value::Type(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

fn builtin_str(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(eval_error(format!(
            "str() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    match args.first() {
        None => Ok(Value::String(String::new())),
        Some(Value::Null) => Ok(Value::String("null".into())),
        Some(Value::String(s)) => Ok(Value::String(s.clone())),
        Some(Value::Bool(b)) => Ok(Value::String(b.to_string())),
        Some(Value::Int(n)) => Ok(Value::String(n.to_string())),
        Some(Value::Float(f)) => Ok(Value::String(format_float(*f))),
        Some(Value::Object(rc)) => rc.call_method("__str__", &[]),
        Some(v) => Ok(Value::String(v.to_string())),
    }
}

fn builtin_int(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(eval_error(format!(
            "int() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    match args.first() {
        None => Ok(Value::Int(0)),
        Some(Value::Int(n)) => Ok(Value::Int(*n)),
        Some(Value::Float(f)) => {
            let t = f.trunc();
            if !t.is_finite() || t < i64::MIN as f64 || t >= -(i64::MIN as f64) {
                Err(eval_error(format!("int() value out of range: {f}")))
            } else {
                Ok(Value::Int(t as i64))
            }
        }
        Some(Value::Bool(b)) => Ok(Value::Int(*b as i64)),
        Some(Value::String(s)) => s
            .trim()
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| eval_error(format!("int() cannot parse {s:?}"))),
        Some(v) => Err(eval_error(format!(
            "int() not supported for {}",
            v.type_name()
        ))),
    }
}

fn builtin_float(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(eval_error(format!(
            "float() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    match args.first() {
        None => Ok(Value::Float(0.0)),
        Some(Value::Float(f)) => Ok(Value::Float(*f)),
        Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
        Some(Value::Bool(b)) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Some(Value::String(s)) => s
            .trim()
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| eval_error(format!("float() cannot parse {s:?}"))),
        Some(v) => Err(eval_error(format!(
            "float() not supported for {}",
            v.type_name()
        ))),
    }
}

fn builtin_bool(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(eval_error(format!(
            "bool() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    Ok(Value::Bool(
        args.first().map(Value::is_truthy).unwrap_or(false),
    ))
}

fn builtin_list(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(eval_error(format!(
            "list() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    match args.first() {
        Some(Value::List(a)) => Ok(Value::list(a.borrow().clone())),
        Some(Value::String(s)) => Ok(Value::list(
            s.chars().map(|c| Value::String(c.to_string())).collect(),
        )),
        Some(Value::Dict(m)) => Ok(Value::list(
            m.borrow()
                .keys()
                .map(|k| Value::String(k.clone()))
                .collect(),
        )),
        Some(v) => Err(eval_error(format!(
            "list() not supported for {}",
            v.type_name()
        ))),
        None => Ok(Value::list(vec![])),
    }
}

fn builtin_dict(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(eval_error(format!(
            "dict() expects at most 1 argument, got {}",
            args.len()
        )));
    }
    match args.first() {
        None => return Ok(Value::dict(IndexMap::new())),
        Some(Value::Dict(m)) => return Ok(Value::dict(m.borrow().clone())),
        _ => {}
    }
    let pairs = match args.first() {
        Some(Value::List(a)) => a.borrow().clone(),
        _ => {
            return Err(eval_error(
                "dict() expects a dict, a list of [key, value] pairs, or no argument",
            ))
        }
    };
    let mut out = IndexMap::new();
    for (i, pair) in pairs.iter().enumerate() {
        match pair {
            Value::List(kv) => {
                let kv = kv.borrow();
                if kv.len() != 2 {
                    return Err(eval_error(format!(
                        "dict() entry at index {i} must be a 2-element list"
                    )));
                }
                let key = match &kv[0] {
                    Value::String(s) => s.clone(),
                    v => {
                        return Err(eval_error(format!(
                            "dict() key at index {i} must be a string, got {}",
                            v.type_name()
                        )))
                    }
                };
                out.insert(key, kv[1].clone());
            }
            _ => {
                return Err(eval_error(format!(
                    "dict() entry at index {i} must be a 2-element list"
                )))
            }
        }
    }
    Ok(Value::dict(out))
}

pub(crate) fn type_of(v: &Value) -> Rc<TypeValue> {
    match v {
        Value::Null => NULL_TYPE.with(Rc::clone),
        Value::Bool(_) => BOOL_TYPE.with(Rc::clone),
        Value::Int(_) => INT_TYPE.with(Rc::clone),
        Value::Float(_) => FLOAT_TYPE.with(Rc::clone),
        Value::String(_) => STR_TYPE.with(Rc::clone),
        Value::List(_) => LIST_TYPE.with(Rc::clone),
        Value::Dict(_) => DICT_TYPE.with(Rc::clone),
        Value::Fn(_) | Value::Closure(_) => FN_TYPE.with(Rc::clone),
        Value::Module(_) => MODULE_TYPE.with(Rc::clone),
        Value::Type(_) => TYPE_TYPE.with(Rc::clone),
        Value::Object(rc) => rc.type_object(),
    }
}

fn builtin_type(args: &[Value]) -> Result<Value> {
    expect_arity("type", args, 1, 1)?;
    Ok(Value::Type(type_of(&args[0])))
}

fn builtin_len(args: &[Value]) -> Result<Value> {
    expect_arity("len", args, 1, 1)?;
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.chars().count() as i64)),
        Value::List(rc) => Ok(Value::Int(rc.borrow().len() as i64)),
        Value::Dict(rc) => Ok(Value::Int(rc.borrow().len() as i64)),
        other => Err(eval_error(format!(
            "len() not supported for {}",
            other.type_name()
        ))),
    }
}

fn builtin_range(args: &[Value]) -> Result<Value> {
    let to_int = |v: &Value, what: &str| match v {
        Value::Int(n) => Ok(*n),
        other => Err(eval_error(format!(
            "range() {what} must be an integer, got {}",
            other.type_name()
        ))),
    };
    match args.len() {
        1 => {
            let end = to_int(&args[0], "end")?;
            Ok(Value::list((0..end).map(Value::Int).collect()))
        }
        2 => {
            let start = to_int(&args[0], "start")?;
            let end = to_int(&args[1], "end")?;
            Ok(Value::list((start..end).map(Value::Int).collect()))
        }
        3 => {
            let start = to_int(&args[0], "start")?;
            let end = to_int(&args[1], "end")?;
            let step = to_int(&args[2], "step")?;
            if step == 0 {
                return Err(eval_error("range() step cannot be zero"));
            }
            let mut result = Vec::new();
            let mut i = start;
            if step > 0 {
                while i < end {
                    result.push(Value::Int(i));
                    i = i
                        .checked_add(step)
                        .ok_or_else(|| eval_error("range() step caused integer overflow"))?;
                }
            } else {
                while i > end {
                    result.push(Value::Int(i));
                    i = i
                        .checked_add(step)
                        .ok_or_else(|| eval_error("range() step caused integer overflow"))?;
                }
            }
            Ok(Value::list(result))
        }
        n => Err(eval_error(format!(
            "range() expects 1-3 arguments, got {n}"
        ))),
    }
}

fn builtin_print(args: &[Value]) -> Result<Value> {
    let parts: Vec<String> = args
        .iter()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .collect();
    println!("{}", parts.join(" "));
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parser::parse;
    use crate::core::test_utils::{assert_eval, run, try_run, values_equal};

    #[test]
    fn test_arithmetic() {
        assert_eval("1 + 2", &[], Value::from(3i64));
        assert_eval("10 - 3", &[], Value::from(7i64));
        assert_eval("2 * 5", &[], Value::from(10i64));
        assert_eval("10 / 4", &[], Value::from(2.5f64));
        assert_eval("10 // 4", &[], Value::from(2i64));
        assert_eval("-7 // 2", &[], Value::from(-4i64));
        assert_eval("7 // -2", &[], Value::from(-4i64));
        assert_eval("7.0 // 2.0", &[], Value::from(3.0f64));
        assert_eval(
            r#""hello" + "_" + "world""#,
            &[],
            Value::from("hello_world"),
        );
        let a = Value::from(vec![1i64, 2i64]);
        let b = Value::from(vec![3i64, 4i64]);
        assert_eval(
            "a + b",
            &[("a", a), ("b", b)],
            Value::from(vec![1i64, 2i64, 3i64, 4i64]),
        );
        assert!(try_run("1 // 0", &[]).is_err());
    }

    #[test]
    fn test_pow() {
        assert_eval("2 ** 10", &[], Value::from(1024i64));
        assert_eval("2 ** 0", &[], Value::from(1i64));
        assert_eval("2 ** -1", &[], Value::from(0.5f64));
        assert_eval("2.0 ** 3", &[], Value::from(8.0f64));
        // right-associative: 2**3**2 = 2**(3**2) = 512
        assert_eval("2 ** 3 ** 2", &[], Value::from(512i64));
        // unary minus: -2**2 = -(2**2) = -4
        assert_eval("-2 ** 2", &[], Value::from(-4i64));
        assert!(try_run("2 ** 9999999999", &[]).is_err());
    }

    #[test]
    fn test_mod() {
        assert_eval("7 % 3", &[], Value::from(1i64));
        assert_eval("-7 % 3", &[], Value::from(2i64));
        assert_eval("7 % -3", &[], Value::from(-2i64));
        assert_eval("-7 % -3", &[], Value::from(-1i64));
        assert_eval("7.5 % 2.5", &[], Value::from(0.0f64));
        assert_eval("-7.0 % 3.0", &[], Value::from(2.0f64));
        assert!(try_run("1 % 0", &[]).is_err());
    }

    #[test]
    fn test_integer_overflow() {
        let max = Value::Int(i64::MAX);
        let min = Value::Int(i64::MIN);

        assert!(try_run("a + b", &[("a", max.clone()), ("b", Value::Int(1))]).is_err());
        assert!(try_run("a - b", &[("a", min.clone()), ("b", Value::Int(1))]).is_err());
        assert!(try_run("a * b", &[("a", max.clone()), ("b", Value::Int(2))]).is_err());
        assert!(try_run("-a", &[("a", min.clone())]).is_err());
        assert!(try_run("a // b", &[("a", min.clone()), ("b", Value::Int(-1))]).is_err());

        assert_eval("a + b", &[("a", max.clone()), ("b", Value::Int(0))], max);
        assert_eval(
            "a / b",
            &[("a", Value::Int(6)), ("b", Value::Int(-1))],
            Value::Float(-6.0),
        );
        assert_eval("-a", &[("a", Value::Int(5))], Value::Int(-5));

        assert_eval(
            "a + b",
            &[("a", Value::Int(i64::MAX)), ("b", Value::Float(1.0))],
            Value::Float(i64::MAX as f64 + 1.0),
        );
    }

    #[test]
    fn test_comparison() {
        assert_eval("1 == 1", &[], Value::from(true));
        assert_eval("1 != 2", &[], Value::from(true));
        assert_eval("2 > 1", &[], Value::from(true));
        assert_eval("1 >= 1", &[], Value::from(true));
        assert_eval(r#""a" < "b""#, &[], Value::from(true));
        assert_eval(r#""b" > "a""#, &[], Value::from(true));
        assert_eval(r#""abc" > "ab""#, &[], Value::from(true));
        assert_eval(r#""abc" == "abc""#, &[], Value::from(true));
        assert_eval(r#""abc" <= "abc""#, &[], Value::from(true));
    }

    #[test]
    // Arrays compare lexicographically: first differing element decides; shorter prefix is less.
    fn test_array_comparison() {
        assert_eval("[1, 2] < [1, 3]", &[], Value::from(true));
        assert_eval("[1, 2] < [1, 2, 3]", &[], Value::from(true));
        assert_eval("[] < [1]", &[], Value::from(true));
    }

    #[test]
    // Unlike Python, ordering arrays of non-comparable elements errors rather than short-circuiting through equality.
    fn test_array_comparison_non_orderable_errors() {
        assert!(try_run(r#"[{"foo": "bar"}] <= [{"foo": "bar"}]"#, &[]).is_err());
    }

    #[test]
    fn test_logical() {
        assert_eval("true and false", &[], Value::from(false));
        assert_eval("true or false", &[], Value::from(true));
        assert_eval("not true", &[], Value::from(false));
    }

    #[test]
    fn test_logical_return_value() {
        // `and` returns the left operand when falsy, else the right operand
        assert_eval("0 and 2", &[], Value::from(0i64));
        assert_eval("1 and 2", &[], Value::from(2i64));
        // `or` returns the left operand when truthy, else the right operand
        assert_eval("1 or 2", &[], Value::from(1i64));
        assert_eval("0 or 2", &[], Value::from(2i64));
        // short-circuit: RHS must not be evaluated when result is already determined
        assert_eval("false and missing_var", &[], Value::from(false));
        assert_eval("true or missing_var", &[], Value::from(true));
    }

    #[test]
    fn test_index() {
        let m = Value::dict(indexmap::indexmap! {
            "name".into() => Value::from("alice"),
        });
        assert_eval(r#"m["name"]"#, &[("m", m.clone())], Value::from("alice"));
        assert!(try_run(r#"m["missing"]"#, &[("m", m)]).is_err());
        let arr = Value::from(vec![1i64, 2i64, 3i64]);
        assert_eval("arr[0]", &[("arr", arr.clone())], Value::from(1i64));
        assert_eval("arr[-1]", &[("arr", arr.clone())], Value::from(3i64));
        assert!(try_run("arr[10]", &[("arr", arr)]).is_err());
        assert_eval(r#""hello"[0]"#, &[], Value::from("h"));
        assert_eval(r#""hello"[-1]"#, &[], Value::from("o"));
        assert!(try_run(r#""hello"[99]"#, &[]).is_err());
    }

    #[test]
    fn test_slice() {
        assert_eval(r#""abcde"[1:3]"#, &[], Value::from("bc"));
        assert_eval(r#""abcde"[:3]"#, &[], Value::from("abc"));
        assert_eval(r#""abcde"[2:]"#, &[], Value::from("cde"));
        assert_eval(r#""abcde"[:]"#, &[], Value::from("abcde"));
        assert_eval(r#""abcde"[::-1]"#, &[], Value::from("edcba"));
        assert_eval(r#""abcde"[-1::-2]"#, &[], Value::from("eca"));
        assert_eval(r#""abcde"[::2]"#, &[], Value::from("ace"));
        let arr = Value::list((0i64..5).map(Value::from).collect());
        assert_eval(
            "arr[1:3]",
            &[("arr", arr.clone())],
            Value::from(vec![1i64, 2i64]),
        );
        assert_eval(
            "arr[-2:]",
            &[("arr", arr.clone())],
            Value::from(vec![3i64, 4i64]),
        );
        assert_eval(
            "arr[::-1]",
            &[("arr", arr.clone())],
            Value::list((0i64..5).rev().map(Value::from).collect()),
        );
        assert!(try_run("arr[s:]", &[("arr", arr.clone()), ("s", Value::Float(1.0))]).is_err());
        assert!(try_run("arr[:s]", &[("arr", arr.clone()), ("s", Value::Float(3.0))]).is_err());
        assert!(try_run("arr[::s]", &[("arr", arr), ("s", Value::Float(2.0))]).is_err());
        assert!(try_run(r#""abc"[::0]"#, &[]).is_err());
    }

    #[test]
    fn test_in_operator() {
        let pkgs = Value::from(vec!["foo", "bar"]);
        assert_eval(
            r#""foo" in pkgs"#,
            &[("pkgs", pkgs.clone())],
            Value::from(true),
        );
        assert_eval(
            r#""baz" in pkgs"#,
            &[("pkgs", pkgs.clone())],
            Value::from(false),
        );
        assert_eval(
            r#""foo" not in pkgs"#,
            &[("pkgs", pkgs.clone())],
            Value::from(false),
        );
        assert_eval(r#""baz" not in pkgs"#, &[("pkgs", pkgs)], Value::from(true));
        assert_eval(r#""world" in "hello world""#, &[], Value::from(true));
        assert_eval(r#""xyz" in "hello world""#, &[], Value::from(false));
        assert_eval(r#""xyz" not in "hello world""#, &[], Value::from(true));
        assert_eval(r#""" in "hello""#, &[], Value::from(true));
        let m = Value::dict(indexmap::indexmap! {
            "a".into() => Value::from(1i64),
            "b".into() => Value::from(2i64),
        });
        assert_eval(r#""a" in m"#, &[("m", m.clone())], Value::from(true));
        assert_eval(r#""c" in m"#, &[("m", m.clone())], Value::from(false));
        assert_eval(r#""a" not in m"#, &[("m", m)], Value::from(false));
        assert!(try_run(r#""x" in null"#, &[]).is_err());
        assert!(try_run(r#""x" not in null"#, &[]).is_err());
        let pkgs2 = Value::from(vec!["a"]);
        assert_eval(r#"not "a" in pkgs"#, &[("pkgs", pkgs2)], Value::from(false));
    }

    #[test]
    fn test_assign() {
        assert_eval("x = 42; x", &[], Value::from(42i64));
        assert_eval("x = 3; x * x", &[], Value::from(9i64));
        assert_eval("x = 2; y = x + 1; x * y", &[], Value::from(6i64));
        assert_eval("x = 1; x = 99; x", &[], Value::from(99i64));
        assert_eval("x = 7; x", &[("x", Value::from(999i64))], Value::from(7i64));
        assert_eval("(x = 10) * 2", &[], Value::from(20i64));
    }

    #[test]
    fn test_unpack() {
        assert_eval("[a, b] = [1, 2]; a + b", &[], Value::from(3i64));
        assert_eval(r#"[a, b] = "xy"; a"#, &[], Value::from("x"));
        assert_eval(
            "[a, [b, c]] = [1, [2, 3]]; a + b + c",
            &[],
            Value::from(6i64),
        );
        assert!(try_run("[a, b] = [1, 2, 3]", &[]).is_err());
        assert!(try_run("[a, b, c] = [1, 2]", &[]).is_err());
        assert!(try_run("[a, b] = 42", &[]).is_err());
        // RHS fully evaluated before any binding: swap works correctly
        assert_eval(
            "a = 1; b = 2; [a, b] = [b, a]; [a, b]",
            &[],
            Value::from(vec![2i64, 1i64]),
        );
    }

    #[test]
    fn test_block() {
        assert_eval("1; 2; 3", &[], Value::from(3i64));
        assert_eval("42;", &[], Value::Null);
        assert_eval("x = 5; x * 2", &[], Value::from(10i64));
        assert_eval("a = 3; b = 4; a * a + b * b", &[], Value::from(25i64));
        assert_eval("x = 1; y = 2; x + y", &[], Value::from(3i64));
    }

    #[test]
    fn test_if() {
        assert_eval("if true { 1 } else { 2 }", &[], Value::from(1i64));
        assert_eval("if false { 1 } else { 2 }", &[], Value::from(2i64));
        assert_eval("if 1 == 1 { 42 } else { 0 }", &[], Value::from(42i64));
        assert_eval("if 1 == 2 { 42 } else { 0 }", &[], Value::from(0i64));
        assert_eval("if null { 1 } else { 2 }", &[], Value::from(2i64));
        assert_eval(
            "if false { 1 } else if false { 2 } else { 3 }",
            &[],
            Value::from(3i64),
        );
        assert_eval(
            "if false { 1 } else if true { 2 } else { 3 }",
            &[],
            Value::from(2i64),
        );
        assert_eval(
            "(if true { 10 } else { 0 }) + (if false { 0 } else { 5 })",
            &[],
            Value::from(15i64),
        );
        assert_eval(
            "if true { x = 7; x * 2 } else { 0 }",
            &[],
            Value::from(14i64),
        );
        assert_eval("if true { 42 }", &[], Value::from(42i64));
        assert_eval("if false { 42 }", &[], Value::Null);
    }

    #[test]
    fn test_dict() {
        assert_eval(
            r#"dict([["a", 1], ["b", 2]])"#,
            &[],
            Value::dict(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            }),
        );
        assert_eval(
            r#"{"a": 1, "b": 2}"#,
            &[],
            Value::dict(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            }),
        );
        assert_eval(
            r#"{"x": true,}"#,
            &[],
            Value::dict(indexmap::indexmap! { "x".into() => Value::Bool(true) }),
        );
        assert_eval("{}", &[], Value::dict(indexmap::indexmap! {}));
        assert_eval(r#"{"pre" + "fix": 9}["prefix"]"#, &[], Value::from(9i64));
        assert_eval(
            r#"{"a": {"b": 2}}"#,
            &[],
            Value::dict(indexmap::indexmap! {
                "a".into() => Value::dict(indexmap::indexmap! { "b".into() => Value::from(2i64) }),
            }),
        );
        // insertion order must not affect equality
        assert_eval(
            r#"{"a": 1, "b": 2} == {"b": 2, "a": 1}"#,
            &[],
            Value::Bool(true),
        );
        // dict() with no args produces empty dict
        assert_eval("dict()", &[], Value::dict(indexmap::indexmap! {}));
        // dict(d) produces a shallow copy
        assert_eval(
            r#"let b = dict(a); b["x"] = 9; a"#,
            &[(
                "a",
                Value::dict(indexmap::indexmap! { "x".into() => Value::from(1i64) }),
            )],
            Value::dict(indexmap::indexmap! { "x".into() => Value::from(1i64) }),
        );
    }

    #[test]
    fn test_constructor() {
        assert_eval(r#"str("hello")"#, &[], Value::from("hello"));
        assert_eval(r#"str(42)"#, &[], Value::from("42"));
        assert_eval(r#"str(true)"#, &[], Value::from("true"));
        assert_eval(r#"str(false)"#, &[], Value::from("false"));
        assert_eval(r#"str(null)"#, &[], Value::from("null"));
        assert_eval(r#"str([1, 2, 3])"#, &[], Value::from("[1, 2, 3]"));
        assert_eval(r#"str([])"#, &[], Value::from("[]"));
        assert_eval(r#"str({"a": 1})"#, &[], Value::from(r#"{"a": 1}"#));
        assert_eval(r#"str(0.0)"#, &[], Value::from("0.0"));
        assert_eval(r#"str(1.0)"#, &[], Value::from("1.0"));
        assert_eval(r#"str(0.0001)"#, &[], Value::from("0.0001"));
        assert_eval(r#"str(1e-5)"#, &[], Value::from("1e-5"));
        assert_eval(r#"str(1e-30)"#, &[], Value::from("1e-30"));
        assert_eval(r#"str(1.5e-10)"#, &[], Value::from("1.5e-10"));
        assert_eval(r#"str(1e16)"#, &[], Value::from("1e16"));
        assert_eval(r#"str(1.5e20)"#, &[], Value::from("1.5e20"));
        assert_eval(r#"int(42)"#, &[], Value::from(42i64));
        assert_eval(r#"int(3.9)"#, &[], Value::from(3i64));
        assert_eval(r#"int(-3.9)"#, &[], Value::from(-3i64));
        assert_eval(r#"int("42")"#, &[], Value::from(42i64));
        assert_eval(r#"int(true)"#, &[], Value::from(1i64));
        assert_eval(r#"int(false)"#, &[], Value::from(0i64));
        assert!(try_run("int(f)", &[("f", Value::Float(f64::NAN))]).is_err());
        assert!(try_run("int(f)", &[("f", Value::Float(f64::INFINITY))]).is_err());
        assert!(try_run("int(f)", &[("f", Value::Float(f64::NEG_INFINITY))]).is_err());
        assert!(try_run("int(f)", &[("f", Value::Float(1e100))]).is_err());
        assert_eval(
            "int(f)",
            &[("f", Value::Float(i64::MIN as f64))],
            Value::from(i64::MIN),
        );
        assert_eval(r#"float(42)"#, &[], Value::from(42.0f64));
        assert_eval(r#"float(1.5)"#, &[], Value::from(1.5f64));
        assert_eval(r#"float("1.5")"#, &[], Value::from(1.5f64));
        assert_eval(r#"float(true)"#, &[], Value::from(1.0f64));
        assert_eval(r#"float(false)"#, &[], Value::from(0.0f64));
        assert_eval(r#"bool(1)"#, &[], Value::from(true));
        assert_eval(r#"bool(0)"#, &[], Value::from(false));
        assert_eval(r#"bool("")"#, &[], Value::from(false));
        assert_eval(r#"bool("x")"#, &[], Value::from(true));
        assert_eval(r#"bool(null)"#, &[], Value::from(false));
        assert_eval(r#"list("abc")"#, &[], Value::from(vec!["a", "b", "c"]));
        let arr = Value::from(vec![1i64, 2i64]);
        assert_eval("list(arr)", &[("arr", arr.clone())], arr);
        let m = Value::dict(
            indexmap::indexmap! { "x".into() => Value::from(1i64), "y".into() => Value::from(2i64) },
        );
        assert_eval("list(m)", &[("m", m)], Value::from(vec!["x", "y"]));
        assert_eval("bool()", &[], Value::from(false));
        assert_eval("int()", &[], Value::from(0i64));
        assert_eval("float()", &[], Value::from(0.0f64));
        assert_eval("str()", &[], Value::from(""));
        assert_eval("list()", &[], Value::list(vec![]));
        assert_eval("bool(0.0)", &[], Value::from(false));
        assert_eval("bool(1.0)", &[], Value::from(true));
        assert_eval("bool([])", &[], Value::from(false));
        assert_eval("bool([0])", &[], Value::from(true));
        assert_eval("bool({})", &[], Value::from(false));
        assert_eval(r#"bool({"a": 1})"#, &[], Value::from(true));
    }

    #[test]
    fn test_string_add_non_string_errors() {
        assert!(try_run(r#""1" + 1"#, &[]).is_err());
    }

    #[test]
    fn test_nan_comparisons() {
        let nan = Value::Float(f64::NAN);
        for expr in &[
            "nan < 1.0",
            "nan > 1.0",
            "nan <= 1.0",
            "nan >= 1.0",
            "1.0 < nan",
        ] {
            assert_eval(expr, &[("nan", nan.clone())], Value::Bool(false));
        }
        assert_eval("nan == nan", &[("nan", nan.clone())], Value::Bool(false));
        assert_eval("nan != nan", &[("nan", nan)], Value::Bool(true));
    }

    #[test]
    fn test_object_operator_overload() {
        #[derive(Debug, Clone)]
        struct Counter(i64);

        impl super::super::value::ImmutableObject for Counter {
            fn type_object(&self) -> Rc<super::super::value::TypeValue> {
                thread_local! {
                    static TY: Rc<super::super::value::TypeValue> =
                        Rc::new(super::super::value::TypeValue::new("Counter", None));
                }
                TY.with(Rc::clone)
            }
            fn call_method(&self, method: &str, args: &[Value]) -> Result<Value> {
                match method {
                    "__add__" => match args.first() {
                        Some(Value::Int(n)) => Ok(Value::object(Counter(self.0 + n))),
                        _ => Err(eval_error("expected int")),
                    },
                    "__eq__" => match args.first() {
                        Some(Value::Object(other)) => {
                            let other = other.display().parse::<i64>().unwrap_or(i64::MIN);
                            Ok(Value::Bool(self.0 == other))
                        }
                        _ => Ok(Value::Bool(false)),
                    },
                    m => Err(eval_error(format!("no method {m}"))),
                }
            }
            fn display(&self) -> String {
                self.0.to_string()
            }
        }

        let env = default_env();
        env_bind(&env, "c", Value::object(Counter(10)));
        let result = eval_inner(&parse("c + 5").unwrap(), &env).unwrap();
        assert!(matches!(result, Value::Object(_)));
        assert_eq!(result.to_string(), "15");
        env_bind(&env, "d", Value::object(Counter(10)));
        {
            let result = eval_inner(&parse("c == d").unwrap(), &env).unwrap();
            let expected = Value::Bool(true);
            assert!(
                values_equal(&result, &expected).expect("values_equal failed"),
                "\nleft:  {:?}\nright: {:?}",
                result,
                expected
            );
        }
    }

    #[test]
    fn test_object_getitem_and_iter() {
        #[derive(Debug, Clone)]
        struct Bag(Vec<(String, i64)>);

        impl super::super::value::ImmutableObject for Bag {
            fn type_object(&self) -> Rc<super::super::value::TypeValue> {
                thread_local! {
                    static TY: Rc<super::super::value::TypeValue> =
                        Rc::new(super::super::value::TypeValue::new("Bag", None));
                }
                TY.with(Rc::clone)
            }
            fn call_method(&self, method: &str, args: &[Value]) -> Result<Value> {
                match method {
                    "__getitem__" => match args.first() {
                        Some(Value::String(k)) => self
                            .0
                            .iter()
                            .find(|(key, _)| key == k)
                            .map(|(_, v)| Value::Int(*v))
                            .ok_or_else(|| eval_error(format!("key '{k}' not found"))),
                        _ => Err(eval_error("__getitem__ expects a string")),
                    },
                    "__iter__" => Ok(Value::list(
                        self.0
                            .iter()
                            .map(|(k, _)| Value::String(k.clone()))
                            .collect(),
                    )),
                    m => Err(eval_error(format!("no method {m}"))),
                }
            }
        }

        let env = default_env();
        env_bind(
            &env,
            "bag",
            Value::object(Bag(vec![("x".into(), 10), ("y".into(), 20)])),
        );

        // __getitem__
        assert_eval(
            r#"bag["x"]"#,
            &[("bag", env.borrow().bindings["bag"].clone())],
            Value::from(10i64),
        );
        assert_eval(
            r#"bag["y"]"#,
            &[("bag", env.borrow().bindings["bag"].clone())],
            Value::from(20i64),
        );
        assert!(try_run(
            r#"bag["z"]"#,
            &[("bag", env.borrow().bindings["bag"].clone())]
        )
        .is_err());

        // __iter__ via for-in
        assert_eval(
            "keys = []; for k in bag { keys = keys + [k] } keys",
            &[("bag", env.borrow().bindings["bag"].clone())],
            Value::from(vec!["x", "y"]),
        );
    }

    #[test]
    fn test_var() {
        assert!(try_run("missing", &[]).is_err());
    }

    #[test]
    fn test_native_func() {
        let env = default_env();
        env_bind(
            &env,
            "join_path",
            Value::Fn(NativeFn::new(|args: &[Value]| {
                let parts: Vec<&str> = args
                    .iter()
                    .map(|v| {
                        if let Value::String(s) = v {
                            s.as_str()
                        } else {
                            ""
                        }
                    })
                    .collect();
                Ok(Value::String(parts.join("/")))
            })),
        );
        {
            let result = eval(&parse(r#"join_path("base", "file.json")"#).unwrap(), &env).unwrap();
            let expected = Value::from("base/file.json");
            assert!(
                values_equal(&result, &expected).expect("values_equal failed"),
                "\nleft:  {:?}\nright: {:?}",
                result,
                expected
            );
        }
    }

    #[test]
    fn test_len_builtin() {
        assert_eval(r#"len("hello")"#, &[], Value::from(5i64));
        assert_eval(r#"len("")"#, &[], Value::from(0i64));
        let arr = Value::from(vec![1i64, 2i64, 3i64]);
        assert_eval("len(arr)", &[("arr", arr)], Value::from(3i64));
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("len(m)", &[("m", m)], Value::from(2i64));
        assert!(try_run("len(42)", &[]).is_err());
    }

    #[test]
    fn test_type_builtin() {
        assert_eval("type(42) == int", &[], Value::Bool(true));
        assert_eval("type(3.14) == float", &[], Value::Bool(true));
        assert_eval(r#"type("hello") == str"#, &[], Value::Bool(true));
        assert_eval("type([1, 2]) == list", &[], Value::Bool(true));
        assert_eval(r#"type({"a": 1}) == dict"#, &[], Value::Bool(true));
        assert_eval("type(true) == bool", &[], Value::Bool(true));
        assert_eval("type(42) != float", &[], Value::Bool(true));
        assert_eval("int == int", &[], Value::Bool(true));
        assert_eval("int == str", &[], Value::Bool(false));
        assert_eval("type(type) == type", &[], Value::Bool(true));
        assert_eval("type(int) == type", &[], Value::Bool(true));
        assert_eval("type(42) == type", &[], Value::Bool(false));
    }

    #[test]
    fn test_complex_expr() {
        let obj = Value::dict(indexmap::indexmap! {
            "type".into() => Value::from("json"),
            "name".into() => Value::from("foo"),
        });
        let names = Value::from(vec!["foo", "bar"]);
        assert_eval(
            r#"obj["type"] == "json" and obj["name"] in names"#,
            &[("obj", obj), ("names", names)],
            Value::from(true),
        );
    }

    #[test]
    fn test_depth_limit_ok() {
        let n = AST_DEPTH.with(|c| c.limit) - 1;
        let expr = format!("1{}", "+1".repeat(n));
        {
            let result = eval(&parse(&expr).unwrap(), &default_env()).unwrap();
            let expected = Value::Int(n as i64 + 1);
            assert!(
                values_equal(&result, &expected).expect("values_equal failed"),
                "\nleft:  {:?}\nright: {:?}",
                result,
                expected
            );
        }
    }

    #[test]
    // This tests if stack overflow is captured as error instead of crashing the engine
    fn test_deep_list_eq_depth_limit() {
        let mut a = Value::list(vec![Value::Int(1)]);
        let mut b = Value::list(vec![Value::Int(1)]);
        for _ in 0..CALL_DEPTH.with(|c| c.limit) {
            a = Value::list(vec![a]);
            b = Value::list(vec![b]);
        }
        assert!(try_run("a == b", &[("a", a), ("b", b)]).is_err());
    }

    #[test]
    fn test_eval_error_pos() {
        let err = try_run("1 / 0", &[]).unwrap_err();
        assert!(matches!(err, Error::Eval { pos: 0, .. }));
    }

    #[test]
    fn test_list_index_assign() {
        assert_eval(
            "a = [1, 2, 3]; a[1] = 99; a",
            &[],
            Value::from(vec![1i64, 99i64, 3i64]),
        );
        assert_eval(
            "a = [10, 20, 30]; a[-1] = 99; a",
            &[],
            Value::from(vec![10i64, 20i64, 99i64]),
        );
        assert_eval("a = [0]; a[0] = 7", &[], Value::from(7i64));
        assert_eval(
            "a = [[1, 2], [3, 4]]; a[0][1] = 99; a[0]",
            &[],
            Value::from(vec![1i64, 99i64]),
        );
    }

    #[test]
    fn test_assign_rhs_evaluated_before_index() {
        assert_eval(
            "a = [0, 0, 0]; i = -999; a[i] = (i = 1); a",
            &[],
            Value::from(vec![0i64, 1i64, 0i64]),
        );
    }

    #[test]
    fn test_list_index_assign_out_of_range() {
        assert!(try_run("a = [1, 2]; a[5] = 9", &[]).is_err());
    }

    #[test]
    fn test_dict_index_assign() {
        assert_eval(
            r#"m = {"a": 1}; m["b"] = 2; m["b"]"#,
            &[],
            Value::from(2i64),
        );
        assert_eval(
            r#"m = {"x": 10}; m["x"] = 99; m["x"]"#,
            &[],
            Value::from(99i64),
        );
        assert_eval(
            r#"m = {"k": [1, 2, 3]}; m["k"][0] = 99; m["k"][0]"#,
            &[],
            Value::from(99i64),
        );
        assert_eval(r#"m = {"a": 0}; m["x"] = 42"#, &[], Value::from(42i64));
    }

    #[test]
    fn test_reference_semantics() {
        assert_eval(
            "a = [1, 2, 3]; b = a; b[0] = 99; a[0]",
            &[],
            Value::from(99i64),
        );
        assert_eval(
            r#"m = {"x": 1}; n = m; n["x"] = 42; m["x"]"#,
            &[],
            Value::from(42i64),
        );

        let arr = Value::from(vec![1i64, 2i64, 3i64]);
        run("arr[0] = 99", &[("arr", arr.clone())]);
        {
            let expected = Value::from(vec![99i64, 2i64, 3i64]);
            assert!(
                values_equal(&arr, &expected).expect("values_equal failed"),
                "\nleft:  {:?}\nright: {:?}",
                arr,
                expected
            );
        }
    }

    #[test]
    fn test_while() {
        assert_eval("i = 0; while i < 5 { i = i + 1 } i", &[], Value::from(5i64));
        assert_eval("while false { 1 }", &[], Value::Null);
        assert_eval(
            "s = 0; i = 1; while i <= 10 { s = s + i; i = i + 1 } s",
            &[],
            Value::from(55i64),
        );
    }

    #[test]
    fn test_invalid_lvalue() {
        assert!(try_run("1 = 2", &[]).is_err());
    }

    #[test]
    fn test_for_in_list() {
        assert_eval(
            "s = 0; for x in [1, 2, 3] { s = s + x } s",
            &[],
            Value::from(6i64),
        );
        assert_eval("for x in [] { x } 42", &[], Value::from(42i64));
        // loop variable persists after loop (Python semantics)
        assert_eval("for x in [10, 20] { x } x", &[], Value::from(20i64));
    }

    #[test]
    fn test_for_in_map_keys() {
        let m = Value::dict(indexmap::indexmap! {
            "a".into() => Value::from(1i64),
            "b".into() => Value::from(2i64),
        });
        assert_eval(
            "keys = []; for k in m { keys = keys + [k] } keys",
            &[("m", m)],
            Value::from(vec!["a", "b"]),
        );
    }

    #[test]
    fn test_for_in_string() {
        assert_eval(
            r#"n = 0; for c in "abc" { n = n + 1 } n"#,
            &[],
            Value::from(3i64),
        );
    }

    #[test]
    fn test_for_in_unpack() {
        assert_eval(
            "s = 0; for [a, b] in [[1, 2], [3, 4]] { s = s + a + b } s",
            &[],
            Value::from(10i64),
        );
        assert!(try_run("for [a, b] in [[1, 2, 3]] { a }", &[]).is_err());
    }

    #[test]
    fn test_for_in_null() {
        assert!(try_run("for x in null { x }", &[]).is_err());
    }

    #[test]
    fn test_return() {
        // bare return evaluates to null
        assert_eval("return", &[], Value::Null);
        // return with value
        assert_eval("return 42", &[], Value::from(42i64));
        // early exit from block: statements after return are not evaluated
        assert_eval("return 1; 2", &[], Value::from(1i64));
        // return inside if
        assert_eval("if true { return 7 } 99", &[], Value::from(7i64));
        assert_eval("if false { return 7 } 99", &[], Value::from(99i64));
        // return inside while exits the whole script
        assert_eval(
            "i = 0; while true { i = i + 1; if i == 3 { return i } }",
            &[],
            Value::from(3i64),
        );
        // return with assign expression
        assert_eval(
            "return x = 5",
            &[("x", Value::from(0i64))],
            Value::from(5i64),
        );
    }

    #[test]
    fn test_bitwise() {
        assert_eval("0b1010 & 0b1100", &[], Value::from(0b1000i64));
        assert_eval("0b1010 | 0b1100", &[], Value::from(0b1110i64));
        assert_eval("0b1010 ^ 0b1100", &[], Value::from(0b0110i64));
        assert_eval("1 << 3", &[], Value::from(8i64));
        assert_eval("16 >> 2", &[], Value::from(4i64));
        assert_eval("1 << 62", &[], Value::from(1i64 << 62));
        // chaining and precedence
        assert_eval("1 | 2 & 3", &[], Value::from(3i64)); // & binds tighter: 1 | (2 & 3) = 1 | 2 = 3
        assert_eval("5 ^ 3 & 6", &[], Value::from(7i64)); // 5 ^ (3 & 6) = 5 ^ 2 = 7
                                                          // compound assignment
        assert_eval("x = 0b1111; x &= 0b1010; x", &[], Value::from(0b1010i64));
        assert_eval("x = 0b1010; x |= 0b0101; x", &[], Value::from(0b1111i64));
        assert_eval("x = 0b1111; x ^= 0b1010; x", &[], Value::from(0b0101i64));
        assert_eval("x = 1; x <<= 4; x", &[], Value::from(16i64));
        assert_eval("x = 32; x >>= 2; x", &[], Value::from(8i64));
        // errors: negative operand
        assert!(try_run("-1 & 1", &[]).is_err());
        assert!(try_run("1 | -1", &[]).is_err());
        // errors: non-integer operand
        assert!(try_run("1.0 & 1", &[]).is_err());
        // errors: shift out of range
        assert!(try_run("1 << 63", &[]).is_err());
        assert_eval("1 >> 63", &[], Value::from(0i64));
        assert!(try_run("1 << -1", &[]).is_err());
        // errors: left shift overflow
        assert!(try_run("4611686018427387904 << 1", &[]).is_err()); // 2^62 << 1 overflows
    }

    #[test]
    fn test_bit_length() {
        assert_eval("(0).bit_length()", &[], Value::from(0i64));
        assert_eval("(1).bit_length()", &[], Value::from(1i64));
        assert_eval("(0b1010).bit_length()", &[], Value::from(4i64));
        assert!(try_run("(-1).bit_length()", &[]).is_err());
        assert!(try_run("(1).bit_length(99)", &[]).is_err());
    }

    #[test]
    fn test_for_in_map_items() {
        let m = Value::dict(indexmap::indexmap! {
            "a".into() => Value::from(10i64),
            "b".into() => Value::from(20i64),
        });
        assert_eval(
            "s = 0; for pair in m.items() { s = s + pair[1] } s",
            &[("m", m)],
            Value::from(30i64),
        );
    }

    #[test]
    fn test_closures() {
        assert_eval("f = fn(x) { x * 2 }; f(5)", &[], Value::from(10i64));
        assert_eval("fn(x) { x + 1 }(3)", &[], Value::from(4i64));
        assert_eval("n = 10; f = fn(x) { x + n }; f(5)", &[], Value::from(15i64));
        assert_eval(
            "x = 1; f = fn() { x = 99 }; f(); x",
            &[],
            Value::from(99i64),
        );
        assert_eval(
            "x = 1; f = fn() { let x = 99; x }; f(); x",
            &[],
            Value::from(1i64),
        );
        assert_eval(
            "fact = fn(n) { if n <= 1 { 1 } else { n * fact(n - 1) } }; fact(5)",
            &[],
            Value::from(120i64),
        );
        assert_eval(
            "f = fn(x) { return x * 2; 999 }; f(4)",
            &[],
            Value::from(8i64),
        );
        assert_eval("let [a, b] = [3, 4]; a + b", &[], Value::from(7i64));
        assert_eval(
            "let [a, [b, c]] = [1, [2, 3]]; a + b + c",
            &[],
            Value::from(6i64),
        );
        assert!(try_run("fn(x, y) { x + y }(1)", &[]).is_err());
        assert!(try_run("fn() { 1 }(42)", &[]).is_err());
    }

    #[test]
    fn test_range() {
        assert_eval("range(5)", &[], Value::from(vec![0i64, 1, 2, 3, 4]));
        assert_eval("range(0)", &[], Value::from(Vec::<Value>::new()));
        assert_eval("range(2, 5)", &[], Value::from(vec![2i64, 3, 4]));
        assert_eval("range(1, 8, 3)", &[], Value::from(vec![1i64, 4, 7]));
        assert_eval("range(5, 0, -2)", &[], Value::from(vec![5i64, 3, 1]));
        assert_eval("range(3, 3)", &[], Value::from(Vec::<Value>::new()));
        assert!(try_run("range(1, 10, 0)", &[]).is_err());
        assert!(try_run(
            "range(start, end, step)",
            &[
                ("start", Value::Int(1)),
                ("end", Value::Int(i64::MAX)),
                ("step", Value::Int(i64::MAX)),
            ]
        )
        .is_err());
        assert!(try_run(
            "range(start, end, step)",
            &[
                ("start", Value::Int(-1)),
                ("end", Value::Int(i64::MIN)),
                ("step", Value::Int(i64::MIN)),
            ]
        )
        .is_err());
    }

    #[test]
    fn test_cyclic_list_increases_live_alloc() {
        let env = default_env();
        let before = crate::core::value::LIVE_ALLOC.with(|c| c.get());
        let expr = parse("let a = []; a.append(a); a").unwrap();
        let v = eval(&expr, &env).unwrap();
        let after = crate::core::value::LIVE_ALLOC.with(|c| c.get());
        // Cyclic list keeps a TrackedRc alive; the guard in `crate::eval::<T>` detects this.
        assert!(after > before, "expected live TrackedRc for cyclic list");
        drop(v);
    }
}
