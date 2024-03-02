use rhai::export_module;

#[export_module]
pub(crate) mod console_module {
    use rhai::plugin::*;

    pub fn log(message: &str) {
        println!("{}", message);
    }

    pub fn dbg(_message: &str) {
        println!("[debug]{}", _message);
    }

    pub fn info(_message: &str) {
        println!("[info]{}", _message);
    }

    pub fn warn(_message: &str) {
        println!("[warn]{}", _message);
    }

    pub fn error(_message: &str) {
        println!("[error]{}", _message);
    }
}
