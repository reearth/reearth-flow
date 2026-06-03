//! Runtime source sampling: briefly execute a source node, read the first N
//! emitted features, and union them into a *closed* [`AttrSchema`].
//!
//! Unlike static inference (`schema_infer`), this actually drives the async
//! source reader to discover real attribute names/types. It is bounded by
//! `sample_size` and never panics: any failure (not a source, build error,
//! runtime error, empty output) is reported as an open schema plus a `note`.
//!
//! Tests live in `runtime/runtime/tests/schema_sample.rs` (integration), not
//! inline: they need a real source factory from `reearth-flow-action-source`,
//! which depends back on this crate — an inline `#[cfg(test)]` use would
//! compile two incompatible copies of `reearth-flow-runtime`.

use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType};
use reearth_flow_types::attribute::{Attribute, AttributeValue};
use reearth_flow_types::Feature;

use crate::event::EventHub;
use crate::executor_operation::NodeContext;
use crate::node::{IngestionMessage, NodeKind, Port};

/// Result of sampling a source node.
pub struct SampleOutcome {
    /// The unioned schema. Closed when features were observed; open otherwise.
    pub schema: AttrSchema,
    /// Set when sampling could not produce a closed schema (with the reason).
    pub note: Option<String>,
}

/// Drive `kind` (if it is a source) for a bounded number of features and union
/// the observed attributes into a closed [`AttrSchema`].
///
/// `sample_size == 0` means "read all features the source emits".
///
/// Never panics. On any failure returns `{ AttrSchema::open(), Some(reason) }`.
pub fn sample_source(
    kind: &NodeKind,
    with: &Option<HashMap<String, serde_json::Value>>,
    sample_size: usize,
) -> SampleOutcome {
    let NodeKind::Source(factory) = kind else {
        return SampleOutcome {
            schema: AttrSchema::open(),
            note: Some("not a source node".to_string()),
        };
    };

    let ctx = NodeContext::default();
    let source = match factory.build(
        ctx.clone(),
        EventHub::new(30),
        String::new(),
        with.clone(),
        None,
    ) {
        Ok(source) => source,
        Err(e) => {
            return SampleOutcome {
                schema: AttrSchema::open(),
                note: Some(format!("source build failed: {e}")),
            }
        }
    };

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            return SampleOutcome {
                schema: AttrSchema::open(),
                note: Some(format!("runtime build failed: {e}")),
            }
        }
    };

    let result = runtime.block_on(read_features(source, ctx, sample_size));

    match result {
        Ok(features) if !features.is_empty() => SampleOutcome {
            schema: union_features(&features),
            note: None,
        },
        Ok(_) => SampleOutcome {
            schema: AttrSchema::open(),
            note: Some("source produced no features in sample".to_string()),
        },
        Err(e) => SampleOutcome {
            schema: AttrSchema::open(),
            note: Some(format!("source run failed: {e}")),
        },
    }
}

/// Spawn the source's `start`, drain features off the channel up to `sample_size`,
/// then drop the receiver (so a still-running source stops on a closed channel)
/// and join the task. Errors from the source are surfaced as a `String`.
async fn read_features(
    mut source: Box<dyn crate::node::Source>,
    ctx: NodeContext,
    sample_size: usize,
) -> Result<Vec<Feature>, String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<(Port, IngestionMessage)>(256);

    let start_ctx = ctx.clone();
    let handle = tokio::spawn(async move { source.start(start_ctx, tx).await });

    let mut features: Vec<Feature> = Vec::new();
    while let Some((_port, message)) = rx.recv().await {
        let IngestionMessage::OperationEvent { feature } = message;
        features.push(feature);
        if sample_size != 0 && features.len() >= sample_size {
            break;
        }
    }
    // Closing the receiver lets a still-running source observe a closed channel
    // and unwind (its `sender.send().await` will error), rather than hang.
    drop(rx);

    match handle.await {
        Ok(Ok(())) => Ok(features),
        // A send error after we stopped reading is expected when we hit the
        // sample cap; only treat it as fatal if we got no features at all.
        Ok(Err(e)) => {
            if features.is_empty() {
                Err(e.to_string())
            } else {
                Ok(features)
            }
        }
        Err(join_err) => {
            if features.is_empty() {
                Err(join_err.to_string())
            } else {
                Ok(features)
            }
        }
    }
}

/// Union a non-empty slice of features into a *closed* schema.
///
/// - A key seen with differing types across features collapses to `Unknown`.
/// - A key present in ALL features is `Always`, otherwise `Maybe`.
/// - First-seen key order is preserved.
fn union_features(features: &[Feature]) -> AttrSchema {
    struct Acc {
        ty: AttrType,
        conflicting: bool,
        count: usize,
    }

    let total = features.len();
    let mut acc: IndexMap<Attribute, Acc> = IndexMap::new();

    for feature in features {
        for (name, value) in feature.attributes.iter() {
            let ty = attr_type_of(value);
            match acc.get_mut(name) {
                Some(existing) => {
                    if !existing.conflicting && existing.ty != ty {
                        existing.conflicting = true;
                    }
                    existing.count += 1;
                }
                None => {
                    acc.insert(
                        name.clone(),
                        Acc {
                            ty,
                            conflicting: false,
                            count: 1,
                        },
                    );
                }
            }
        }
    }

    let mut schema = AttrSchema::empty();
    for (name, a) in acc {
        let ty = if a.conflicting { AttrType::Unknown } else { a.ty };
        let field = if a.count == total {
            AttrField::always(ty)
        } else {
            AttrField::maybe(ty)
        };
        schema.insert(name, field);
    }
    schema
}

fn attr_type_of(value: &AttributeValue) -> AttrType {
    match value {
        AttributeValue::Null => AttrType::Null,
        AttributeValue::Bool(_) => AttrType::Bool,
        AttributeValue::Number(_) => AttrType::Number,
        AttributeValue::String(_) => AttrType::String,
        AttributeValue::DateTime(_) => AttrType::DateTime,
        AttributeValue::Array(_) => AttrType::Array,
        AttributeValue::Map(_) => AttrType::Map,
        AttributeValue::Bytes(_) => AttrType::Bytes,
    }
}
