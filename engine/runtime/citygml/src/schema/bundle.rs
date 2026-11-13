//! The schema documents compiled into the crate by `build.rs`.

/// One embedded document: its path under `schemas/`, and its bytes.
struct EmbeddedSchema {
    key: &'static str,
    bytes: &'static [u8],
}

// Defines EMBEDDED_SCHEMAS from the rows of `schemas/MANIFEST.tsv`.
include!(concat!(env!("OUT_DIR"), "/embedded_schemas.rs"));

/// The embedded copy of the schema at `key`, a path under `schemas/` such as
/// `schemas.opengis.net/citygml/building/2.0/building.xsd`.
pub(crate) fn get(key: &str) -> Option<&'static [u8]> {
    EMBEDDED_SCHEMAS
        .iter()
        .find(|schema| schema.key == key)
        .map(|schema| schema.bytes)
}

/// Every embedded key, in manifest order.
pub(crate) fn keys() -> impl Iterator<Item = &'static str> {
    EMBEDDED_SCHEMAS.iter().map(|schema| schema.key)
}

/// The bundle key a schema URL would have: the URL without its scheme. `None`
/// for a location that is not an `http` or `https` URL.
pub(crate) fn key_for_url(url: &str) -> Option<String> {
    url.strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILDING: &str = "schemas.opengis.net/citygml/building/2.0/building.xsd";

    #[test]
    fn a_bundled_schema_is_found_by_its_key() {
        let bytes = get(BUILDING).expect("building.xsd is bundled");
        assert!(std::str::from_utf8(bytes).unwrap().contains("BuildingType"));
    }

    #[test]
    fn city_object_group_is_bundled_because_i_ur_imports_it() {
        assert!(
            get("schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd").is_some()
        );
    }

    #[test]
    fn an_unknown_key_is_none() {
        assert!(get("schemas.opengis.net/citygml/none/2.0/none.xsd").is_none());
    }

    #[test]
    fn http_and_https_urls_map_to_the_same_key() {
        let url = "http://schemas.opengis.net/citygml/building/2.0/building.xsd";
        assert_eq!(key_for_url(url).as_deref(), Some(BUILDING));
        assert_eq!(
            key_for_url(&url.replacen("http", "https", 1)).as_deref(),
            Some(BUILDING)
        );
        assert_eq!(key_for_url("../building.xsd"), None);
    }

    #[test]
    fn keys_list_every_manifest_row() {
        assert_eq!(keys().count(), 49);
    }
}
