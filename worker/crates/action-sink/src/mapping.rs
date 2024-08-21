use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, SinkFactory};

use crate::{echo::EchoSinkFactory, file::writer::FileWriterSinkFactory, noop::NoopSinkFactory};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn SinkFactory>> = vec![
        Box::<FileWriterSinkFactory>::default(),
        Box::<EchoSinkFactory>::default(),
        Box::<NoopSinkFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Sink(f)))
        .collect::<HashMap<_, _>>()
});
