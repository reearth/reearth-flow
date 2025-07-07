use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use reearth_flow_common::uri::PROTOCOL_SEPARATOR;
use reearth_flow_common::xml;

use super::errors::{Result, XmlProcessorError};
use crate::xml::schema_fetcher::SchemaFetcher;

/// Resolved schema with its content and dependencies
#[derive(Debug, Clone)]
pub struct ResolvedSchema {
    #[allow(dead_code)]
    pub url: String,
    pub content: String,
    #[allow(dead_code)]
    pub dependencies: Vec<String>,
}

/// Result of schema resolution containing all schemas and their relationships
#[derive(Debug)]
pub struct SchemaResolutionResult {
    pub schemas: HashMap<String, ResolvedSchema>,
    #[allow(dead_code)]
    pub root_schema: String,
}

/// Resolves XML Schema dependencies by recursively fetching and processing
/// import/include directives
pub struct XmlSchemaResolver {
    fetcher: Arc<dyn SchemaFetcher>,
    cache: Arc<parking_lot::Mutex<HashMap<String, String>>>,
}

impl XmlSchemaResolver {
    pub fn new(fetcher: Arc<dyn SchemaFetcher>) -> Self {
        Self {
            fetcher,
            cache: Arc::new(parking_lot::Mutex::new(HashMap::new())),
        }
    }

    /// Resolve a schema and all its dependencies recursively
    pub fn resolve_schema_dependencies(&self, root_url: &str) -> Result<SchemaResolutionResult> {
        let mut visited = HashSet::new();
        let mut to_process = vec![root_url.to_string()];
        let mut schemas = HashMap::new();

        // Phase 1: Collect all dependencies recursively
        while let Some(url) = to_process.pop() {
            if visited.contains(&url) {
                continue; // Skip circular references
            }
            visited.insert(url.clone());

            // Fetch schema content
            let content = self.fetch_with_cache(&url)?;

            // Extract dependencies from the schema
            let dependencies = self.extract_dependencies(&content, &url)?;

            // Add unvisited dependencies to processing queue
            for dep in &dependencies {
                if !visited.contains(dep) {
                    to_process.push(dep.clone());
                }
            }

            schemas.insert(
                url.clone(),
                ResolvedSchema {
                    url: url.clone(),
                    content,
                    dependencies,
                },
            );
        }

        Ok(SchemaResolutionResult {
            schemas,
            root_schema: root_url.to_string(),
        })
    }

    /// Fetch schema content with caching
    fn fetch_with_cache(&self, url: &str) -> Result<String> {
        // Check cache first
        {
            let cache = self.cache.lock();
            if let Some(content) = cache.get(url) {
                return Ok(content.clone());
            }
        }

        // Fetch if not cached
        let content = self.fetcher.fetch_schema(url)?;

        // Store in cache
        {
            let mut cache = self.cache.lock();
            cache.insert(url.to_string(), content.clone());
        }

        Ok(content)
    }

    /// Extract import and include dependencies from schema content
    fn extract_dependencies(&self, schema_content: &str, base_url: &str) -> Result<Vec<String>> {
        let doc = xml::parse(schema_content)
            .map_err(|e| XmlProcessorError::Validator(format!("Failed to parse schema: {e}")))?;

        let root = xml::get_root_readonly_node(&doc)
            .map_err(|e| XmlProcessorError::Validator(format!("Failed to get root node: {e}")))?;

        let mut dependencies = Vec::new();

        // Find all xs:import and xs:include elements
        self.collect_dependencies_recursive(&root, base_url, &mut dependencies)?;

        Ok(dependencies)
    }

    /// Recursively collect schema dependencies from import/include elements
    fn collect_dependencies_recursive(
        &self,
        node: &xml::XmlRoNode,
        base_url: &str,
        dependencies: &mut Vec<String>,
    ) -> Result<()> {
        let tag = xml::get_readonly_node_tag(node);

        // Check if this is an import or include element
        if tag.ends_with(":import") || tag.ends_with(":include") {
            if let Some(schema_location) = self.get_attribute(node, "schemaLocation") {
                let resolved_url = self.resolve_url(base_url, &schema_location)?;
                dependencies.push(resolved_url);
            }
        }

        // Process child nodes
        let children = node.get_child_nodes();
        for child in children {
            if let Some(node_type) = child.get_type() {
                if node_type == xml::XmlNodeType::ElementNode {
                    self.collect_dependencies_recursive(&child, base_url, dependencies)?;
                }
            }
        }

        Ok(())
    }

    /// Get attribute value from a node
    fn get_attribute(&self, node: &xml::XmlRoNode, name: &str) -> Option<String> {
        let attrs = node.get_attributes();
        attrs
            .into_iter()
            .find(|(attr_name, _)| attr_name == name)
            .map(|(_, value)| value)
    }

    /// Resolve a potentially relative URL against a base URL
    fn resolve_url(&self, base_url: &str, url: &str) -> Result<String> {
        // If URL is already absolute, return as-is
        if url.contains(PROTOCOL_SEPARATOR) {
            return Ok(url.to_string());
        }

        // If URL starts with /, it's absolute path on the same host
        if url.starts_with('/') {
            // Extract protocol and host from base URL
            if let Some(separator_pos) = base_url.find(PROTOCOL_SEPARATOR) {
                let protocol_end = separator_pos + PROTOCOL_SEPARATOR.len();
                if let Some(path_start) = base_url[protocol_end..].find('/') {
                    let base_host = &base_url[..protocol_end + path_start];
                    return Ok(format!("{}{}", base_host, url));
                }
            }
        }

        // Otherwise, resolve as relative URL
        if let Some(last_slash) = base_url.rfind('/') {
            let base_dir = &base_url[..last_slash];
            Ok(format!("{}/{}", base_dir, url))
        } else {
            Ok(format!("{}/{}", base_url, url))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap as StdHashMap;

    #[derive(Clone)]
    struct MockSchemaFetcher {
        responses: StdHashMap<String, Result<String>>,
    }

    impl MockSchemaFetcher {
        fn new() -> Self {
            Self {
                responses: StdHashMap::new(),
            }
        }

        fn with_response(mut self, url: &str, response: Result<String>) -> Self {
            self.responses.insert(url.to_string(), response);
            self
        }
    }

    impl SchemaFetcher for MockSchemaFetcher {
        fn fetch_schema(&self, url: &str) -> Result<String> {
            self.responses.get(url).cloned().unwrap_or_else(|| {
                Err(XmlProcessorError::Validator(format!(
                    "No mock response for URL: {url}"
                )))
            })
        }
    }

    #[test]
    fn test_resolve_single_schema() {
        let schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="test" type="xs:string"/>
</xs:schema>"#;

        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/schema.xsd", Ok(schema.to_string()));

        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));
        let result = resolver
            .resolve_schema_dependencies("http://example.com/schema.xsd")
            .unwrap();

        assert_eq!(result.schemas.len(), 1);
        assert!(result.schemas.contains_key("http://example.com/schema.xsd"));
        assert_eq!(result.root_schema, "http://example.com/schema.xsd");
    }

    #[test]
    fn test_resolve_schema_with_import() {
        let main_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:import namespace="http://example.com/types" schemaLocation="types.xsd"/>
    <xs:element name="main" type="xs:string"/>
</xs:schema>"#;

        let types_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/types">
    <xs:simpleType name="MyType">
        <xs:restriction base="xs:string"/>
    </xs:simpleType>
</xs:schema>"#;

        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/main.xsd", Ok(main_schema.to_string()))
            .with_response("http://example.com/types.xsd", Ok(types_schema.to_string()));

        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));
        let result = resolver
            .resolve_schema_dependencies("http://example.com/main.xsd")
            .unwrap();

        assert_eq!(result.schemas.len(), 2);
        assert!(result.schemas.contains_key("http://example.com/main.xsd"));
        assert!(result.schemas.contains_key("http://example.com/types.xsd"));

        let main = &result.schemas["http://example.com/main.xsd"];
        assert_eq!(main.dependencies.len(), 1);
        assert_eq!(main.dependencies[0], "http://example.com/types.xsd");
    }

    #[test]
    fn test_resolve_schema_with_circular_dependency() {
        let schema_a = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:import schemaLocation="b.xsd"/>
</xs:schema>"#;

        let schema_b = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:import schemaLocation="a.xsd"/>
</xs:schema>"#;

        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/a.xsd", Ok(schema_a.to_string()))
            .with_response("http://example.com/b.xsd", Ok(schema_b.to_string()));

        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));
        let result = resolver
            .resolve_schema_dependencies("http://example.com/a.xsd")
            .unwrap();

        // Should handle circular dependency without infinite loop
        assert_eq!(result.schemas.len(), 2);
    }

    #[test]
    fn test_url_resolution() {
        let resolver = XmlSchemaResolver::new(Arc::new(MockSchemaFetcher::new()));

        // Test absolute URL
        assert_eq!(
            resolver
                .resolve_url(
                    "http://example.com/dir/base.xsd",
                    "http://other.com/schema.xsd"
                )
                .unwrap(),
            "http://other.com/schema.xsd"
        );

        // Test relative URL
        assert_eq!(
            resolver
                .resolve_url("http://example.com/dir/base.xsd", "types.xsd")
                .unwrap(),
            "http://example.com/dir/types.xsd"
        );

        // Test absolute path
        assert_eq!(
            resolver
                .resolve_url("http://example.com/dir/base.xsd", "/schemas/types.xsd")
                .unwrap(),
            "http://example.com/schemas/types.xsd"
        );
    }
}
