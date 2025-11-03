use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, SourceFactory};

use crate::{
    feature_creator::FeatureCreatorFactory,
    file::{
        citygml::CityGmlReaderFactory, csv::CsvReaderFactory, czml::CzmlReaderFactory,
        geojson::GeoJsonReaderFactory, geopackage::GeoPackageReaderFactory,
        gltf::GltfReaderFactory, json::JsonReaderFactory, obj::ObjReaderFactory,
        path_extractor::FilePathExtractorFactory,
        shapefile::ShapefileReaderFactory,
    },
    sql::SqlReaderFactory,
};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn SourceFactory>> = vec![
        Box::<CityGmlReaderFactory>::default(),
        Box::<FilePathExtractorFactory>::default(),
        Box::<FeatureCreatorFactory>::default(),
        Box::<SqlReaderFactory>::default(),
        Box::<CsvReaderFactory>::default(),
        Box::<CzmlReaderFactory>::default(),
        Box::<GeoJsonReaderFactory>::default(),
        Box::<GeoPackageReaderFactory>::default(),
        Box::<GltfReaderFactory>::default(),
        Box::<JsonReaderFactory>::default(),
        Box::<ObjReaderFactory>::default(),
        Box::<ShapefileReaderFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Source(f)))
        .collect::<HashMap<_, _>>()
});
