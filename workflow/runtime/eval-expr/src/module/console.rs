use rhai::export_module;

#[export_module]
pub(crate) mod console_module {
    use rhai::plugin::*;

    use crate::utils;

    pub fn dbg(_message: Dynamic) {
        println!("[debug]{:?}", utils::dynamic_to_value(&_message));
    }

    pub fn info(_message: Dynamic) {
        println!("[info]{:?}", utils::dynamic_to_value(&_message));
    }

    pub fn warn(_message: Dynamic) {
        println!("[warn]{:?}", utils::dynamic_to_value(&_message));
    }

    pub fn error(_message: Dynamic) {
        println!("[error]{:?}", utils::dynamic_to_value(&_message));
    }
}
