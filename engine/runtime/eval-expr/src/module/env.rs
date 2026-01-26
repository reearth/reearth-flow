use rhai::plugin::*;

use crate::{engine::EngineEnv, utils};

#[export_module]
pub(crate) mod env_module {
    pub fn get(engine: &mut EngineEnv, name: &str) -> Dynamic {
        let v = engine
            .get(name)
            .unwrap_or_else(|| panic!("fail to get engine '{name}'"));
        utils::value_to_dynamic(&v)
    }

    pub fn set(engine: &mut EngineEnv, name: &str, value: Dynamic) {
        engine.set(name, utils::dynamic_to_value(&value));
    }
}

#[export_module]
pub(crate) mod scope_module {
    use crate::scope::ScopeEnv;

    pub fn get(env: &mut ScopeEnv, name: &str) -> Dynamic {
        if let Some(v) = env.get(name) {
            return utils::value_to_dynamic(&v);
        }

        Dynamic::UNIT
    }

    pub fn set(env: &mut ScopeEnv, name: &str, value: Dynamic) {
        env.set(name, utils::dynamic_to_value(&value));
    }
}
