use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    coordinate_system_setter::CoordinateSystemSetterFactory, extruder::ExtruderFactory,
    filter::GeometryFilterFactory, splitter::GeometrySplitterFactory,
    three_dimention_box_replacer::ThreeDimentionBoxReplacerFactory,
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<CoordinateSystemSetterFactory>::default(),
        Box::<ExtruderFactory>::default(),
        Box::<ThreeDimentionBoxReplacerFactory>::default(),
        Box::<GeometryFilterFactory>::default(),
        Box::<GeometrySplitterFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
