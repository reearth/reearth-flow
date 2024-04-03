#![allow(dead_code)]

// ------------------------------------------------------------------------------------------------
// Pure Syntactic Tokens
// ------------------------------------------------------------------------------------------------

pub(crate) const XML_DECL_START: &str = "<?xml";
pub(crate) const XML_DECL_END: &str = "?>";

pub(crate) const XML_PI_START: &str = "<?";
pub(crate) const XML_PI_END: &str = "?>";
pub(crate) const XML_PI_RESERVED: &str = "xml";

pub(crate) const XML_COMMENT_START: &str = "<!--";
pub(crate) const XML_COMMENT_END: &str = "-->";

pub(crate) const XML_CDATA_START: &str = "<![CDATA[";
pub(crate) const XML_CDATA_END: &str = "]]>";

pub(crate) const XML_DOCTYPE_START: &str = "<!DOCTYPE";
pub(crate) const XML_DOCTYPE_END: &str = ">";
pub(crate) const XML_DOCTYPE_ENTITY_START: &str = "[";
pub(crate) const XML_DOCTYPE_ENTITY_END: &str = "]";
pub(crate) const XML_DOCTYPE_PUBLIC: &str = "PUBLIC";
pub(crate) const XML_DOCTYPE_SYSTEM: &str = "SYSTEM";

pub(crate) const XML_ELEMENT_START_START: &str = "<";
pub(crate) const XML_ELEMENT_START_END: &str = ">";
pub(crate) const XML_ELEMENT_END_START: &str = "</";
pub(crate) const XML_ELEMENT_END_END: &str = ">";

pub(crate) const XML_ENTITY_START: &str = "<!ENTITY";
pub(crate) const XML_ENTITY_END: &str = ">";
pub(crate) const XML_ENTITY_NOTATION: &str = "NDATA";

pub(crate) const XML_ENTITYREF_START: &str = "&";
pub(crate) const XML_NUMBERED_ENTITYREF_START: &str = "&#";
pub(crate) const XML_HEX_NUMBERED_ENTITYREF_START: &str = "&#x";
pub(crate) const XML_ENTITYREF_END: &str = ";";

pub(crate) const XML_NOTATION_START: &str = "<!NOTATION";
pub(crate) const XML_NOTATION_END: &str = ">";

pub(crate) const XML_NS_SEPARATOR: &str = ":";

pub(crate) const XML_EMPTY: &str = "";

// ------------------------------------------------------------------------------------------------
// XML Declaration
// ------------------------------------------------------------------------------------------------

pub(crate) const XML_DECL_VERSION: &str = "version";

pub(crate) const XML_DECL_VERSION_10: &str = "1.0";
pub(crate) const XML_DECL_VERSION_11: &str = "1.1";

pub(crate) const XML_DECL_ENCODING: &str = "encoding";

pub(crate) const XML_DECL_STANDALONE: &str = "standalone";

pub(crate) const XML_DECL_STANDALONE_YES: &str = "yes";
pub(crate) const XML_DECL_STANDALONE_NO: &str = "no";

// ------------------------------------------------------------------------------------------------
// The XML language namespace Support
// ------------------------------------------------------------------------------------------------

pub(crate) const XML_NS_URI: &str = "http://www.w3.org/XML/1998/namespace";
pub(crate) const XML_NS_ATTRIBUTE: &str = "xml";

pub(crate) const XML_NS_ATTR_BASE: &str = "base";
pub(crate) const XML_NS_ATTR_ID: &str = "id";
pub(crate) const XML_NS_ATTR_LANG: &str = "lang";
pub(crate) const XML_NS_ATTR_SPACE: &str = "space";

pub(crate) const XML_NS_ATTR_SPACE_DEFAULT: &str = "default";
pub(crate) const XML_NS_ATTR_SPACE_PRESERVE: &str = "preserve";

// ------------------------------------------------------------------------------------------------
// Namespace Support
// ------------------------------------------------------------------------------------------------

pub(crate) const XMLNS_NS_URI: &str = "http://www.w3.org/2000/xmlns/";
pub(crate) const XMLNS_NS_ATTRIBUTE: &str = "xmlns";

// ------------------------------------------------------------------------------------------------
// DOM Node Names
// ------------------------------------------------------------------------------------------------

pub(crate) const XML_NAME_CDATA: &str = "#cdata-section";
pub(crate) const XML_NAME_COMMENT: &str = "#comment";
pub(crate) const XML_NAME_DOCUMENT: &str = "#document";
pub(crate) const XML_NAME_DOCUMENT_FRAGMENT: &str = "#document-fragment";
pub(crate) const XML_NAME_TEXT: &str = "#text";

// ------------------------------------------------------------------------------------------------
// DOM Features
// ------------------------------------------------------------------------------------------------

pub(crate) const XML_FEATURE_CORE: &str = "Core"; // DOM Level-2 "Fundamental Interfaces"
pub(crate) const XML_FEATURE_XML: &str = "XML"; // DOM Level-2 "Extended Interfaces"

pub(crate) const XML_FEATURE_V1: &str = "1.0";
pub(crate) const XML_FEATURE_V2: &str = "2.0";

// ------------------------------------------------------------------------------------------------
// Pre-Defined Reserved Characters
// ------------------------------------------------------------------------------------------------

pub(crate) const XML_ESC_AMP_CHAR: char = '&';
pub(crate) const XML_ESC_APOS_CHAR: char = '\'';
pub(crate) const XML_ESC_GT_CHAR: char = '>';
pub(crate) const XML_ESC_LT_CHAR: char = '<';
pub(crate) const XML_ESC_QUOT_CHAR: char = '"';
