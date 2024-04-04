use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::convert::{as_document, as_document_type};
use crate::name::Name;
use crate::node::RefNode;
use crate::syntax::*;
use crate::traits::{Node, NodeType};

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) enum SpaceHandling {
    #[default]
    Default,
    Preserve,
}

pub(crate) trait EntityResolver {
    fn resolve(&self, entity: &str) -> Option<String>;
}

impl EntityResolver for RefNode {
    fn resolve(&self, entity: &str) -> Option<String> {
        let doc_type = match self.node_type() {
            NodeType::DocumentType => Some(self.clone()),
            NodeType::Document => {
                let document = as_document(self).unwrap();
                document.doc_type()
            }
            _ => match self.owner_document() {
                None => None,
                Some(document_node) => {
                    let document = as_document(&document_node).unwrap();
                    document.doc_type()
                }
            },
        };
        match doc_type {
            None => None,
            Some(doc_type) => {
                let doc_type = as_document_type(&doc_type).unwrap();
                let name = Name::from_str(entity).unwrap();
                match doc_type.entities().get(&name) {
                    None => None,
                    Some(entity) => entity.node_value(),
                }
            }
        }
    }
}

pub(crate) fn normalize_attribute_value(
    value: &str,
    resolver: &dyn EntityResolver,
    is_cdata: bool,
) -> String {
    let step_1 = normalize_end_of_lines(value);
    let step_3 = if step_1.is_empty() {
        step_1
    } else {
        let find = regex::Regex::new(
            r"(?P<entity_ref>[&%][\pL_][\pL\.\d_\-]*;)|(?P<char>&#\d+;)|(?P<char_hex>&#x[0-9a-fA-F]+;)|(?P<ws>[\u{09}\u{0A}\u{0D}])",
        )
        .unwrap();
        let mut step_2 = String::new();
        let mut last_end = 0;
        for capture in find.captures_iter(&step_1) {
            let (start, end, replacement) = if let Some(a_match) = capture.name("entity_ref") {
                //
                // TODO: this does not yet deal with entity references.
                //
                let replacement = match resolver.resolve(a_match.as_str()) {
                    None => panic!("unknown entity reference {}", a_match.as_str()),
                    Some(replacement) => {
                        normalize_attribute_value(&replacement, resolver, is_cdata)
                    }
                };
                (a_match.start(), a_match.end(), replacement)
            } else if let Some(a_match) = capture.name("char") {
                let replacement = char_from_entity(a_match.as_str());
                (a_match.start(), a_match.end(), replacement)
            } else if let Some(a_match) = capture.name("char_hex") {
                let replacement = char_from_entity(a_match.as_str());
                (a_match.start(), a_match.end(), replacement)
            } else if let Some(a_match) = capture.name("ws") {
                (a_match.start(), a_match.end(), "\u{20}".to_string())
            } else {
                panic!("unexpected result");
            };

            step_2.push_str(&step_1[last_end..start]);
            step_2.push_str(&replacement);
            last_end = end;
        }

        if last_end < value.len() {
            step_2.push_str(&step_1[last_end..]);
        }
        step_2
    };
    if is_cdata {
        step_3
    } else {
        step_3.trim_matches(' ').to_string()
    }
}

pub(crate) fn normalize_end_of_lines(value: &str) -> String {
    if value.is_empty() {
        value.to_string()
    } else {
        let line_ends = regex::Regex::new(r"\u{0D}[\u{0A}\u{85}]?|\u{85}|\u{2028}").unwrap();
        line_ends.replace_all(value, "\u{0A}").to_string()
    }
}

pub(crate) fn escape(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for c in input.chars() {
        match c {
            XML_ESC_AMP_CHAR => result.push_str(&to_entity(XML_ESC_AMP_CHAR)),
            XML_ESC_APOS_CHAR => result.push_str(&to_entity(XML_ESC_APOS_CHAR)),
            XML_ESC_GT_CHAR => result.push_str(&to_entity(XML_ESC_GT_CHAR)),
            XML_ESC_LT_CHAR => result.push_str(&to_entity(XML_ESC_LT_CHAR)),
            XML_ESC_QUOT_CHAR => result.push_str(&to_entity(XML_ESC_QUOT_CHAR)),
            o => result.push(o),
        }
    }
    result
}

pub(crate) fn to_entity(c: char) -> String {
    format!(
        "{}{}{}",
        XML_NUMBERED_ENTITYREF_START, c as u16, XML_ENTITYREF_END
    )
}

fn char_from_entity(entity: &str) -> String {
    assert!(entity.starts_with("&#"));
    assert!(entity.ends_with(';'));
    let code_point = if &entity[2..3] == "x" {
        let code_point = &entity[3..entity.len() - 1];
        u32::from_str_radix(code_point, 16).unwrap()
    } else {
        let code_point = &entity[2..entity.len() - 1];
        code_point.parse::<u32>().unwrap()
    };
    let character = char::try_from(code_point).unwrap();
    character.to_string()
}

impl Display for SpaceHandling {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}{}{}=\"{}\"",
            XML_NS_ATTRIBUTE,
            XML_NS_SEPARATOR,
            XML_NS_ATTR_SPACE,
            match self {
                SpaceHandling::Default => XML_NS_ATTR_SPACE_DEFAULT,
                SpaceHandling::Preserve => XML_NS_ATTR_SPACE_PRESERVE,
            }
        )
    }
}

impl FromStr for SpaceHandling {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == XML_NS_ATTR_SPACE_DEFAULT {
            Ok(SpaceHandling::Default)
        } else if s == XML_NS_ATTR_SPACE_PRESERVE {
            Ok(SpaceHandling::Preserve)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;
    use std::collections::HashMap;

    #[test]
    fn test_space_handling_default() {
        let sh = SpaceHandling::default();
        assert_eq!(sh, SpaceHandling::Default);
    }

    #[test]
    fn test_space_handling_display() {
        assert_eq!(
            format!("{}", SpaceHandling::Default),
            format!(
                "{}{}{}=\"{}\"",
                XML_NS_ATTRIBUTE, XML_NS_SEPARATOR, XML_NS_ATTR_SPACE, XML_NS_ATTR_SPACE_DEFAULT
            )
        );
        assert_eq!(
            format!("{}", SpaceHandling::Preserve),
            format!(
                "{}{}{}=\"{}\"",
                XML_NS_ATTRIBUTE, XML_NS_SEPARATOR, XML_NS_ATTR_SPACE, XML_NS_ATTR_SPACE_PRESERVE
            )
        );
    }

    #[test]
    fn test_space_handling_from_str() {
        assert_eq!(
            SpaceHandling::from_str(XML_NS_ATTR_SPACE_DEFAULT).unwrap(),
            SpaceHandling::Default
        );
        assert_eq!(
            SpaceHandling::from_str(XML_NS_ATTR_SPACE_PRESERVE).unwrap(),
            SpaceHandling::Preserve
        );
        assert!(SpaceHandling::from_str("").is_err());
        assert!(SpaceHandling::from_str("other").is_err());
    }

    #[test]
    fn test_end_of_line_handling() {
        let input = "one\u{0D}two\u{0D}\u{0A}\u{0A}three\u{0A}\u{0D}\u{85}four\u{85}five\u{2028}";
        let output = normalize_end_of_lines(input);
        assert_eq!(
            output,
            "one\u{0A}two\u{0A}\u{0A}three\u{0A}\u{0A}four\u{0A}five\u{0A}".to_string()
        )
    }

    struct TestResolver {
        entity_map: HashMap<String, String>,
    }

    impl EntityResolver for TestResolver {
        fn resolve(&self, entity: &str) -> Option<String> {
            self.entity_map.get(entity).cloned()
        }
    }

    impl TestResolver {
        pub(crate) fn new() -> Self {
            let mut new_self = Self {
                entity_map: Default::default(),
            };
            let _safe_to_ignore = new_self
                .entity_map
                .insert("&pound;".to_string(), "£".to_string());
            let _safe_to_ignore = new_self
                .entity_map
                .insert("&yen;".to_string(), "¥".to_string());
            let _safe_to_ignore = new_self
                .entity_map
                .insert("&euro;".to_string(), "€".to_string());
            let _safe_to_ignore = new_self.entity_map.insert(
                "&currency;".to_string(),
                "$, &pound;, &euro;, and &yen;".to_string(),
            );
            new_self
        }
    }

    fn test_resolver() -> Box<dyn EntityResolver> {
        let resolver = TestResolver::new();
        Box::new(resolver)
    }

    #[test]
    fn test_normalize_avalue_entity_resolver() {
        let resolver = test_resolver();
        let resolver = resolver.borrow();
        assert_eq!(
            normalize_attribute_value("10$ in &pound;s please", resolver, true),
            "10$ in £s please"
        );
        assert_eq!(
            normalize_attribute_value("&yen; to &euro;", resolver, false),
            "¥ to €"
        );
        assert_eq!(
            normalize_attribute_value("&currency;", resolver, false),
            "$, £, €, and ¥"
        );
    }
}
