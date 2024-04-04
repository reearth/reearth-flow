use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::result::Result as StdResult;
use std::str::{from_utf8, FromStr};

use crate::error::Error;
use crate::syntax::*;
use crate::Result;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub(crate) namespace_uri: Option<String>,
    pub(crate) prefix: Option<String>,
    pub(crate) local_name: String,
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.prefix {
            Some(prefix) => write!(f, "{}{}{}", prefix, XML_NS_SEPARATOR, self.local_name),
            None => write!(f, "{}", self.local_name),
        }
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Name {{ namespace_uri: {:?}, prefix: {:?}, local_name: {} }}",
            self.namespace_uri, self.prefix, self.local_name
        )
    }
}

impl FromStr for Name {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        if value.is_empty() {
            Err(Error::Syntax("Name may not be empty".to_string()))
        } else {
            let parts = value
                .split(XML_NS_SEPARATOR)
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            match parts.len() {
                1 => Name::new(Name::check_part(parts.first().unwrap())?, None, None),
                2 => Name::new(
                    Name::check_part(parts.get(1).unwrap())?,
                    Some(Name::check_part(parts.first().unwrap())?),
                    None,
                ),
                _ => Err(Error::Syntax("Too many parts".to_string())),
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------

impl TryFrom<&[u8]> for Name {
    type Error = Error;

    fn try_from(value: &[u8]) -> StdResult<Self, Self::Error> {
        match from_utf8(value) {
            Ok(str) => Self::from_str(str),
            Err(e) => Err(Error::InvalidCharacter(format!(
                "Could not convert from UTF-8, error {:?}",
                e
            ))),
        }
    }
}

// ------------------------------------------------------------------------------------------------

impl Name {
    pub fn new_ns(namespace_uri: &str, qualified_name: &str) -> Result<Self> {
        let mut parsed = Name::from_str(qualified_name)?;
        parsed.namespace_uri = Some(Self::check_namespace_uri(
            namespace_uri,
            &parsed.prefix,
            &parsed.local_name,
        )?);
        Ok(parsed)
    }

    fn new(
        local_name: String,
        prefix: Option<String>,
        namespace_uri: Option<String>,
    ) -> Result<Self> {
        if local_name.is_empty() {
            return Err(Error::Syntax("local_name may not be empty".to_string()));
        }
        if let Some(prefix) = &prefix {
            if prefix.is_empty() {
                return Err(Error::Syntax("prefix may not be empty".to_string()));
            }
        }
        if let Some(namespace_uri) = &namespace_uri {
            if namespace_uri.is_empty() {
                return Err(Error::Syntax("namespace_uri may not be empty".to_string()));
            }
        }
        Ok(Self {
            namespace_uri,
            prefix,
            local_name,
        })
    }

    fn check_part(part: &str) -> Result<String> {
        if part.is_empty() {
            Err(Error::Syntax("Part may not be empty".to_string()))
        } else {
            Ok(part.to_string())
        }
    }

    fn check_namespace_uri(
        namespace_uri: &str,
        _prefix: &Option<String>,
        _local: &str,
    ) -> Result<String> {
        if namespace_uri.is_empty() {
            Err(Error::Syntax("Namespace URI may not be empty".to_string()))
        } else {
            Ok(namespace_uri.to_string())
        }
    }

    pub fn for_cdata() -> Self {
        Self {
            namespace_uri: None,
            prefix: None,
            local_name: XML_NAME_CDATA.to_string(),
        }
    }

    pub fn for_comment() -> Self {
        Self {
            namespace_uri: None,
            prefix: None,
            local_name: XML_NAME_COMMENT.to_string(),
        }
    }

    ///
    /// Return the reserved name for `Document` nodes
    ///
    pub fn for_document() -> Self {
        Self {
            namespace_uri: None,
            prefix: None,
            local_name: XML_NAME_DOCUMENT.to_string(),
        }
    }

    pub fn for_document_fragment() -> Self {
        Self {
            namespace_uri: None,
            prefix: None,
            local_name: XML_NAME_DOCUMENT_FRAGMENT.to_string(),
        }
    }

    pub fn for_text() -> Self {
        Self {
            namespace_uri: None,
            prefix: None,
            local_name: XML_NAME_TEXT.to_string(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn for_null() -> Self {
        Self {
            namespace_uri: None,
            prefix: None,
            local_name: "null".to_string(),
        }
    }

    pub fn is_namespace_attribute(&self) -> bool {
        let xmlns_attribute = XMLNS_NS_ATTRIBUTE.to_string();
        (self.local_name == xmlns_attribute && self.prefix.is_none())
            || self.prefix == Some(xmlns_attribute)
    }

    pub fn is_id_attribute(&self, lax: bool) -> bool {
        let id_attribute = XML_NS_ATTR_ID.to_string();
        if lax {
            self.local_name == id_attribute
        } else {
            let xml_ns = XML_NS_URI.to_string();
            let xml_prefix = XML_NS_ATTRIBUTE.to_string();
            //
            // has to be 'xml:id', either by the prefix 'xml' or using the correct namespace
            self.local_name == id_attribute
                && (self.namespace_uri == Some(xml_ns) || self.prefix == Some(xml_prefix))
        }
    }

    pub fn namespace_uri(&self) -> &Option<String> {
        &self.namespace_uri
    }

    pub fn local_name(&self) -> &String {
        &self.local_name
    }

    pub fn prefix(&self) -> &Option<String> {
        &self.prefix
    }
}

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local() {
        let name = Name::from_str("hello").unwrap();
        assert_eq!(name.local_name, "hello".to_string());
        assert!(name.prefix().is_none());
        assert!(name.namespace_uri().is_none());
    }

    #[test]
    fn test_parse_qualified() {
        let name = Name::from_str("x:hello").unwrap();
        assert_eq!(name.local_name, "hello".to_string());
        assert_eq!(name.prefix(), &Some("x".to_string()));
        assert!(name.namespace_uri().is_none());
    }

    #[test]
    fn test_parse_namespaced() {
        let name = Name::new_ns("http://example.org/schema/x", "x:hello").unwrap();
        assert_eq!(name.local_name, "hello".to_string());
        assert_eq!(name.prefix(), &Some("x".to_string()));
        assert_eq!(
            name.namespace_uri(),
            &Some("http://example.org/schema/x".to_string())
        );
    }

    #[test]
    fn test_error_on_empty() {
        let name = Name::from_str("");
        assert!(name.is_err());

        let name = Name::from_str(":name");
        assert!(name.is_err());

        let name = Name::from_str("prefix:");
        assert!(name.is_err());

        let name = Name::new_ns("", "prefix:name");
        assert!(name.is_err());
    }

    #[test]
    fn test_xml_ns_names() {
        let name = Name::new_ns(XML_NS_URI, "xml:id");
        assert!(name.is_ok());
        let name = name.unwrap();
        assert!(name.is_id_attribute(true));
        assert!(name.is_id_attribute(false));

        let name = Name::from_str("another:id");
        assert!(name.is_ok());
        let name = name.unwrap();
        assert!(name.is_id_attribute(true));
        assert!(!name.is_id_attribute(false));

        let name = Name::from_str("x:hello");
        assert!(name.is_ok());
        let name = name.unwrap();
        assert!(!name.is_id_attribute(true));
        assert!(!name.is_id_attribute(false));
    }

    #[test]
    fn test_xmlns_ns_names() {
        let name = Name::new_ns(XMLNS_NS_URI, "xmlns");
        assert!(name.is_ok());
        assert!(name.unwrap().is_namespace_attribute());

        let name = Name::new_ns(XMLNS_NS_URI, "xmlns:p");
        assert!(name.is_ok());
        assert!(name.unwrap().is_namespace_attribute());

        let name = Name::from_str("x:hello").unwrap();
        assert!(!name.is_namespace_attribute());
    }
}
