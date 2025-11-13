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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_node_info() {
        set_node_info("node_1".to_string(), Some("TestNode".to_string()));
        
        let info = get_node_info();
        assert!(info.is_some());
        
        let (id, name) = info.unwrap();
        assert_eq!(id, "node_1");
        assert_eq!(name, Some("TestNode".to_string()));
        
        clear_node_info();
    }

    #[test]
    fn test_set_node_info_without_name() {
        set_node_info("node_2".to_string(), None);
        
        let info = get_node_info();
        assert!(info.is_some());
        
        let (id, name) = info.unwrap();
        assert_eq!(id, "node_2");
        assert_eq!(name, None);
        
        clear_node_info();
    }

    #[test]
    fn test_clear_node_info() {
        set_node_info("node_3".to_string(), Some("Node3".to_string()));
        assert!(get_node_info().is_some());
        
        clear_node_info();
        assert!(get_node_info().is_none());
    }

    #[test]
    fn test_get_node_info_empty() {
        clear_node_info();
        let info = get_node_info();
        assert!(info.is_none());
    }

    #[test]
    fn test_overwrite_node_info() {
        set_node_info("node_old".to_string(), Some("Old".to_string()));
        set_node_info("node_new".to_string(), Some("New".to_string()));
        
        let info = get_node_info();
        let (id, _) = info.unwrap();
        assert_eq!(id, "node_new");
        
        clear_node_info();
    }

    #[test]
    fn test_node_info_japanese_name() {
        set_node_info("node_jp".to_string(), Some("建物処理ノード".to_string()));
        
        let info = get_node_info();
        let (_, name) = info.unwrap();
        assert_eq!(name, Some("建物処理ノード".to_string()));
        
        clear_node_info();
    }
}

