//! Per-tile `EXT_structural_metadata` property table: one implicit `Feature`
//! class per glb, its properties the union of every attribute path present
//! across that glb's features.
//!
//! 3D Tiles has no array/map property type (`EXT_structural_metadata`'s
//! `array` flag only covers a fixed/variable-length list of one SCALAR/VEC/
//! STRING element, not arbitrary nesting), so a `Map`/`Array`-valued
//! attribute — routine in PLATEAU's `uro:`/`bldg:` extension attributes,
//! e.g. `bldg:measuredHeight` = `{"@uom": "m", "$": "12.2"}` — is walked
//! recursively and each scalar leaf becomes its own STRING property, named
//! by its path (parent and child keys joined with `_`), rather than one
//! property holding the whole structure as a JSON string.
//!
//! §6.2.3 of the geometry design doc plans per-feature-type classing (keyed
//! by `schemaKey`) and a schema shared across every glb via `schema.json`;
//! this pass has neither — one class, inlined per glb, string-typed
//! properties only. `schemaKey` and `skipUnexposedAttributes` are still
//! honored (reusing the writer's existing params) since both are simple
//! exclusions, not part of the classing/hoisting this pass defers.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use reearth_flow_types::{AttributeValue, Feature};

#[derive(Debug, Clone, Copy, Default)]
pub struct MetadataOptions<'a> {
    pub schema_key: Option<&'a str>,
    pub skip_unexposed_attributes: bool,
}

/// `properties[i] = (raw attribute path, glTF-identifier-safe property id)`;
/// `rows[feature][i]` is that feature's value for column `i` (`""` if the
/// feature doesn't carry that path).
pub(super) struct PropertyTable {
    pub(super) properties: Vec<(String, String)>,
    pub(super) rows: Vec<Vec<String>>,
}

pub(super) fn build_table(features: &[&Feature], options: MetadataOptions) -> PropertyTable {
    let flattened: Vec<BTreeMap<String, String>> = features
        .iter()
        .map(|feature| flatten_attributes(feature, options))
        .collect();

    let mut raw_paths = BTreeSet::new();
    for f in &flattened {
        raw_paths.extend(f.keys().cloned());
    }

    let mut used_ids = HashSet::new();
    let properties: Vec<(String, String)> = raw_paths
        .into_iter()
        .map(|raw| {
            let id = sanitize_identifier(&raw, &mut used_ids);
            (raw, id)
        })
        .collect();

    let rows = flattened
        .iter()
        .map(|f| {
            properties
                .iter()
                .map(|(raw, _)| f.get(raw).cloned().unwrap_or_default())
                .collect()
        })
        .collect();

    PropertyTable { properties, rows }
}

fn flatten_attributes(feature: &Feature, options: MetadataOptions) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (key, value) in feature.attributes.iter() {
        let key = key.inner();
        if is_excluded(&key, options) {
            continue;
        }
        flatten(key, value, &mut out);
    }
    out
}

/// Walks `value`, inserting one `path -> stringified leaf` entry per scalar
/// reached; a `Map`/`Array` contributes no entry of its own, only its
/// descendants, with `path` extended by `_<child key>` / `_<index>`.
fn flatten(path: String, value: &AttributeValue, out: &mut BTreeMap<String, String>) {
    match value {
        AttributeValue::Map(map) => {
            for (key, child) in map {
                flatten(format!("{path}_{key}"), child, out);
            }
        }
        AttributeValue::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                flatten(format!("{path}_{i}"), child, out);
            }
        }
        leaf => {
            out.insert(path, leaf.to_string());
        }
    }
}

fn is_excluded(key: &str, options: MetadataOptions) -> bool {
    (options.skip_unexposed_attributes && key.starts_with("__")) || options.schema_key == Some(key)
}

/// CityGML attribute keys are commonly namespace-prefixed (`bldg:measuredHeight`,
/// `uro:buildingIDAttribute`) and so routinely violate `EXT_structural_metadata`'s
/// identifier syntax; this maps a raw key to a valid, collision-free id, while
/// the raw key survives separately as the property's `name` for display.
fn sanitize_identifier(raw: &str, used: &mut HashSet<String>) -> String {
    let mut id: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if id.is_empty() || id.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        id.insert(0, '_');
    }
    if used.insert(id.clone()) {
        return id;
    }
    let mut n = 1;
    loop {
        let candidate = format!("{id}_{n}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}
