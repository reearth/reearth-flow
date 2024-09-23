use rhai::plugin::*;

use crate::{engine::Engine, utils};

#[export_module]
pub(crate) mod env_module {
    pub fn get(engine: &mut Engine, name: &str) -> Dynamic {
        let v = engine
            .get(name)
            .unwrap_or_else(|| panic!("fail to get engine '{name}'"));
        utils::value_to_dynamic(&v)
    }

    pub fn set(engine: &mut Engine, name: &str, value: Dynamic) {
        engine.set(name, utils::dynamic_to_value(&value));
    }
}

#[export_module]
pub(crate) mod scope_module {
    use crate::scope::Scope;

    pub fn get(env: &mut Scope, name: &str) -> Dynamic {
        if let Some(v) = env.get(name) {
            return utils::value_to_dynamic(&v);
        }

        Dynamic::UNIT
    }

    pub fn set(env: &mut Scope, name: &str, value: Dynamic) {
        env.set(name, utils::dynamic_to_value(&value));
    }
}
