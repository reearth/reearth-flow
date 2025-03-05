use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    counter::FeatureCounterFactory,
    duplicate_filter::FeatureDuplicateFilterFactory,
    file_path_extractor::FeatureFilePathExtractorFactory,
    filter::FeatureFilterFactory,
    list_exploder::ListExploderFactory,
    lod_filter::FeatureLodFilterFactory,
    merger::FeatureMergerFactory,
    reader::{citygml::processor::FeatureCityGmlReaderFactory, FeatureReaderFactory},
    rhai::RhaiCallerFactory,
    sorter::FeatureSorterFactory,
    transformer::FeatureTransformerFactory,
    type_filter::FeatureTypeFilterFactory,
    writer::FeatureWriterFactory,
};

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<FeatureMergerFactory>::default(),
        Box::<FeatureSorterFactory>::default(),
        Box::<FeatureFilterFactory>::default(),
        Box::<FeatureTransformerFactory>::default(),
        Box::<FeatureCounterFactory>::default(),
        Box::<FeatureReaderFactory>::default(),
        Box::<RhaiCallerFactory>::default(),
        Box::<ListExploderFactory>::default(),
        Box::<FeatureTypeFilterFactory>::default(),
        Box::<FeatureFilePathExtractorFactory>::default(),
        Box::<FeatureLodFilterFactory>::default(),
        Box::<FeatureDuplicateFilterFactory>::default(),
        Box::<FeatureWriterFactory>::default(),
        Box::<FeatureCityGmlReaderFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
