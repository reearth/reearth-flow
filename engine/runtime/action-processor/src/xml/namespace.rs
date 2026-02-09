#[cfg(test)]
mod tests {
    use reearth_flow_common::xml::{self, XmlRoNamespace};

    use crate::xml::types::ValidationResult;

    fn recursive_check_namespace(
        node: xml::XmlRoNode,
        namespaces: &Vec<XmlRoNamespace>,
    ) -> Vec<ValidationResult> {
        let mut result = Vec::new();
        match node.get_namespace() {
            Some(ns) => {
                if !namespaces.iter().any(|n| n.get_prefix() == ns.get_prefix()) {
                    result.push(ValidationResult::new(
                        "NamespaceError",
                        &format!("No namespace declaration for {}", ns.get_prefix()),
                    ));
                }
            }
            None => {
                let tag = xml::get_readonly_node_tag(&node);
                if tag.contains(':') {
                    let prefix = tag.split(':').collect::<Vec<&str>>()[0];
                    if !namespaces.iter().any(|n| n.get_prefix() == prefix) {
                        result.push(ValidationResult::new(
                            "NamespaceError",
                            &format!("No namespace declaration for {prefix}"),
                        ));
                    }
                } else {
                    result.push(ValidationResult::new(
                        "NamespaceError",
                        "No namespace declaration",
                    ));
                }
            }
        };
        let child_node = node.get_child_nodes();
        let child_nodes = child_node
            .into_iter()
            .filter(|n| n.get_type() == xml::XmlNodeType::Element)
            .collect::<Vec<_>>();
        for child in child_nodes {
            let child_result = recursive_check_namespace(child, namespaces);
            result.extend(child_result);
        }
        result
    }

    #[test]
    fn test_recursive_check_namespace_valid() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<test:root xmlns:test="http://example.com/test">
    <test:element>Valid content</test:element>
    <test:nested>
        <test:child>Child content</test:child>
    </test:nested>
</test:root>"#;

        let document = xml::parse(xml_content).unwrap();
        let root_node = xml::get_root_readonly_node(&document).unwrap();
        let namespaces: Vec<XmlRoNamespace> = root_node
            .get_namespace_declarations()
            .into_iter()
            .map(|ns| ns.into())
            .collect();

        let result = recursive_check_namespace(root_node, &namespaces);
        assert!(
            result.is_empty(),
            "Should not have namespace errors for valid XML with declared namespaces"
        );
    }

    #[test]
    fn test_recursive_check_namespace_missing_declaration() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:test="http://example.com/test">
    <test:element>Valid element</test:element>
    <unknown:element>This uses undeclared namespace</unknown:element>
</root>"#;

        let document = xml::parse(xml_content).unwrap();
        let root_node = xml::get_root_readonly_node(&document).unwrap();
        let namespaces: Vec<XmlRoNamespace> = root_node
            .get_namespace_declarations()
            .into_iter()
            .map(|ns| ns.into())
            .collect();

        let result = recursive_check_namespace(root_node, &namespaces);
        assert!(!result.is_empty(), "Should have namespace errors");
        assert_eq!(result.len(), 2, "Should have two errors");
        assert!(result.iter().all(|r| r.error_type == "NamespaceError"));
        assert!(result
            .iter()
            .any(|r| r.message == "No namespace declaration"));
        assert!(result.iter().any(|r| r.message.contains("unknown")));
    }

    #[test]
    fn test_recursive_check_namespace_nested_errors() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:valid="http://example.com/valid">
    <valid:element>
        <invalid:nested>
            <alsoInvalid:deep>Error here</alsoInvalid:deep>
        </invalid:nested>
    </valid:element>
</root>"#;

        let document = xml::parse(xml_content).unwrap();
        let root_node = xml::get_root_readonly_node(&document).unwrap();
        let namespaces: Vec<XmlRoNamespace> = root_node
            .get_namespace_declarations()
            .into_iter()
            .map(|ns| ns.into())
            .collect();

        let result = recursive_check_namespace(root_node, &namespaces);
        assert_eq!(result.len(), 3, "Should have three namespace errors");
        assert!(result.iter().all(|r| r.error_type == "NamespaceError"));
        assert!(result
            .iter()
            .any(|r| r.message == "No namespace declaration"));
        assert!(result.iter().any(|r| r.message.contains("invalid")));
        assert!(result.iter().any(|r| r.message.contains("alsoInvalid")));
    }

    #[test]
    fn test_recursive_check_namespace_no_prefix_elements() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <child1>
        <child2>Content</child2>
    </child1>
</root>"#;

        let document = xml::parse(xml_content).unwrap();
        let root_node = xml::get_root_readonly_node(&document).unwrap();
        let namespaces: Vec<XmlRoNamespace> = root_node
            .get_namespace_declarations()
            .into_iter()
            .map(|ns| ns.into())
            .collect();

        let result = recursive_check_namespace(root_node, &namespaces);
        assert_eq!(result.len(), 3, "Should have three errors for all elements");
        assert!(result.iter().all(|r| r.error_type == "NamespaceError"));
        assert!(result
            .iter()
            .all(|r| r.message == "No namespace declaration"));
    }
}
