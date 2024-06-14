use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    coordinate_system_setter::CoordinateSystemSetterFactory, extractor::GeometryExtractorFactory,
    extruder::ExtruderFactory, filter::GeometryFilterFactory, reprojector::ReprojectorFactory,
    splitter::GeometrySplitterFactory,
    three_dimention_box_replacer::ThreeDimentionBoxReplacerFactory,
    two_dimention_forcer::TwoDimentionForcerFactory,
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<CoordinateSystemSetterFactory>::default(),
        Box::<ExtruderFactory>::default(),
        Box::<ThreeDimentionBoxReplacerFactory>::default(),
        Box::<GeometryFilterFactory>::default(),
        Box::<GeometrySplitterFactory>::default(),
        Box::<ReprojectorFactory>::default(),
        Box::<TwoDimentionForcerFactory>::default(),
        Box::<GeometryExtractorFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
