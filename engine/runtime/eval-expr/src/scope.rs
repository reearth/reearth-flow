use std::sync::{Arc, RwLock};

use crate::{engine::Engine, error::Error, ShareLock, Value, Vars};

/// Inner environment exposed to Rhai scripts as "env".
/// This is separate from `Scope` to avoid a cyclic reference:
/// `Scope` owns `rhai::Scope`, and if we stored `Scope` inside `rhai::Scope`,
/// the Arc reference counts would never reach zero.
#[derive(Debug, Clone)]
pub(crate) struct ScopeEnv {
    engine: Arc<Engine>,
    vars: ShareLock<Vars>,
}

impl ScopeEnv {
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
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    engine: Arc<Engine>,
    pub(crate) scope: ShareLock<rhai::Scope<'static>>,
    vars: ShareLock<Vars>,
}

impl Scope {
    pub fn new(engine: &Engine) -> Self {
        let engine = Arc::new(engine.clone());
        let vars = Arc::new(RwLock::new(Vars::new()));

        // Create the environment that will be exposed to Rhai scripts.
        // ScopeEnv only holds Arc references to engine and vars, not to
        // the rhai::Scope itself, which breaks the cyclic reference.
        let env = ScopeEnv {
            engine: Arc::clone(&engine),
            vars: Arc::clone(&vars),
        };

        let mut rhai_scope = rhai::Scope::new();
        rhai_scope.set_or_push("env", env);

        Self {
            engine,
            scope: Arc::new(RwLock::new(rhai_scope)),
            vars,
        }
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
            Err(err) => {
                if matches!(&err, Error::ExprInternalRuntime(_)) {
                    Err(err)
                } else {
                    Err(Error::ExprInternalRuntime(format!("{err} in {expr}")))
                }
            }
        }
    }

    pub fn eval_ast<T: rhai::Variant + Clone>(&self, ast: &rhai::AST) -> crate::Result<T> {
        match self.engine.eval_scope_ast::<T>(ast, self) {
            Ok(ret) => Ok(ret),
            Err(err) => {
                if matches!(&err, Error::ExprInternalRuntime(_)) {
                    Err(err)
                } else {
                    Err(Error::ExprInternalRuntime(format!(
                        "ast = {ast:?}, err = {err}"
                    )))
                }
            }
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
