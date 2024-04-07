use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::{BytesCData, BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;

use crate::convert::as_document_mut;
use crate::decl::{XmlDecl, XmlVersion};
use crate::dom_impl::get_implementation;
use crate::error::Error;
use crate::node::{Extension, RefNode};
use crate::traits::{Document, Element, Node};
use crate::Result;

pub fn read_xml(xml: &str) -> Result<RefNode> {
    inner_read(&mut Reader::from_str(xml))
}

pub fn read_reader<B: BufRead>(reader: B) -> Result<RefNode> {
    inner_read(&mut Reader::from_reader(reader))
}

impl<T> From<Error> for Result<T> {
    fn from(val: Error) -> Self {
        Err(val)
    }
}

fn inner_read<T: BufRead>(reader: &mut Reader<T>) -> Result<RefNode> {
    let _safe_to_ignore = reader.trim_text(true);

    let mut event_buffer: Vec<u8> = Vec::new();

    document(reader, &mut event_buffer)
}

fn document<T: BufRead>(reader: &mut Reader<T>, event_buffer: &mut Vec<u8>) -> Result<RefNode> {
    let mut document = get_implementation()
        .create_document(None, None, None)
        .unwrap();

    loop {
        match reader.read_event_into(event_buffer) {
            Ok(Event::Decl(ev)) => {
                let mut mut_document = document.borrow_mut();
                if let Extension::Document {
                    xml_declaration, ..
                } = &mut mut_document.extension
                {
                    if xml_declaration.is_some() {
                        return Err(Error::Malformed("multiple xml declarations".to_string()));
                    } else {
                        let (version, encoding, standalone) = make_decl(ev)?;
                        *xml_declaration = Some(XmlDecl::new(
                            XmlVersion::from_str(&version).unwrap(),
                            encoding,
                            standalone,
                        ));
                    }
                }
            }
            Ok(Event::Start(ev)) => {
                let mut new_element = handle_start(reader, &mut document, None, ev)?;
                let _safe_to_ignore =
                    element(reader, event_buffer, &mut document, &mut new_element);
            }
            Ok(Event::Empty(ev)) => {
                let _safe_to_ignore = handle_start(reader, &mut document, None, ev)?;
            }
            Ok(Event::End(ev)) => {
                let _safe_to_ignore = handle_end(reader, &mut document, None, ev)?;
            }
            Ok(Event::Comment(ev)) => {
                let _safe_to_ignore = handle_comment(reader, &mut document, None, ev)?;
            }
            Ok(Event::PI(ev)) => {
                let _safe_to_ignore = handle_pi(reader, &mut document, None, ev)?;
            }
            Ok(Event::Text(ev)) => {
                let _safe_to_ignore = handle_text(reader, &mut document, None, ev)?;
            }
            Ok(Event::CData(ev)) => {
                let _safe_to_ignore = handle_cdata(reader, &mut document, None, ev)?;
            }
            Ok(Event::Eof) => return Ok(document),
            Ok(e) => {
                return Error::Malformed(format!("parse document with {:?}", e)).into();
            }
            Err(err) => {
                return Error::from(err).into();
            }
        }
    }
}

fn element<T: BufRead>(
    reader: &mut Reader<T>,
    event_buffer: &mut Vec<u8>,
    document: &mut RefNode,
    parent_element: &mut RefNode,
) -> Result<RefNode> {
    loop {
        match reader.read_event_into(event_buffer) {
            Ok(Event::Start(ev)) => {
                let mut new_element = handle_start(reader, document, Some(parent_element), ev)?;
                let _safe_to_ignore = element(reader, event_buffer, document, &mut new_element)?;
            }
            Ok(Event::Empty(ev)) => {
                let _safe_to_ignore = handle_start(reader, document, Some(parent_element), ev)?;
            }
            Ok(Event::End(ev)) => {
                let _safe_to_ignore = handle_end(reader, document, Some(parent_element), ev)?;
                return Ok(parent_element.clone());
            }
            Ok(Event::Comment(ev)) => {
                let _safe_to_ignore = handle_comment(reader, document, Some(parent_element), ev)?;
            }
            Ok(Event::PI(ev)) => {
                let _safe_to_ignore = handle_pi(reader, document, Some(parent_element), ev)?;
            }
            Ok(Event::Text(ev)) => {
                let _safe_to_ignore = handle_text(reader, document, Some(parent_element), ev)?;
            }
            Ok(Event::CData(ev)) => {
                let _safe_to_ignore = handle_cdata(reader, document, Some(parent_element), ev)?;
            }
            Ok(e) => {
                return Error::Malformed(format!("parse element with {:?}", e)).into();
            }
            Err(err) => {
                return Error::from(err).into();
            }
        }
    }
}

fn handle_start<T: BufRead>(
    _reader: &mut Reader<T>,
    document: &mut RefNode,
    parent_node: Option<&mut RefNode>,
    ev: BytesStart<'_>,
) -> Result<RefNode> {
    let mut element = {
        let mut_document = as_document_mut(document).unwrap();
        let name = String::from_utf8(ev.name().into_inner().to_vec())
            .map_err(|e| Error::Malformed(format!("parse version : {:?}", e)))?;
        let new_node = mut_document.create_element(&name).unwrap();
        let mut actual_parent = match parent_node {
            None => document.clone(),
            Some(actual) => actual.clone(),
        };
        actual_parent.append_child(new_node)?
    };

    for attribute in ev.attributes() {
        let attribute = attribute.unwrap();
        let value = attribute.unescape_value()?;
        let name = std::str::from_utf8(attribute.key.into_inner())
            .map_err(|e| Error::Malformed(format!("parse attribute key : {:?}", e)))?;
        let attribute_node = document.create_attribute_with(name, &value)?;
        let _safe_to_ignore = element.set_attribute_node(attribute_node)?;
    }
    Ok(element)
}

fn handle_end<T: BufRead>(
    _reader: &mut Reader<T>,
    document: &mut RefNode,
    parent_node: Option<&mut RefNode>,
    _ev: BytesEnd<'_>,
) -> Result<RefNode> {
    Ok(match parent_node {
        None => document,
        Some(actual) => actual,
    }
    .clone())
}

fn handle_comment<T: BufRead>(
    reader: &mut Reader<T>,
    document: &mut RefNode,
    parent_node: Option<&mut RefNode>,
    ev: BytesText<'_>,
) -> Result<RefNode> {
    let mut_document = as_document_mut(document).unwrap();
    let text = make_text(reader, ev)?;
    let new_node = mut_document.create_comment(&text);
    let actual_parent = match parent_node {
        None => document,
        Some(actual) => actual,
    };
    actual_parent.append_child(new_node)
}

fn handle_text<T: BufRead>(
    reader: &mut Reader<T>,
    document: &mut RefNode,
    parent_node: Option<&mut RefNode>,
    ev: BytesText<'_>,
) -> Result<RefNode> {
    let mut_document = as_document_mut(document).unwrap();
    let text = make_text(reader, ev)?;
    let new_node = mut_document.create_text_node(&text);
    let actual_parent = match parent_node {
        None => document,
        Some(actual) => actual,
    };
    actual_parent.append_child(new_node)
}

fn handle_cdata<T: BufRead>(
    _reader: &mut Reader<T>,
    document: &mut RefNode,
    parent_node: Option<&mut RefNode>,
    ev: BytesCData<'_>,
) -> Result<RefNode> {
    let mut_document = as_document_mut(document).unwrap();
    let text = make_cdata(ev)?;
    let new_node = mut_document.create_cdata_section(text.as_ref()).unwrap();
    let actual_parent = match parent_node {
        None => document,
        Some(actual) => actual,
    };
    actual_parent.append_child(new_node)
}

fn handle_pi<T: BufRead>(
    _reader: &mut Reader<T>,
    document: &mut RefNode,
    parent_node: Option<&mut RefNode>,
    ev: BytesText<'_>,
) -> Result<RefNode> {
    let mut_document = as_document_mut(document).unwrap();
    let (target, data) = {
        let text = ev.unescape()?;
        let parts = text.splitn(2, ' ').collect::<Vec<&str>>();
        match parts.len() {
            1 => (parts[0].to_string(), None),
            2 => {
                let data = parts[1].trim();
                if data.is_empty() {
                    (parts[0].to_string(), None)
                } else {
                    (parts[0].to_string(), Some(data.to_string()))
                }
            }
            _ => return Error::Malformed("handle pi".to_string()).into(),
        }
    };
    let new_node = match data {
        None => mut_document
            .create_processing_instruction(&target, None)
            .unwrap(),
        Some(s) => mut_document
            .create_processing_instruction(&target, Some(s.as_str()))
            .unwrap(),
    };
    let actual_parent = match parent_node {
        None => document,
        Some(actual) => actual,
    };
    actual_parent.append_child(new_node)
}

// ------------------------------------------------------------------------------------------------

fn make_text<T: BufRead>(_reader: &mut Reader<T>, ev: BytesText<'_>) -> Result<String> {
    let result = ev.unescape()?;
    Ok(result.into_owned())
}

fn make_cdata(ev: BytesCData<'_>) -> Result<String> {
    let cdata_bytes = ev.into_inner();
    let decoded_string = String::from_utf8(cdata_bytes.to_vec())
        .map_err(|e| Error::Malformed(format!("parse cdata : {:?}", e)))?;
    Ok(decoded_string.to_string())
}

fn make_decl(ev: BytesDecl<'_>) -> Result<(String, Option<String>, Option<bool>)> {
    let version = ev.version()?;
    let version = String::from_utf8(version.to_vec())
        .map_err(|e| Error::Malformed(format!("parse version : {:?}", e)))?;
    let version = unquote(version.to_string())?;
    let encoding = if let Some(Ok(ev_value)) = ev.encoding() {
        let encoding = String::from_utf8(ev_value.to_vec())
            .map_err(|e| Error::Malformed(format!("parse encoding : {:?}", e)))?;
        Some(encoding.to_string())
    } else {
        None
    };
    let standalone = if let Some(Ok(ev_value)) = ev.standalone() {
        let standalone = String::from_utf8(ev_value.to_vec())
            .map_err(|e| Error::Malformed(format!("parse standalone : {:?}", e)))?;
        Some(standalone == "yes")
    } else {
        None
    };
    Ok((version, encoding, standalone))
}

#[allow(clippy::if_same_then_else)]
fn unquote(s: String) -> Result<String> {
    if s.starts_with('"') && s.ends_with('"') {
        Ok(s[1..s.len() - 1].to_string())
    } else if s.starts_with('\'') && s.ends_with('\'') {
        Ok(s[1..s.len() - 1].to_string())
    } else if s.starts_with('"') || s.starts_with('\'') {
        Err(Error::InvalidCharacter("unmatched quote".to_string()))
    } else {
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_good_xml(xml: &str) {
        let dom = read_xml(xml);
        assert!(dom.is_ok());
        let document = dom.unwrap();
        let root = document.first_child().unwrap();
        println!("root: {:?}", root.node_name());
    }

    #[test]
    fn test_shortest_document() {
        test_good_xml("<xml></xml>");
    }

    #[test]
    fn test_shortish_document() {
        test_good_xml("<?xml version=\"1.0\"?><xml></xml>");
    }

    #[test]
    fn test_commented_document() {
        test_good_xml("<!-- start here --><xml></xml><!-- end here -->");
    }

    #[test]
    fn test_commented_element() {
        test_good_xml("<xml><!--  inside --></xml>");
    }

    #[test]
    fn test_pi() {
        test_good_xml("<?xml-stylesheet type=\"text/xsl\" href=\"style.xsl\"?><xml></xml>");
    }

    #[test]
    fn test_nested_document() {
        test_good_xml("<xml><xslt></xslt></xml>");
    }

    #[test]
    fn test_attributes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <gml:Dictionary xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:gml="http://www.opengis.net/gml" gml:id="Agreement_class">
            <gml:name>Agreement_class</gml:name>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id1">
                    <gml:description>building agreement</gml:description>
                    <gml:name>1010</gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id2">
                    <gml:description>green space agreement</gml:description>
                    <gml:name>1020</gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id3">
                    <gml:description>landscape agreement</gml:description>
                    <gml:name>1030</gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id4">
                    <gml:description>development permit</gml:description>
                    <gml:name>1040</gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
        </gml:Dictionary>
                "#;
        test_good_xml(xml);
    }
}
