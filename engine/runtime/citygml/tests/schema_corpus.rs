//! Places every child of every city object in every CityGML 2.0 document under
//! `engine/testing/data` with the runtime schema model. The expected exceptions
//! are listed; the check fails both on a new failure and on a listed document
//! that has started passing, like `tools/citygml-schema/known-invalid.txt`.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use reearth_flow_citygml::schema::{Placement, QName, SchemaSet};

const CORE: &str = "http://www.opengis.net/citygml/2.0";

/// Documents that must fail, relative to `engine/testing/data`, with a
/// substring of the error they must fail with, and why.
const EXPECTED_FAILURES: &[(&str, &str, &str)] = &[
    ("testcases/quality-check/plateau4/01-01-common/L01/udx/bldg/54377074_bldg_6697_op.gml",
     "xml: ill-formed document",
     "malformed XML on purpose: the L01 case is an unclosed tag"),
    ("testcases/quality-check/plateau4/02-bldg/L13_LOD0_01/udx/bldg/L13_LOD0_01.gml",
     "Building: unplaced bldgDataQualityAttribute",
     "declares uro 3.0 but uses bldgDataQualityAttribute, which uro 3.0 calls buildingDataQualityAttribute"),
    ("testcases/quality-check/plateau4/06-fld/Z-fld-01_invalid-vertex-count_02/udx/fld/pref/river_a/49300100_fld_6697_l2_op.gml",
     "WaterBody: unplaced lod2MultiSurface",
     "wtr:WaterBody has no lod2MultiSurface in CityGML 2.0; xmllint rejects it too"),
    ("testcases/data-convert/plateau4/09-unf/unf/13999_tokyo_udx-mlit_2024_citygml_2_sample-takeshiba_op_unf/udx/unf/53393680_unf_6697_op.gml",
     "WaterPipe: out of order frnDataQualityAttribute",
     "places frnDataQualityAttribute, a CityFurniture hook property, after WaterPipe's own properties"),
];

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testing/data")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// CityGML 2.0 documents, excluding the writer's own golden files.
fn documents() -> Vec<PathBuf> {
    let mut all = Vec::new();
    walk(&data(), &mut all);
    all.retain(|p| {
        p.extension().is_some_and(|e| e == "gml")
            && !p.to_string_lossy().contains("citygml_writer")
            && std::fs::read(p).is_ok_and(|b| {
                String::from_utf8_lossy(&b[..b.len().min(4000)]).contains("opengis.net/citygml/2.0")
            })
    });
    all.sort();
    all
}

/// i-UR schemas by namespace, from the fixtures' `schemas/iur` copies, for
/// documents whose own `schemas/` folder was not kept.
fn fallback() -> HashMap<String, PathBuf> {
    let mut files = Vec::new();
    walk(&data().join("fixtures"), &mut files);
    // First wins below, so the walk order must not depend on the filesystem.
    files.sort();
    let mut out = HashMap::new();
    for path in files.into_iter().filter(|p| {
        p.extension().is_some_and(|e| e == "xsd") && p.to_string_lossy().contains("/schemas/iur/")
    }) {
        let head = String::from_utf8_lossy(&std::fs::read(&path).unwrap()[..])
            .chars()
            .take(4000)
            .collect::<String>();
        if let Some(rest) = head.split("targetNamespace=\"").nth(1) {
            let namespace = rest.split('"').next().unwrap().to_owned();
            out.entry(namespace).or_insert(path);
        }
    }
    out
}

struct Document {
    schema_location: Vec<String>,
    objects: Vec<(QName, Vec<QName>)>,
}

fn qname(resolved: &ResolveResult<'_>, local: &[u8]) -> QName {
    let namespace = match resolved {
        ResolveResult::Bound(ns) => String::from_utf8_lossy(ns.as_ref()).into_owned(),
        _ => String::new(),
    };
    QName::new(namespace, String::from_utf8_lossy(local).into_owned())
}

fn read(path: &Path) -> Result<Document, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut reader = NsReader::from_reader(bytes.as_slice());
    let mut buf = Vec::new();
    let mut doc = Document {
        schema_location: Vec::new(),
        objects: Vec::new(),
    };
    let mut depth = 0usize;
    let mut member: Option<usize> = None;
    loop {
        let (resolved, event) = reader
            .read_resolved_event_into(&mut buf)
            .map_err(|e| format!("xml: {e}"))?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let name = qname(&resolved, e.local_name().as_ref());
                if depth == 0 {
                    if let Ok(Some(attr)) = e.try_get_attribute("xsi:schemaLocation") {
                        doc.schema_location = attr
                            .unescape_value()
                            .map_err(|e| e.to_string())?
                            .split_whitespace()
                            .map(str::to_owned)
                            .collect();
                    }
                }
                match member {
                    Some(m) if depth == m + 1 => doc.objects.push((name, Vec::new())),
                    Some(m) if depth == m + 2 => {
                        doc.objects.last_mut().expect("object opened").1.push(name)
                    }
                    Some(_) => {}
                    None if name == QName::new(CORE, "cityObjectMember") => member = Some(depth),
                    None => {}
                }
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                }
            }
            Event::End(_) => {
                depth -= 1;
                if member == Some(depth) {
                    member = None;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(doc)
}

fn check(
    path: &Path,
    fallback: &HashMap<String, PathBuf>,
    cache: &mut HashMap<Vec<PathBuf>, SchemaSet>,
) -> Result<(), String> {
    let doc = read(path)?;
    let mut ade: Vec<PathBuf> = doc
        .schema_location
        .chunks(2)
        .filter_map(|pair| match pair {
            [namespace, location] if !location.starts_with("http") => {
                let local = path.parent().unwrap().join(location);
                if local.exists() {
                    local.canonicalize().ok()
                } else {
                    fallback.get(namespace).cloned()
                }
            }
            _ => None,
        })
        .collect();
    ade.sort();
    ade.dedup();
    if !cache.contains_key(&ade) {
        let set = SchemaSet::with_ade(&ade).map_err(|e| format!("schemas: {e}"))?;
        cache.insert(ade.clone(), set);
    }
    let set = &cache[&ade];
    for (object, children) in &doc.objects {
        let class = set
            .class_for_element(object)
            .ok_or_else(|| format!("{}: unknown type", object.local))?;
        let mut last = None;
        for child in children {
            match class.place(child, last) {
                Placement::Slot(index) => last = Some(index),
                Placement::OutOfOrder(_) => {
                    return Err(format!("{}: out of order {}", object.local, child.local))
                }
                Placement::Unknown => {
                    return Err(format!("{}: unplaced {}", object.local, child.local))
                }
            }
        }
    }
    Ok(())
}

#[test]
fn every_citygml_2_document_is_placed_or_expected_to_fail() {
    let root = data();
    let fallback = fallback();
    let mut cache = HashMap::new();
    let docs = documents();
    assert!(
        docs.len() >= 190,
        "found only {} CityGML 2.0 documents",
        docs.len()
    );

    let mut outcome: BTreeMap<String, Result<(), String>> = BTreeMap::new();
    for doc in &docs {
        let key = doc
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        outcome.insert(key, check(doc, &fallback, &mut cache));
    }
    let expected: BTreeMap<&str, &str> = EXPECTED_FAILURES
        .iter()
        .map(|(path, substring, _)| (*path, *substring))
        .collect();
    let unexpected: Vec<String> = outcome
        .iter()
        .filter(|(key, result)| result.is_err() && !expected.contains_key(key.as_str()))
        .map(|(key, result)| format!("{key}: {}", result.as_ref().unwrap_err()))
        .collect();
    let wrong_reason: Vec<String> = expected
        .iter()
        .filter_map(|(key, substring)| match outcome.get(*key) {
            Some(Err(message)) if !message.contains(substring) => {
                Some(format!("{key}: expected `{substring}`, got `{message}`"))
            }
            _ => None,
        })
        .collect();
    let now_passing: Vec<&str> = expected
        .keys()
        .filter(|key| outcome.get(**key).is_some_and(Result::is_ok))
        .copied()
        .collect();
    let missing: Vec<&str> = expected
        .keys()
        .filter(|key| !outcome.contains_key(**key))
        .copied()
        .collect();
    assert!(
        unexpected.is_empty()
            && wrong_reason.is_empty()
            && now_passing.is_empty()
            && missing.is_empty(),
        "unexpected failures:\n  {}\nlisted but failing for a different reason:\n  {}\nlisted but now passing:\n  {}\nlisted but not found:\n  {}",
        unexpected.join("\n  "),
        wrong_reason.join("\n  "),
        now_passing.join("\n  "),
        missing.join("\n  ")
    );
}
