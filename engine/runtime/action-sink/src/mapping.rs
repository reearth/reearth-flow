use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, SinkFactory};

use crate::{
    echo::EchoSinkFactory,
    file::{
        cesium3dtiles::sink::Cesium3DTilesSinkFactory, csv::CsvWriterFactory,
        czml::CzmlWriterFactory, excel::ExcelWriterFactory, geojson::GeoJsonWriterFactory,
        gltf::GltfWriterSinkFactory, mvt::sink::MVTSinkFactory, obj::ObjWriterFactory,
        shapefile::ShapefileWriterFactory, writer::FileWriterSinkFactory, xml::XmlWriterFactory,
        zip::ZipFileWriterFactory,
    },
    noop::NoopSinkFactory,
};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn SinkFactory>> = vec![
        Box::<FileWriterSinkFactory>::default(),
        Box::<Cesium3DTilesSinkFactory>::default(),
        Box::<CsvWriterFactory>::default(),
        Box::<EchoSinkFactory>::default(),
        Box::<ExcelWriterFactory>::default(),
        Box::<NoopSinkFactory>::default(),
        Box::<GeoJsonWriterFactory>::default(),
        Box::<MVTSinkFactory>::default(),
        Box::<GltfWriterSinkFactory>::default(),
        Box::<CzmlWriterFactory>::default(),
        Box::<ObjWriterFactory>::default(),
        Box::<ShapefileWriterFactory>::default(),
        Box::<XmlWriterFactory>::default(),
        Box::<ZipFileWriterFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Sink(f)))
        .collect::<HashMap<_, _>>()
});
