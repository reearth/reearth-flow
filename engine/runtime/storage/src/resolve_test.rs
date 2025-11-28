#[cfg(test)]
mod tests {
    use crate::resolve::StorageResolver;
    use reearth_flow_common::uri::Uri;
    use std::sync::Arc;

    #[test]
    fn test_storage_resolver_new() {
        let resolver = StorageResolver::new();
        assert!(Arc::strong_count(&resolver.storages) == 1);
    }

    #[test]
    fn test_storage_resolver_resolve_memory() {
        let resolver = StorageResolver::new();
        let uri = Uri::for_test("ram:///test");
        
        let result = resolver.resolve(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_storage_resolver_resolve_caching() {
        let resolver = StorageResolver::new();
        let uri = Uri::for_test("ram:///test");
        
        let storage1 = resolver.resolve(&uri).unwrap();
        let storage2 = resolver.resolve(&uri).unwrap();
        
        assert!(Arc::ptr_eq(&storage1, &storage2));
    }

    #[test]
    fn test_storage_resolver_different_root_uris() {
        let resolver = StorageResolver::new();
        let uri1 = Uri::for_test("ram:///test1");
        let uri2 = Uri::for_test("file:///test2");
        
        let storage1 = resolver.resolve(&uri1).unwrap();
        let storage2 = resolver.resolve(&uri2).unwrap();
        
        assert!(!Arc::ptr_eq(&storage1, &storage2));
    }

    #[test]
    fn test_storage_resolver_clone() {
        let resolver1 = StorageResolver::new();
        let resolver2 = resolver1.clone();
        
        let uri = Uri::for_test("ram:///test");
        let storage1 = resolver1.resolve(&uri).unwrap();
        let storage2 = resolver2.resolve(&uri).unwrap();
        
        assert!(Arc::ptr_eq(&storage1, &storage2));
    }

    #[test]
    fn test_storage_resolver_root_uri_normalization() {
        let resolver = StorageResolver::new();
        let uri1 = Uri::for_test("ram:///test/path/file.txt");
        let uri2 = Uri::for_test("ram:///test/another/file.txt");
        
        let storage1 = resolver.resolve(&uri1).unwrap();
        let storage2 = resolver.resolve(&uri2).unwrap();
        
        assert!(Arc::ptr_eq(&storage1, &storage2));
    }

    #[test]
    fn test_storage_resolver_default() {
        let resolver = StorageResolver::default();
        let uri = Uri::for_test("ram:///test");
        
        let result = resolver.resolve(&uri);
        assert!(result.is_ok());
    }
}

