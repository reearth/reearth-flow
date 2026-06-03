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

    let handle = tokio::spawn(async move { source.start(ctx, tx).await });

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
pub(crate) fn union_features(features: &[Feature]) -> AttrSchema {
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

#[cfg(test)]
mod tests {
    //! Direct unit tests for [`union_features`]. These build `Feature`s by hand
    //! (no source factory, no `action-source` dependency), so they do NOT trip
    //! the double-crate-version problem documented in the module header.
    use indexmap::IndexMap;
    use reearth_flow_types::attr_schema::{AttrType, Presence};
    use reearth_flow_types::{Attribute, AttributeValue, Feature};

    use super::union_features;

    /// Build a `Feature` from `(name, value)` pairs, preserving insertion order.
    fn feature(pairs: &[(&str, AttributeValue)]) -> Feature {
        let mut attrs: IndexMap<Attribute, AttributeValue> = IndexMap::new();
        for (name, value) in pairs {
            attrs.insert(Attribute::new(name.to_string()), value.clone());
        }
        Feature::new_with_attributes(attrs)
    }

    fn str_val(s: &str) -> AttributeValue {
        AttributeValue::String(s.to_string())
    }

    fn num_val(n: i64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from(n))
    }

    #[test]
    fn union_marks_partial_presence_as_maybe() {
        // feature1 has {a, b}; feature2 has {a} only.
        let features = [
            feature(&[("a", str_val("x")), ("b", str_val("y"))]),
            feature(&[("a", str_val("z"))]),
        ];

        let schema = union_features(&features);
        assert!(!schema.open, "unioned schema should be closed");

        let a = schema
            .fields
            .get(&Attribute::new("a".to_string()))
            .expect("a present");
        assert_eq!(a.presence, Presence::Always, "a is in all features");
        assert_eq!(a.ty, AttrType::String);

        let b = schema
            .fields
            .get(&Attribute::new("b".to_string()))
            .expect("b present");
        assert_eq!(b.presence, Presence::Maybe, "b is only in feature1");
        assert_eq!(b.ty, AttrType::String);
    }

    #[test]
    fn union_marks_type_conflict_as_unknown() {
        // x is a String in feature1 and a Number in feature2.
        let features = [
            feature(&[("x", str_val("hello"))]),
            feature(&[("x", num_val(1))]),
        ];

        let schema = union_features(&features);

        let x = schema
            .fields
            .get(&Attribute::new("x".to_string()))
            .expect("x present");
        assert_eq!(x.ty, AttrType::Unknown, "conflicting types collapse to Unknown");
        assert_eq!(
            x.presence,
            Presence::Always,
            "x is present in every feature"
        );
    }

    #[test]
    fn union_preserves_first_seen_order() {
        // Keys are first seen as c, a, b across the two features.
        let features = [
            feature(&[("c", num_val(1)), ("a", num_val(2))]),
            feature(&[("a", num_val(3)), ("b", num_val(4))]),
        ];

        let schema = union_features(&features);

        let order: Vec<String> = schema.fields.keys().map(|k| k.to_string()).collect();
        assert_eq!(
            order,
            vec!["c".to_string(), "a".to_string(), "b".to_string()],
            "fields should follow first-seen order"
        );
    }
}
