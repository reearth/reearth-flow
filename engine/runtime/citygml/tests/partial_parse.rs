use std::collections::{HashMap, HashSet};

use reearth_flow_citygml::parser::{CityGmlVersion, Parser};
use reearth_flow_citygml::pipeline::build_features;
use url::Url;

/// A document whose first city object is complete and whose second closes an
/// element with the wrong tag, so the XML breaks partway through.
fn breaks_partway(version: CityGmlVersion) -> String {
    let (citygml, gml) = match version {
        CityGmlVersion::V2 => ("2.0", "http://www.opengis.net/gml"),
        CityGmlVersion::V3 => ("3.0", "http://www.opengis.net/gml/3.2"),
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><core:CityModel xmlns:core="http://www.opengis.net/citygml/{citygml}" xmlns:bldg="http://www.opengis.net/citygml/building/{citygml}" xmlns:gml="{gml}"><core:cityObjectMember><bldg:Building gml:id="b1"><bldg:function>1000</bldg:function></bldg:Building></core:cityObjectMember><core:cityObjectMember><bldg:Building gml:id="b2"><bldg:function>1000</bldg:usage></bldg:Building></core:cityObjectMember></core:CityModel>"#
    )
}

/// `Parser::parse` commits each city object as it reads it, so an error
/// partway through leaves the objects before it in the parser. The Feature
/// CityGML readers refuse a relaxed `citygml.parse_failed` policy because of
/// this. If the parser ever gains rollback, this fails, and that refusal can
/// become a skip of just the broken file.
#[test]
fn the_parser_commits_objects_read_before_the_xml_breaks() {
    for version in [CityGmlVersion::V2, CityGmlVersion::V3] {
        let mut parser = Parser::with_extract_tags(version, HashSet::new());
        let url = Url::parse("file:///breaks-partway.gml").unwrap();
        assert!(
            parser
                .parse(breaks_partway(version).as_bytes(), &url)
                .is_err(),
            "{version:?}: the document must fail to parse"
        );
        let features = build_features(
            parser,
            &HashSet::new(),
            &HashMap::new(),
            None,
            true,
            false,
            &[],
            false,
        );
        assert_eq!(
            features.len(),
            1,
            "{version:?}: b1 is already in the parser"
        );
    }
}
