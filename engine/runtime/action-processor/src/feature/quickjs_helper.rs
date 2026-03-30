use std::{collections::HashMap, fmt, sync::Arc};

use reearth_flow_types::{Attribute, Attributes};
use rquickjs::{Context, Function, Object, Runtime};
use serde_json::Value;

/// Opaque handle to a QuickJS runtime with pre-compiled functions.
///
/// Functions are compiled once via [`JsEngine::compile_expr`] or
/// [`JsEngine::compile_body`] and stored as JS globals. Callers invoke
/// them later by name via [`JsEngine::call`].
///
/// `Clone` creates a fresh runtime with the same source re-compiled,
/// so each processor thread gets its own isolated engine.
pub(super) struct JsEngine {
    _runtime: Runtime,
    context: Context,
    /// (registered_name, source, is_expr) for re-compilation on clone.
    entries: Vec<(String, String, bool)>,
}

impl JsEngine {
    pub fn new() -> Result<Self, String> {
        let runtime =
            Runtime::new().map_err(|e| format!("Failed to create JS runtime: {e}"))?;
        let context =
            Context::full(&runtime).map_err(|e| format!("Failed to create JS context: {e}"))?;
        Ok(Self {
            _runtime: runtime,
            context,
            entries: Vec::new(),
        })
    }

    /// Compile a JS expression (wrapped as `return (expr)`) and register it.
    /// Returns the registered function name.
    pub fn compile_expr(&mut self, source: &str) -> Result<String, String> {
        let name = format!("__fn{}", self.entries.len());
        let wrapper = format!("(function(value, env) {{ return ({source}); }})");
        self.register(&name, &wrapper)?;
        self.entries.push((name.clone(), source.to_string(), true));
        Ok(name)
    }

    /// Compile a JS function body (may contain statements and `return`) and
    /// register it. Returns the registered function name.
    pub fn compile_body(&mut self, source: &str) -> Result<String, String> {
        let name = format!("__fn{}", self.entries.len());
        let wrapper = format!("(function(value, env) {{ {source} }})");
        self.register(&name, &wrapper)?;
        self.entries.push((name.clone(), source.to_string(), false));
        Ok(name)
    }

    fn register(&self, name: &str, code: &str) -> Result<(), String> {
        let code = code.to_string();
        let name = name.to_string();
        self.context.with(|ctx| {
            let func: rquickjs::Value = ctx
                .eval(code.into_bytes())
                .map_err(|e| format!("Invalid JS: {e}"))?;
            ctx.globals()
                .set(&*name, func)
                .map_err(|e| format!("Failed to register function: {e}"))?;
            Ok(())
        })
    }

    /// Call a previously compiled function by name, returning the result as JSON.
    pub fn call(
        &self,
        name: &str,
        attrs: &Arc<Attributes>,
        env: &Option<HashMap<String, Value>>,
    ) -> Result<Value, String> {
        let attrs = Arc::clone(attrs);
        let name = name.to_string();
        self.context.with(|ctx| {
            let func: Function = ctx
                .globals()
                .get(&*name)
                .map_err(|e| format!("Function '{name}' not found: {e}"))?;
            let value_fn = make_value_fn(&ctx, attrs)
                .map_err(|e| format!("Failed to create value() function: {e}"))?;
            let js_env =
                make_env_js(&ctx, env).map_err(|e| format!("Failed to create env: {e}"))?;
            let result: rquickjs::Value = func
                .call((value_fn, js_env))
                .map_err(|e| format!("Failed to eval: {e}"))?;
            Ok(js_to_json(&result))
        })
    }

    /// Call a previously compiled function, giving the caller access to the raw
    /// JS result inside the context.
    pub fn call_raw<T, F>(
        &self,
        name: &str,
        attrs: &Arc<Attributes>,
        env: &Option<HashMap<String, Value>>,
        f: F,
    ) -> Result<T, String>
    where
        F: FnOnce(&rquickjs::Ctx<'_>, rquickjs::Value<'_>) -> Result<T, String>,
    {
        let attrs = Arc::clone(attrs);
        let name = name.to_string();
        self.context.with(|ctx| {
            let func: Function = ctx
                .globals()
                .get(&*name)
                .map_err(|e| format!("Function '{name}' not found: {e}"))?;
            let value_fn = make_value_fn(&ctx, attrs)
                .map_err(|e| format!("Failed to create value() function: {e}"))?;
            let js_env =
                make_env_js(&ctx, env).map_err(|e| format!("Failed to create env: {e}"))?;
            let result: rquickjs::Value = func
                .call((value_fn, js_env))
                .map_err(|e| format!("Failed to eval: {e}"))?;
            f(&ctx, result)
        })
    }
}

impl Clone for JsEngine {
    fn clone(&self) -> Self {
        let mut engine = Self::new().expect("Failed to create JS runtime for clone");
        for (_, source, is_expr) in &self.entries {
            if *is_expr {
                engine
                    .compile_expr(source)
                    .expect("Previously validated expr failed on clone");
            } else {
                engine
                    .compile_body(source)
                    .expect("Previously validated body failed on clone");
            }
        }
        engine
    }
}

impl fmt::Debug for JsEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsEngine")
            .field("entries", &self.entries.len())
            .finish()
    }
}

fn json_to_js<'js>(
    ctx: &rquickjs::Ctx<'js>,
    value: &Value,
) -> rquickjs::Result<rquickjs::Value<'js>> {
    match value {
        Value::Null => Ok(rquickjs::Value::new_null(ctx.clone())),
        Value::Bool(b) => Ok(rquickjs::Value::new_bool(ctx.clone(), *b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(rquickjs::Value::new_int(ctx.clone(), i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(rquickjs::Value::new_float(ctx.clone(), f))
            } else {
                Ok(rquickjs::Value::new_null(ctx.clone()))
            }
        }
        Value::String(s) => {
            let js_str = rquickjs::String::from_str(ctx.clone(), s)?;
            Ok(js_str.into_value())
        }
        Value::Array(arr) => {
            let js_arr = rquickjs::Array::new(ctx.clone())?;
            for (i, v) in arr.iter().enumerate() {
                let js_val = json_to_js(ctx, v)?;
                js_arr.set(i, js_val)?;
            }
            Ok(js_arr.into_value())
        }
        Value::Object(map) => {
            let js_obj = Object::new(ctx.clone())?;
            for (k, v) in map {
                let js_val = json_to_js(ctx, v)?;
                js_obj.set(k.as_str(), js_val)?;
            }
            Ok(js_obj.into_value())
        }
    }
}

pub(super) fn js_to_json(value: &rquickjs::Value<'_>) -> Value {
    if value.is_undefined() || value.is_null() {
        Value::Null
    } else if let Some(b) = value.as_bool() {
        Value::Bool(b)
    } else if let Some(i) = value.as_int() {
        Value::Number(i.into())
    } else if let Some(f) = value.as_float() {
        serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null)
    } else if let Some(s) = value.as_string() {
        s.to_string()
            .map(Value::String)
            .unwrap_or(Value::Null)
    } else if let Some(arr) = value.as_array() {
        let items: Vec<Value> = arr
            .iter::<rquickjs::Value>()
            .filter_map(|v| v.ok().map(|v| js_to_json(&v)))
            .collect();
        Value::Array(items)
    } else if let Some(obj) = value.as_object() {
        let mut map = serde_json::Map::new();
        if let Some(props) = obj.keys::<String>().collect::<Result<Vec<_>, _>>().ok() {
            for key in props {
                if let Ok(val) = obj.get::<_, rquickjs::Value>(&key) {
                    map.insert(key, js_to_json(&val));
                }
            }
        }
        Value::Object(map)
    } else {
        Value::Null
    }
}

fn make_value_fn<'js>(
    js: &rquickjs::Ctx<'js>,
    attrs: Arc<Attributes>,
) -> rquickjs::Result<Function<'js>> {
    let js2 = js.clone();
    Function::new(js.clone(), move |key: String| {
        match attrs.get(&Attribute::new(key)) {
            Some(v) => {
                let json_val: Value = v.clone().into();
                json_to_js(&js2, &json_val)
            }
            None => Ok(rquickjs::Value::new_undefined(js2.clone())),
        }
    })
}

fn make_env_js<'js>(
    js: &rquickjs::Ctx<'js>,
    global_params: &Option<HashMap<String, Value>>,
) -> Result<rquickjs::Value<'js>, rquickjs::Error> {
    let env_value: Value = if let Some(params) = global_params {
        Value::Object(
            params
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    } else {
        Value::Object(serde_json::Map::new())
    };
    json_to_js(js, &env_value)
}
