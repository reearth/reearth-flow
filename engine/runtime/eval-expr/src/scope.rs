use std::sync::{Arc, RwLock};

use crate::{engine::Engine, error::Error, ShareLock, Value, Vars};

#[derive(Debug, Default, Clone)]
pub struct Scope {
    engine: Arc<Engine>,
    pub(crate) scope: ShareLock<rhai::Scope<'static>>,
    vars: ShareLock<Vars>,
}

impl Scope {
    pub fn new(engine: &Engine) -> Self {
        let scope = rhai::Scope::new();

        let scope = Self {
            engine: Arc::new(engine.clone()),
            scope: Arc::new(RwLock::new(scope)),
            vars: Arc::new(RwLock::new(Vars::new())),
        };
        scope
            .scope
            .write()
            .unwrap()
            .set_or_push("env", scope.clone());
        scope
    }

    pub fn vars(&self) -> Vars {
        self.vars.read().unwrap().clone()
    }

    pub fn set_scope_var<T: Send + Sync + Clone + 'static>(&self, name: &str, v: &T) {
        self.scope.write().unwrap().set_or_push(name, v.clone());
    }

    pub fn append(&self, vars: &Vars) {
        for (name, v) in vars {
            self.set(name, v.clone());
        }
    }

    pub fn output(&self, vars: &Vars) {
        self.engine.append(vars);
    }

    pub fn eval<T: rhai::Variant + Clone>(&self, expr: &str) -> crate::Result<T> {
        match self.engine.eval_scope::<T>(expr, self) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(Error::ExprInternalRuntime(format!("{err} in {expr}"))),
        }
    }

    pub fn eval_ast<T: rhai::Variant + Clone>(&self, ast: &rhai::AST) -> crate::Result<T> {
        match self.engine.eval_scope_ast::<T>(ast, self) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(Error::ExprInternalRuntime(format!(
                "ast = {ast:?}, err = {err}"
            ))),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let vars = self.vars.read().unwrap();
        vars.get(name).cloned().or_else(|| self.engine.get(name))
    }

    pub fn set(&self, name: &str, value: Value) {
        if self.engine.vars().contains_key(name) {
            return self.engine.set(name, value);
        }

        let mut vars = self.vars.write().unwrap();
        vars.entry(name.to_string())
            .and_modify(|i| *i = value.clone())
            .or_insert(value);
    }

    pub fn remove(&self, name: &str) {
        if self.engine.vars().contains_key(name) {
            return self.engine.remove(name);
        }

        let mut vars = self.vars.write().unwrap();
        vars.remove(name);
    }
}
