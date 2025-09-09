use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, SourceFactory};

use crate::{
    feature_creator::FeatureCreatorFactory,
    file::{
        csv::CsvReaderFactory, czml::CzmlReaderFactory, geojson::GeoJsonReaderFactory,
        obj::ObjReaderFactory, path_extractor::FilePathExtractorFactory, reader::FileReaderFactory,
        shapefile::ShapefileReaderFactory,
    },
    sql::SqlReaderFactory,
};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn SourceFactory>> = vec![
        Box::<FileReaderFactory>::default(),
        Box::<FilePathExtractorFactory>::default(),
        Box::<FeatureCreatorFactory>::default(),
        Box::<SqlReaderFactory>::default(),
        Box::<CsvReaderFactory>::default(),
        Box::<CzmlReaderFactory>::default(),
        Box::<GeoJsonReaderFactory>::default(),
        Box::<ObjReaderFactory>::default(),
        Box::<ShapefileReaderFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Source(f)))
        .collect::<HashMap<_, _>>()
});
