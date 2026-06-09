use std::cell::Cell;
use std::collections::HashMap;

use indexmap::IndexMap;

use super::ast::{BinOp, Expr, ExprKind, UnaryOp};
use super::builtins::{array as array_methods, map as map_methods, str as str_methods};
use super::builtins::{builtin_math, builtin_regex, builtin_url};
use super::error::{Error, InnerError, InnerResult, Result};
use super::value::{format_float, NativeFn, Value};
use crate::expect_arity;

#[cfg(debug_assertions)]
const MAX_EVAL_DEPTH: usize = 64;
#[cfg(not(debug_assertions))]
const MAX_EVAL_DEPTH: usize = 1024;

thread_local! {
    static EVAL_DEPTH: Cell<usize> = const { Cell::new(0) };
}

struct DepthGuard;

impl DepthGuard {
    fn enter() -> InnerResult<Self> {
        let depth = EVAL_DEPTH.with(|d| {
            let v = d.get() + 1;
            d.set(v);
            v
        });
        if depth > MAX_EVAL_DEPTH {
            EVAL_DEPTH.with(|d| d.set(d.get() - 1));
            Err(InnerError::new(format!(
                "expression exceeds maximum evaluation depth ({MAX_EVAL_DEPTH})"
            )))
        } else {
            Ok(DepthGuard)
        }
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        EVAL_DEPTH.with(|d| d.set(d.get() - 1));
    }
}

trait ToEvalError<T> {
    fn to_eval_error(self, pos: usize) -> Result<T>;
}

impl<T> ToEvalError<T> for InnerResult<T> {
    fn to_eval_error(self, pos: usize) -> Result<T> {
        self.map_err(|e| Error::Eval { pos, msg: e.msg })
    }
}

pub type Env = HashMap<String, Value>;

pub fn default_env() -> Env {
    let mut env = Env::new();
    env.insert("str".into(), Value::Fn(NativeFn::new(builtin_str)));
    env.insert("int".into(), Value::Fn(NativeFn::new(builtin_int)));
    env.insert("float".into(), Value::Fn(NativeFn::new(builtin_float)));
    env.insert("bool".into(), Value::Fn(NativeFn::new(builtin_bool)));
    env.insert("list".into(), Value::Fn(NativeFn::new(builtin_list)));
    env.insert("map".into(), Value::Fn(NativeFn::new(builtin_map)));
    env.insert("Url".into(), Value::Fn(NativeFn::new(builtin_url)));
    env.insert("Regex".into(), Value::Fn(NativeFn::new(builtin_regex)));
    env.insert("math".into(), builtin_math());
    env.insert("print".into(), Value::Fn(NativeFn::new(builtin_print)));
    env.insert("type".into(), Value::Fn(NativeFn::new(builtin_type)));
    env.insert("len".into(), Value::Fn(NativeFn::new(builtin_len)));
    env
}

pub fn eval(expr: &Expr, env: &mut Env) -> Result<Value> {
    match eval_inner(expr, env) {
        Err(Error::Return(v)) => Ok(v),
        other => other,
    }
}

/// Recursion entrypoint for native function/operator/method invocations.
pub(crate) fn call_inner(f: &NativeFn, args: &[Value]) -> InnerResult<Value> {
    let _guard = DepthGuard::enter()?;
    f.call(args)
}

fn call_func(f: &NativeFn, args: &[Value], pos: usize) -> Result<Value> {
    call_inner(f, args).to_eval_error(pos)
}

pub(crate) fn eval_eq(a: Value, b: Value) -> InnerResult<bool> {
    match call_inner(&NativeFn::new(eq_op), &[a, b])? {
        Value::Bool(b) => Ok(b),
        _ => Err(InnerError::new("__eq__ must return a bool")),
    }
}

fn eq_op(args: &[Value]) -> InnerResult<Value> {
    let [a, b] = args else {
        return Err(InnerError::new("== requires two operands"));
    };
    if let Value::Object(rc) = a {
        return rc.call_method("__eq__", std::slice::from_ref(b));
    }
    match (a, b) {
        (Value::Array(a), Value::Array(b)) => array_methods::eq_inner(a, b).map(Value::Bool),
        (Value::Map(a), Value::Map(b)) => map_methods::eq_inner(a, b).map(Value::Bool),
        _ => Ok(Value::Bool(primitive_eq(a, b))),
    }
}

fn resolve_attr(recv: Value, attr: &str) -> InnerResult<Value> {
    match recv {
        Value::Module(m) => m
            .get(attr)
            .cloned()
            .ok_or_else(|| InnerError::new(format!("module has no attribute '{attr}'"))),
        Value::Int(n) => match attr {
            "bit_length" => Ok(Value::Fn(NativeFn::new(move |args| {
                expect_arity("int.bit_length", args, 0, 0)?;
                if n < 0 {
                    return Err(InnerError::new(
                        "bit_length() not supported for negative integers",
                    ));
                }
                Ok(Value::Int((i64::BITS - n.leading_zeros()) as i64))
            }))),
            _ => Err(InnerError::new(format!("int has no attribute '{attr}'"))),
        },
        recv @ Value::String(_) => str_methods::resolve_method(recv, attr).map(Value::Fn),
        recv @ Value::Array(_) => array_methods::resolve_method(recv, attr).map(Value::Fn),
        recv @ Value::Map(_) => map_methods::resolve_method(recv, attr).map(Value::Fn),
        Value::Object(rc) => {
            if let Some(result) = rc.get_property(attr) {
                return result;
            }
            let attr = attr.to_string();
            Ok(Value::Fn(NativeFn::new(move |args| {
                rc.call_method(&attr, args)
            })))
        }
        v => Err(InnerError::new(format!(
            "{} has no attribute '{attr}'",
            v.type_name()
        ))),
    }
}

// Returns a NativeFn for a binary operator. args[0]=left, args[1]=right.
fn resolve_op(op: &BinOp) -> NativeFn {
    match op {
        BinOp::Add => NativeFn::new(|args| {
            let (left, right) = binary_args(args)?;
            if let Value::Object(rc) = &left {
                return rc.call_method("__add__", &[right]);
            }
            match (left, right) {
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + b.as_str())),
                (Value::Array(a), Value::Array(b)) => {
                    let mut new_vec = a.borrow().clone();
                    new_vec.extend(b.borrow().iter().cloned());
                    Ok(Value::array(new_vec))
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
                        return Err(InnerError::new("division by zero"));
                    }
                    Ok(Value::Float(a as f64 / b as f64))
                }
                Ok((Numeric::Float(a), Numeric::Float(b))) => {
                    if b == 0.0 {
                        return Err(InnerError::new("division by zero"));
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
                        return Err(InnerError::new("division by zero"));
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
                        return Err(InnerError::new("division by zero"));
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
                        return Err(InnerError::new("modulo by zero"));
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
                        return Err(InnerError::new("modulo by zero"));
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
                        Err(InnerError::new("exponent too large"))
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
            _ => Err(InnerError::new("__eq__ must return a bool")),
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
                return Err(InnerError::new(format!(
                    "left shift amount {b} out of range [0, 62]"
                )));
            }
            let result = a
                .checked_shl(b as u32)
                .filter(|&v| v >= 0)
                .ok_or_else(|| InnerError::new("left shift result overflows 63-bit integer"))?;
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

fn contains_inner(left: Value, right: Value) -> InnerResult<bool> {
    match right {
        Value::Array(arr) => {
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
            l => Err(InnerError::new(format!(
                "'in' not supported between {} and string",
                l.type_name()
            ))),
        },
        Value::Map(map) => match left {
            Value::String(key) => Ok(map.borrow().contains_key(&key)),
            l => Err(InnerError::new(format!(
                "'in' not supported between {} and map",
                l.type_name()
            ))),
        },
        Value::Object(rc) => rc
            .call_method("__contains__", &[left])
            .and_then(|v| match v {
                Value::Bool(b) => Ok(b),
                other => Err(InnerError::new(format!(
                    "__contains__ must return bool, got {}",
                    other.type_name()
                ))),
            }),
        r => Err(InnerError::new(format!(
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
            Ok(Value::Bool(!is_truthy(val)))
        }),
        UnaryOp::Neg => NativeFn::new(|args| {
            let val = unary_arg(args)?;
            match val {
                Value::Int(n) => n
                    .checked_neg()
                    .map(Value::Int)
                    .ok_or_else(|| InnerError::new("integer overflow")),
                Value::Float(f) => Ok(Value::Float(-f)),
                v => Err(InnerError::new(format!("cannot negate {}", v.type_name()))),
            }
        }),
    }
}

fn binary_args(args: &[Value]) -> InnerResult<(Value, Value)> {
    let [a, b] = args else {
        return Err(InnerError::new("binary operator requires two operands"));
    };
    Ok((a.clone(), b.clone()))
}

fn unary_arg(args: &[Value]) -> InnerResult<&Value> {
    args.first()
        .ok_or_else(|| InnerError::new("unary operator requires one operand"))
}

fn bitwise_args(a: &Value, b: &Value) -> InnerResult<(i64, i64)> {
    let to_bits = |v: &Value| match v {
        Value::Int(n) if *n >= 0 => Ok(*n),
        Value::Int(_) => Err(InnerError::new(
            "bitwise operands must be non-negative integers",
        )),
        other => Err(InnerError::new(format!(
            "bitwise operands must be non-negative integers, got {}",
            other.type_name()
        ))),
    };
    Ok((to_bits(a)?, to_bits(b)?))
}

// Recursion entrypoint for AST expression evaluation.
fn eval_inner(expr: &Expr, env: &mut Env) -> Result<Value> {
    let pos = expr.span.start;
    let _depth = DepthGuard::enter().to_eval_error(pos)?;
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
            Ok(Value::array(values))
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
                            msg: format!("map key must be a string, got {}", other.type_name()),
                        })
                    }
                };
                map.insert(key_str, eval_inner(v, env)?);
            }
            Ok(Value::map(map))
        }
        ExprKind::Var(name) => env.get(name.as_str()).cloned().ok_or_else(|| Error::Eval {
            pos,
            msg: format!("unknown variable '{name}'"),
        }),
        ExprKind::Index(target, key) => {
            let target = eval_inner(target, env)?;
            let key = eval_inner(key, env)?;
            eval_index(target, key).to_eval_error(pos)
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
            eval_slice(target, start, stop, step).to_eval_error(pos)
        }
        ExprKind::Unary(op, e) => {
            let val = eval_inner(e, env)?;
            let f = resolve_unary_op(op);
            call_func(&f, &[val], pos)
        }
        ExprKind::Attribute { receiver, attr } => {
            let recv = eval_inner(receiver, env)?;
            resolve_attr(recv, attr).to_eval_error(pos)
        }
        ExprKind::Call { callee, args } => {
            let f = eval_inner(callee, env)?;
            let evaled: Result<Vec<_>> = args.iter().map(|a| eval_inner(a, env)).collect();
            match f {
                Value::Fn(native_fn) => call_func(&native_fn, &evaled?, pos),
                _ => Err(Error::Eval {
                    pos,
                    msg: format!("value of type {} is not callable", f.type_name()),
                }),
            }
        }
        ExprKind::Binary(left, op, right) => {
            match op {
                BinOp::And => {
                    let l = eval_inner(left, env)?;
                    if !is_truthy(&l) {
                        return Ok(l);
                    }
                    return eval_inner(right, env);
                }
                BinOp::Or => {
                    let l = eval_inner(left, env)?;
                    if is_truthy(&l) {
                        return Ok(l);
                    }
                    return eval_inner(right, env);
                }
                _ => {}
            }
            let left = eval_inner(left, env)?;
            let right = eval_inner(right, env)?;
            let f = resolve_op(op);
            call_func(&f, &[left, right], pos)
        }
        ExprKind::Assign { lvalue, value } => {
            let v = eval_inner(value, env)?;
            eval_assign_lvalue(lvalue, v.clone(), env)?;
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
            if is_truthy(&c) {
                eval_inner(then, env)
            } else {
                eval_inner(else_, env)
            }
        }
        // No iteration cap — see docs/design.md#no-while-iteration-limit
        ExprKind::While { cond, body } => {
            loop {
                let c = eval_inner(cond, env)?;
                if !is_truthy(&c) {
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
                eval_assign_lvalue(var, item, env)?;
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
    }
}

fn collect_iterable(value: Value, pos: usize) -> Result<Vec<Value>> {
    match value {
        Value::Array(rc) => Ok(rc.borrow().clone()),
        Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        Value::Map(rc) => Ok(rc
            .borrow()
            .keys()
            .map(|k| Value::String(k.clone()))
            .collect()),
        Value::Object(rc) => match rc.call_method("__iter__", &[]).to_eval_error(pos)? {
            Value::Array(arr) => Ok(arr.borrow().clone()),
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

fn eval_assign_lvalue(lvalue: &Expr, value: Value, env: &mut Env) -> Result<()> {
    let pos = lvalue.span.start;
    match &lvalue.kind {
        ExprKind::Var(name) => {
            env.insert(name.clone(), value);
            Ok(())
        }
        ExprKind::Index(target, key) => {
            let container = eval_inner(target, env)?;
            let key_val = eval_inner(key, env)?;
            match (container, &key_val) {
                (Value::Array(rc), Value::Int(i)) => {
                    let len = rc.borrow().len();
                    let idx = array_methods::resolve_index(*i, len).ok_or_else(|| Error::Eval {
                        pos,
                        msg: format!("array index {} out of range (len {})", i, len),
                    })?;
                    rc.borrow_mut()[idx] = value;
                }
                (Value::Map(rc), Value::String(k)) => {
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
                eval_assign_lvalue(target, item, env)?;
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
    env: &mut Env,
    pos: usize,
) -> Result<Value> {
    let rhs_val = eval_inner(rhs, env)?;
    let f = resolve_op(op);
    match &lvalue.kind {
        ExprKind::Var(name) => {
            let current = env.get(name).cloned().ok_or_else(|| Error::Eval {
                pos,
                msg: format!("undefined variable '{name}'"),
            })?;
            let new_val = call_func(&f, &[current, rhs_val], pos)?;
            env.insert(name.clone(), new_val.clone());
            Ok(new_val)
        }
        ExprKind::Index(target, key) => {
            let container = eval_inner(target, env)?;
            let key_val = eval_inner(key, env)?;
            match (container, &key_val) {
                (Value::Array(rc), Value::Int(i)) => {
                    let len = rc.borrow().len();
                    let idx = array_methods::resolve_index(*i, len).ok_or_else(|| Error::Eval {
                        pos,
                        msg: format!("array index {} out of range (len {})", i, len),
                    })?;
                    let current = rc.borrow()[idx].clone();
                    let new_val = call_func(&f, &[current, rhs_val], pos)?;
                    rc.borrow_mut()[idx] = new_val.clone();
                    Ok(new_val)
                }
                (Value::Map(rc), Value::String(k)) => {
                    let current = rc.borrow().get(k.as_str()).cloned().unwrap_or(Value::Null);
                    let new_val = call_func(&f, &[current, rhs_val], pos)?;
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

fn eval_index(target: Value, key: Value) -> InnerResult<Value> {
    match (target, key) {
        (Value::Map(map), Value::String(k)) => map
            .borrow()
            .get(&k)
            .cloned()
            .ok_or_else(|| InnerError::new(format!("map key '{k}' not found"))),
        (Value::Array(arr), Value::Int(i)) => {
            let arr = arr.borrow();
            let len = arr.len();
            array_methods::resolve_index(i, len)
                .map(|pos| arr[pos].clone())
                .ok_or_else(|| InnerError::new(format!("array index {i} out of range (len {len})")))
        }
        (Value::String(s), Value::Int(i)) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len();
            array_methods::resolve_index(i, len)
                .map(|pos| Value::String(chars[pos].to_string()))
                .ok_or_else(|| {
                    InnerError::new(format!("string index {i} out of range (len {len})"))
                })
        }
        (Value::Object(rc), key) => rc.call_method("__getitem__", &[key]),
        (target, key) => Err(InnerError::new(format!(
            "cannot index {} with {}",
            target.type_name(),
            key.type_name()
        ))),
    }
}

fn as_slice_index(v: Value, what: &str) -> InnerResult<i64> {
    match v {
        Value::Int(n) => Ok(n),
        v => Err(InnerError::new(format!(
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
) -> InnerResult<Value> {
    let step = match step {
        None => 1i64,
        Some(v) => as_slice_index(v, "step")?,
    };
    if step == 0 {
        return Err(InnerError::new("slice step cannot be zero"));
    }
    let start = start.map(|v| as_slice_index(v, "start")).transpose()?;
    let stop = stop.map(|v| as_slice_index(v, "stop")).transpose()?;

    match target {
        Value::Array(arr) => {
            let arr = arr.borrow();
            let indices = slice_indices(arr.len(), start, stop, step);
            Ok(Value::array(
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
        v => Err(InnerError::new(format!("cannot slice {}", v.type_name()))),
    }
}

enum Numeric {
    Int(i64),
    Float(f64),
}

fn coerce_numeric(a: &Value, b: &Value) -> InnerResult<(Numeric, Numeric)> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok((Numeric::Int(*a), Numeric::Int(*b))),
        (Value::Int(a), Value::Float(b)) => Ok((Numeric::Float(*a as f64), Numeric::Float(*b))),
        (Value::Float(a), Value::Int(b)) => Ok((Numeric::Float(*a), Numeric::Float(*b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok((Numeric::Float(*a), Numeric::Float(*b))),
        (a, b) => Err(InnerError::new(format!(
            "cannot apply numeric op to {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn int_overflow() -> InnerError {
    InnerError::new("integer overflow")
}

fn numeric_op(
    left: Value,
    right: Value,
    int_op: impl Fn(i64, i64) -> InnerResult<i64>,
    float_op: impl Fn(f64, f64) -> f64,
) -> InnerResult<Value> {
    match coerce_numeric(&left, &right)? {
        (Numeric::Int(a), Numeric::Int(b)) => Ok(Value::Int(int_op(a, b)?)),
        (Numeric::Float(a), Numeric::Float(b)) => Ok(Value::Float(float_op(a, b))),
        _ => unreachable!(),
    }
}

fn binop_type_error(op: &str, l: &Value, r: &Value) -> InnerError {
    InnerError::new(format!(
        "'{op}' not supported between {} and {}",
        l.type_name(),
        r.type_name()
    ))
}

fn compare_values(
    left: Value,
    right: Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> InnerResult<Value> {
    let ord = match coerce_numeric(&left, &right) {
        Ok((Numeric::Int(a), Numeric::Int(b))) => a.cmp(&b),
        Ok((Numeric::Float(a), Numeric::Float(b))) => match a.partial_cmp(&b) {
            Some(ord) => ord,
            None => return Ok(Value::Bool(false)),
        },
        Ok(_) => unreachable!(),
        Err(_) => match (&left, &right) {
            (Value::String(a), Value::String(b)) => a.as_str().cmp(b.as_str()),
            _ => {
                return Err(InnerError::new(format!(
                    "cannot compare {} and {}",
                    left.type_name(),
                    right.type_name()
                )))
            }
        },
    };
    Ok(Value::Bool(pred(ord)))
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
        _ => false,
    }
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.borrow().is_empty(),
        Value::Map(o) => !o.borrow().is_empty(),
        Value::Fn(_) | Value::Object(_) | Value::Module(_) => true,
    }
}

pub fn bool_cast(v: Value) -> bool {
    is_truthy(&v)
}

pub fn str_cast(v: Value) -> InnerResult<String> {
    match builtin_str(std::slice::from_ref(&v))? {
        Value::String(s) => Ok(s),
        _ => unreachable!(),
    }
}

fn builtin_str(args: &[Value]) -> InnerResult<Value> {
    if args.len() > 1 {
        return Err(InnerError::new(format!(
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

fn builtin_int(args: &[Value]) -> InnerResult<Value> {
    if args.len() > 1 {
        return Err(InnerError::new(format!(
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
                Err(InnerError::new(format!("int() value out of range: {f}")))
            } else {
                Ok(Value::Int(t as i64))
            }
        }
        Some(Value::Bool(b)) => Ok(Value::Int(*b as i64)),
        Some(Value::String(s)) => s
            .trim()
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| InnerError::new(format!("int() cannot parse {s:?}"))),
        Some(v) => Err(InnerError::new(format!(
            "int() not supported for {}",
            v.type_name()
        ))),
    }
}

fn builtin_float(args: &[Value]) -> InnerResult<Value> {
    if args.len() > 1 {
        return Err(InnerError::new(format!(
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
            .map_err(|_| InnerError::new(format!("float() cannot parse {s:?}"))),
        Some(v) => Err(InnerError::new(format!(
            "float() not supported for {}",
            v.type_name()
        ))),
    }
}

fn builtin_bool(args: &[Value]) -> InnerResult<Value> {
    if args.len() > 1 {
        return Err(InnerError::new(format!(
            "bool() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    Ok(Value::Bool(args.first().map(is_truthy).unwrap_or(false)))
}

fn builtin_list(args: &[Value]) -> InnerResult<Value> {
    if args.len() > 1 {
        return Err(InnerError::new(format!(
            "list() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::array(a.borrow().clone())),
        Some(Value::String(s)) => Ok(Value::array(
            s.chars().map(|c| Value::String(c.to_string())).collect(),
        )),
        Some(Value::Map(m)) => Ok(Value::array(
            m.borrow()
                .keys()
                .map(|k| Value::String(k.clone()))
                .collect(),
        )),
        Some(v) => Err(InnerError::new(format!(
            "list() not supported for {}",
            v.type_name()
        ))),
        None => Ok(Value::array(vec![])),
    }
}

fn builtin_map(args: &[Value]) -> InnerResult<Value> {
    if args.len() != 1 {
        return Err(InnerError::new(format!(
            "map() expects 1 argument, got {}",
            args.len()
        )));
    }
    let pairs = match args.first() {
        Some(Value::Array(a)) => a.borrow().clone(),
        _ => {
            return Err(InnerError::new(
                "map() expects an array of [key, value] pairs",
            ))
        }
    };
    let mut out = IndexMap::new();
    for (i, pair) in pairs.iter().enumerate() {
        match pair {
            Value::Array(kv) => {
                let kv = kv.borrow();
                if kv.len() != 2 {
                    return Err(InnerError::new(format!(
                        "map() entry at index {i} must be a 2-element array"
                    )));
                }
                let key = match &kv[0] {
                    Value::String(s) => s.clone(),
                    v => {
                        return Err(InnerError::new(format!(
                            "map() key at index {i} must be a string, got {}",
                            v.type_name()
                        )))
                    }
                };
                out.insert(key, kv[1].clone());
            }
            _ => {
                return Err(InnerError::new(format!(
                    "map() entry at index {i} must be a 2-element array"
                )))
            }
        }
    }
    Ok(Value::map(out))
}

fn builtin_type(args: &[Value]) -> InnerResult<Value> {
    expect_arity("type", args, 1, 1)?;
    Ok(Value::String(args[0].type_name().to_string()))
}

fn builtin_len(args: &[Value]) -> InnerResult<Value> {
    expect_arity("len", args, 1, 1)?;
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.chars().count() as i64)),
        Value::Array(rc) => Ok(Value::Int(rc.borrow().len() as i64)),
        Value::Map(rc) => Ok(Value::Int(rc.borrow().len() as i64)),
        other => Err(InnerError::new(format!(
            "len() not supported for {}",
            other.type_name()
        ))),
    }
}

fn builtin_print(args: &[Value]) -> InnerResult<Value> {
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
    }

    #[test]
    fn test_index() {
        let m = Value::map(indexmap::indexmap! {
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
        let arr = Value::array((0i64..5).map(Value::from).collect());
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
            Value::array((0i64..5).rev().map(Value::from).collect()),
        );
        assert!(try_run("arr[s:]", &[("arr", arr.clone()), ("s", Value::Float(1.0))]).is_err());
        assert!(try_run("arr[:s]", &[("arr", arr.clone()), ("s", Value::Float(3.0))]).is_err());
        assert!(try_run("arr[::s]", &[("arr", arr), ("s", Value::Float(2.0))]).is_err());
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
        let m = Value::map(indexmap::indexmap! {
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
    fn test_map() {
        assert_eval(
            r#"map([["a", 1], ["b", 2]])"#,
            &[],
            Value::map(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            }),
        );
        assert_eval(
            r#"{"a": 1, "b": 2}"#,
            &[],
            Value::map(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            }),
        );
        assert_eval(
            r#"{"x": true,}"#,
            &[],
            Value::map(indexmap::indexmap! { "x".into() => Value::Bool(true) }),
        );
        assert_eval("{}", &[], Value::map(indexmap::indexmap! {}));
        assert_eval(r#"{"pre" + "fix": 9}["prefix"]"#, &[], Value::from(9i64));
        assert_eval(
            r#"{"a": {"b": 2}}"#,
            &[],
            Value::map(indexmap::indexmap! {
                "a".into() => Value::map(indexmap::indexmap! { "b".into() => Value::from(2i64) }),
            }),
        );
        // insertion order must not affect equality
        assert_eval(
            r#"{"a": 1, "b": 2} == {"b": 2, "a": 1}"#,
            &[],
            Value::Bool(true),
        );
    }

    #[test]
    fn test_cast() {
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
        let m = Value::map(
            indexmap::indexmap! { "x".into() => Value::from(1i64), "y".into() => Value::from(2i64) },
        );
        assert_eval("list(m)", &[("m", m)], Value::from(vec!["x", "y"]));
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
            fn type_name(&self) -> &'static str {
                "Counter"
            }
            fn call_method(&self, method: &str, args: &[Value]) -> InnerResult<Value> {
                match method {
                    "__add__" => match args.first() {
                        Some(Value::Int(n)) => Ok(Value::object(Counter(self.0 + n))),
                        _ => Err(InnerError::new("expected int")),
                    },
                    "__eq__" => match args.first() {
                        Some(Value::Object(other)) => {
                            let other = other.display().parse::<i64>().unwrap_or(i64::MIN);
                            Ok(Value::Bool(self.0 == other))
                        }
                        _ => Ok(Value::Bool(false)),
                    },
                    m => Err(InnerError::new(format!("no method {m}"))),
                }
            }
            fn display(&self) -> String {
                self.0.to_string()
            }
        }

        let mut env = default_env();
        env.insert("c".to_string(), Value::object(Counter(10)));
        let result = eval_inner(&parse("c + 5").unwrap(), &mut env).unwrap();
        assert!(matches!(result, Value::Object(_)));
        assert_eq!(result.to_string(), "15");
        env.insert("d".to_string(), Value::object(Counter(10)));
        {
            let result = eval_inner(&parse("c == d").unwrap(), &mut env).unwrap();
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
            fn type_name(&self) -> &'static str {
                "Bag"
            }
            fn call_method(&self, method: &str, args: &[Value]) -> InnerResult<Value> {
                match method {
                    "__getitem__" => match args.first() {
                        Some(Value::String(k)) => self
                            .0
                            .iter()
                            .find(|(key, _)| key == k)
                            .map(|(_, v)| Value::Int(*v))
                            .ok_or_else(|| InnerError::new(format!("key '{k}' not found"))),
                        _ => Err(InnerError::new("__getitem__ expects a string")),
                    },
                    "__iter__" => Ok(Value::array(
                        self.0
                            .iter()
                            .map(|(k, _)| Value::String(k.clone()))
                            .collect(),
                    )),
                    m => Err(InnerError::new(format!("no method {m}"))),
                }
            }
        }

        let mut env = default_env();
        env.insert(
            "bag".into(),
            Value::object(Bag(vec![("x".into(), 10), ("y".into(), 20)])),
        );

        // __getitem__
        assert_eval(
            r#"bag["x"]"#,
            &[("bag", env["bag"].clone())],
            Value::from(10i64),
        );
        assert_eval(
            r#"bag["y"]"#,
            &[("bag", env["bag"].clone())],
            Value::from(20i64),
        );
        assert!(try_run(r#"bag["z"]"#, &[("bag", env["bag"].clone())]).is_err());

        // __iter__ via for-in
        assert_eval(
            "keys = []; for k in bag { keys = keys + [k] } keys",
            &[("bag", env["bag"].clone())],
            Value::from(vec!["x", "y"]),
        );
    }

    #[test]
    fn test_var() {
        let mut env = default_env();
        assert!(eval(&parse("missing").unwrap(), &mut env).is_err());
    }

    #[test]
    fn test_native_func() {
        let mut env = default_env();
        env.insert(
            "join_path".into(),
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
            let result = eval(
                &parse(r#"join_path("base", "file.json")"#).unwrap(),
                &mut env,
            )
            .unwrap();
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
        let m = Value::map(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("len(m)", &[("m", m)], Value::from(2i64));
        assert!(try_run("len(42)", &[]).is_err());
    }

    #[test]
    fn test_type_builtin() {
        assert_eval("type(null)", &[], Value::from("null"));
        assert_eval("type(true)", &[], Value::from("bool"));
        assert_eval("type(42)", &[], Value::from("int"));
        assert_eval("type(3.14)", &[], Value::from("float"));
        assert_eval(r#"type("hello")"#, &[], Value::from("string"));
        assert_eval("type([1, 2])", &[], Value::from("list"));
        assert_eval(r#"type({"a": 1})"#, &[], Value::from("map"));
    }

    #[test]
    fn test_complex_expr() {
        let obj = Value::map(indexmap::indexmap! {
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
        let n = MAX_EVAL_DEPTH - 1;
        let expr = format!("1{}", "+1".repeat(n));
        {
            let result = eval(&parse(&expr).unwrap(), &mut default_env()).unwrap();
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
        let mut a = Value::array(vec![Value::Int(1)]);
        let mut b = Value::array(vec![Value::Int(1)]);
        for _ in 0..MAX_EVAL_DEPTH {
            a = Value::array(vec![a]);
            b = Value::array(vec![b]);
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
    fn test_map_index_assign() {
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
        let m = Value::map(indexmap::indexmap! {
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
        let m = Value::map(indexmap::indexmap! {
            "a".into() => Value::from(10i64),
            "b".into() => Value::from(20i64),
        });
        assert_eval(
            "s = 0; for pair in m.items() { s = s + pair[1] } s",
            &[("m", m)],
            Value::from(30i64),
        );
    }
}
