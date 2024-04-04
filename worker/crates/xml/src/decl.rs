use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::syntax::*;
use crate::Result;

///
/// Captures the supported version of the XML specification itself, as used in `XmlDecl`.
///
#[derive(Clone, Debug, PartialEq)]
pub enum XmlVersion {
    /// Version 1.0 [https://www.w3.org/TR/xml]
    V10,
    /// Version 1.1 [https://www.w3.org/TR/xml11]
    V11,
}

#[derive(Clone, Debug)]
pub struct XmlDecl {
    version: XmlVersion,
    encoding: Option<String>,
    standalone: Option<bool>,
}

pub(crate) const ENCODING_SEP_CHAR: char = '-';

fn is_encoding_start_char(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_lowercase()
}

fn is_encoding_rest_char(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_'
}

fn is_encoding_sub_string(s: &str) -> bool {
    s.chars().all(is_encoding_rest_char)
}

fn is_encoding(s: &str) -> bool {
    !s.is_empty()
        && s.starts_with(is_encoding_start_char)
        && s[1..].split(ENCODING_SEP_CHAR).all(is_encoding_sub_string)
}

impl Display for XmlVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                XmlVersion::V10 => XML_DECL_VERSION_10,
                XmlVersion::V11 => XML_DECL_VERSION_11,
            }
        )
    }
}

impl FromStr for XmlVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == XML_DECL_VERSION_10 {
            Ok(XmlVersion::V10)
        } else if s == XML_DECL_VERSION_11 {
            Ok(XmlVersion::V11)
        } else {
            Err(())
        }
    }
}

impl Default for XmlDecl {
    fn default() -> Self {
        Self {
            version: XmlVersion::V10,
            encoding: None,
            standalone: None,
        }
    }
}

impl Display for XmlDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}=\"{}\"",
            XML_DECL_START, XML_DECL_VERSION, self.version
        )?;
        if let Some(encoding) = &self.encoding {
            write!(f, " {}=\"{}\"", XML_DECL_ENCODING, encoding)?;
        }
        if let Some(standalone) = &self.standalone {
            write!(
                f,
                " {}=\"{}\"",
                XML_DECL_STANDALONE,
                if *standalone {
                    XML_DECL_STANDALONE_YES
                } else {
                    XML_DECL_STANDALONE_NO
                }
            )?;
        }
        write!(f, "{}", XML_DECL_END)
    }
}

#[allow(dead_code)]
impl XmlDecl {
    pub fn new(version: XmlVersion, encoding: Option<String>, standalone: Option<bool>) -> Self {
        if let Some(encoding) = &encoding {
            if !is_encoding(encoding) {
                panic!("XML encoding declaration value is not valid");
            }
        }
        Self {
            version,
            encoding,
            standalone,
        }
    }

    pub fn version(&self) -> XmlVersion {
        self.version.clone()
    }

    pub fn encoding(&self) -> Option<String> {
        self.encoding.clone()
    }

    pub fn standalone(&self) -> Option<bool> {
        self.standalone
    }
}
