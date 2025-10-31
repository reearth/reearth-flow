use std::sync::{Arc, RwLock};

use rhai::{Engine as ScriptEngine, Scope as RhaiScope};

use super::module::console::console_module;
use super::module::env::{env_module, scope_module};
use super::module::file::file_module;
use super::module::str::str_module;
use crate::module::collection::collection_module;
use crate::module::datetime::datetime_module;
use crate::module::json::json_module;
use crate::module::math::math_module;
use crate::module::xml::xml_module;
use crate::{error::Error, scope::Scope, ShareLock, Value, Vars};

#[derive(Debug, Default, Clone)]
pub struct Engine {
    pub(crate) script_engine: Arc<ScriptEngine>,
    pub(crate) scope: ShareLock<RhaiScope<'static>>,
    pub(crate) vars: ShareLock<Vars>,
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

impl Engine {
    pub fn new() -> Self {
        let mut script_engine = ScriptEngine::new();
        script_engine.set_allow_looping(true);
        script_engine.set_allow_anonymous_fn(true);
        script_engine.set_allow_shadowing(true);
        let scope = rhai::Scope::new();
        vec![
            rhai::exported_module!(env_module),
            rhai::exported_module!(scope_module),
        ]
        .iter()
        .for_each(|module| {
            script_engine.register_global_module(module.clone().into());
        });
        script_engine
            .register_static_module("console", rhai::exported_module!(console_module).into());
        script_engine.register_static_module("file", rhai::exported_module!(file_module).into());
        script_engine.register_static_module("str", rhai::exported_module!(str_module).into());
        script_engine.register_static_module("xml", rhai::exported_module!(xml_module).into());
        script_engine.register_static_module("json", rhai::exported_module!(json_module).into());
        script_engine.register_static_module(
            "collection",
            rhai::exported_module!(collection_module).into(),
        );
        script_engine
            .register_static_module("datetime", rhai::exported_module!(datetime_module).into());
        script_engine.register_static_module("math", rhai::exported_module!(math_module).into());

        let engine = Self {
            script_engine: Arc::new(script_engine),
            scope: Arc::new(RwLock::new(scope)),
            vars: Arc::new(RwLock::new(Vars::new())),
        };
        engine.init();
        engine
    }

    pub fn init(&self) {
        self.scope.write().unwrap().set_or_push("env", self.clone());
    }

    pub fn new_scope(&self) -> Scope {
        Scope::new(self)
    }

    pub fn vars(&self) -> Vars {
        self.vars.read().unwrap().clone()
    }

    pub fn set_scope_var<T: Send + Sync + Clone + 'static>(&self, name: &str, v: &T) {
        self.scope.write().unwrap().set_or_push(name, v.clone());
    }

    pub fn append(&self, vars: &Vars) {
        let env = &mut self.vars.write().unwrap();
        for (name, v) in vars {
            env.entry(name.to_string())
                .and_modify(|i| *i = v.clone())
                .or_insert(v.clone());
        }
    }

    pub fn eval<T: rhai::Variant + Clone>(&self, expr: &str) -> crate::Result<T> {
        let scr = Arc::clone(&self.script_engine);
        let mut scope = self
            .scope
            .write()
            .map_err(|_| Error::ExprInternalRuntime("lock".to_string()))?;
        match scr.eval_with_scope::<T>(&mut scope, expr) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(Error::ExprInternalRuntime(format!(
                "expr code = {expr}, err = {err}"
            ))),
        }
    }

    pub fn eval_ast<T: rhai::Variant + Clone>(&self, ast: &rhai::AST) -> crate::Result<T> {
        let scr = Arc::clone(&self.script_engine);
        let mut scope = self
            .scope
            .write()
            .map_err(|_| Error::ExprInternalRuntime("lock".to_string()))?;
        match scr.eval_ast_with_scope::<T>(&mut scope, ast) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(Error::ExprInternalRuntime(format!(
                "ast = {ast:?} err = {err}"
            ))),
        }
    }

    pub fn eval_scope<T: rhai::Variant + Clone>(
        &self,
        expr: &str,
        scope: &Scope,
    ) -> crate::Result<T> {
        let scr = Arc::clone(&self.script_engine);
        let mut scope = scope.scope.write().unwrap();

        match scr.eval_with_scope::<T>(&mut scope, expr) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(Error::ExprInternalRuntime(format!(
                "expr code = {expr}, err = {err}"
            ))),
        }
    }

    pub fn eval_scope_ast<T: rhai::Variant + Clone>(
        &self,
        ast: &rhai::AST,
        scope: &Scope,
    ) -> crate::Result<T> {
        let scr = Arc::clone(&self.script_engine);
        let mut scope = scope.scope.write().unwrap();

        match scr.eval_ast_with_scope::<T>(&mut scope, ast) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(Error::ExprInternalRuntime(format!(
                "ast = {ast:?} err = {err}"
            ))),
        }
    }

    pub fn compile(&self, expr: &str) -> crate::Result<rhai::AST> {
        let scr = Arc::clone(&self.script_engine);
        scr.compile(expr)
            .map_err(|err| Error::ExprCompile(format!("expr code = {expr}, err = {err}")))
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let vars = self.vars.read().unwrap();
        vars.get(name).cloned()
    }

    pub fn set(&self, name: &str, value: Value) {
        let mut vars = self.vars.write().unwrap();
        vars.entry(name.to_string())
            .and_modify(|i| *i = value.clone())
            .or_insert(value);
    }

    pub fn remove(&self, name: &str) {
        let mut vars = self.vars.write().unwrap();
        vars.remove(name);
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_eval() {
        let engine = Engine::new();
        let script = r#"
        let v = 5;
        v
        "#;

        let result = engine.eval::<i64>(script);
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn test_eval_error() {
        let engine = Engine::new();

        let script = r#"
        let v = 5
        v
        "#;

        let script_result = engine.eval::<i64>(script);
        let reuslt = match script_result {
            Ok(..) => false,
            Err(_) => true,
        };
        assert!(reuslt);
    }

    #[test]
    fn test_get() {
        let engine = Engine::new();
        let vars = Vars::from_iter([("a".to_string(), 10.into()), ("b".to_string(), "b".into())]);
        engine.append(&vars);

        let script = r#"
        let a = env.get("a");
        a
        "#;
        let result = engine.eval::<i64>(script);
        print!("hogehoge {result:?}");
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_scope_eval() {
        let engine = Engine::new();
        let scope = engine.new_scope();
        let script = r#"
        let v = 5;
        v
        "#;

        let result = scope.eval::<i64>(script);
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn env_room_get() {
        let engine = Engine::new();
        let scope = engine.new_scope();

        let vars = Vars::from_iter([("a".to_string(), 10.into()), ("b".to_string(), "b".into())]);
        scope.append(&vars);

        let script = r#"
        let a = env.get("a");
        a
        "#;
        let result = scope.eval::<i64>(script);
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_scope_share_global_vars() {
        let engine = Engine::new();

        engine.set("abc", serde_json::json!(1.5));
        let scope = engine.new_scope();
        let script = r#"
        let v = 5;
        let v2 = env.get("abc");
        console::info(`${v2}`);
        v2
        "#;

        let result = scope.eval::<f64>(script);
        assert_eq!(result.unwrap(), 1.5);
    }
}
