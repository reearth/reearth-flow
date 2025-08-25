use std::cell::RefCell;

thread_local! {
    static CURRENT_NODE_INFO: RefCell<Option<(String, Option<String>)>> = const { RefCell::new(None) };
}

pub fn set_node_info(node_id: String, node_name: Option<String>) {
    CURRENT_NODE_INFO.with(|info| {
        *info.borrow_mut() = Some((node_id, node_name));
    });
}

pub fn get_node_info() -> Option<(String, Option<String>)> {
    CURRENT_NODE_INFO.with(|info| info.borrow().clone())
}

pub fn clear_node_info() {
    CURRENT_NODE_INFO.with(|info| {
        *info.borrow_mut() = None;
    });
}
