use rhai::export_module;

#[export_module]
pub(crate) mod xml_module {
    use reearth_flow_common::xml;
    use rhai::plugin::*;

    pub fn find_node_tags_by_xpath(content: &str, xpath: &str) -> Vec<String> {
        let Ok(document) = xml::parse(content) else {
            return vec![];
        };
        let Ok(ctx) = xml::create_context(&document) else {
            return vec![];
        };
        let Ok(root) = xml::get_root_readonly_node(&document) else {
            return vec![];
        };
        let Ok(nodes) = xml::find_readonly_nodes_by_xpath(&ctx, xpath, &root) else {
            return vec![];
        };
        nodes.iter().map(xml::get_readonly_node_tag).collect()
    }

    pub fn exists_node_by_xpath(content: &str, xpath: &str) -> bool {
        !find_node_tags_by_xpath(content, xpath).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::xml_module::*;

    #[test]
    fn test_find_node_tags_by_xpath() {
        let content = r#"
            <root>
                <node1>Value 1</node1>
                <node2>Value 2</node2>
                <node3>Value 3</node3>
            </root>
        "#;

        // Test case 1: Valid XPath expression
        let xpath = "//node2";
        let expected_result = vec!["node2".to_string()];
        assert_eq!(find_node_tags_by_xpath(content, xpath), expected_result);

        // Test case 2: Empty content
        let content = "";
        let xpath = "//node";
        let expected_result: Vec<String> = vec![];
        assert_eq!(find_node_tags_by_xpath(content, xpath), expected_result);

        // Test case 3: Non-existent node
        let content = r#"
            <root>
                <node1>Value 1</node1>
                <node2>Value 2</node2>
                <node3>Value 3</node3>
            </root>
        "#;
        let xpath = "//node4";
        let expected_result: Vec<String> = vec![];
        assert_eq!(find_node_tags_by_xpath(content, xpath), expected_result);
    }
}
