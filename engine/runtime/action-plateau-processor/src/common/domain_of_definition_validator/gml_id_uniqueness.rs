//! C03: the gml:ids are unique across all files.

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{FEATURES_PORT, Port},
};
use reearth_flow_types::{AttributeValue, Attributes, Feature};
use serde_json::Number;

use super::traversal::GmlIdEntry;

pub(super) static DUPLICATE_GML_ID_STATS_PORT: Lazy<Port> =
    Lazy::new(|| Port::new("duplicateGmlIdStats"));

/// The occurrences of each gml:id, each as { `tag` / `xpath` / `index` /
/// `udxDirs` / `name` : value }.
pub(super) type GmlIdTable = HashMap<String, Vec<HashMap<String, String>>>;

/// An occurrence of a gml:id on the element `tag` at `xpath` of `file`.
fn occurrence(file: &Feature, tag: &str, xpath: &str) -> HashMap<String, String> {
    let file_attribute = |key: &str| file.get(key).map(|v| v.to_string()).unwrap_or_default();
    HashMap::from([
        ("tag".to_string(), tag.to_string()),
        ("xpath".to_string(), xpath.to_string()),
        ("index".to_string(), file_attribute("index")),
        ("udxDirs".to_string(), file_attribute("udxDirs")),
        ("name".to_string(), file_attribute("name")),
    ])
}

/// The gml:ids of a member, whose own is `gml_id` on `tag`. A gml:id appearing
/// more than once in the member counts once, by its last occurrence.
pub(super) fn member_gml_ids(
    file: &Feature,
    gml_id: &str,
    tag: &str,
    entries: &[GmlIdEntry],
) -> GmlIdTable {
    let mut gml_ids = GmlIdTable::new();
    gml_ids.insert(gml_id.to_string(), vec![occurrence(file, tag, tag)]);
    for entry in entries {
        gml_ids.insert(
            entry.gml_id.clone(),
            vec![occurrence(file, &entry.tag, &entry.xpath)],
        );
    }
    gml_ids
}

/// Appends the occurrences of `from` to `into`.
pub(super) fn merge(into: &mut GmlIdTable, from: &GmlIdTable) {
    for (gml_id, occurrences) in from {
        into.entry(gml_id.clone())
            .or_default()
            .extend(occurrences.iter().cloned());
    }
}

/// The gml:ids of all files, checked when all are processed.
#[derive(Debug, Clone, Default)]
pub(super) struct GmlIdUniqueness {
    file_gml_ids: Vec<GmlIdTable>,
    filenames: HashSet<String>,
}

impl GmlIdUniqueness {
    /// Adds a file, which is in the statistics even when it is not processed.
    pub(super) fn add_filename(&mut self, name: String) {
        self.filenames.insert(name);
    }

    pub(super) fn add_file_gml_ids(&mut self, gml_ids: GmlIdTable) {
        self.file_gml_ids.push(gml_ids);
    }

    /// Sends the number of duplicate gml:ids of each file, then each
    /// occurrence of a duplicate gml:id.
    pub(super) fn finish(&self, ctx: &NodeContext, fw: &ProcessorChannelForwarder) {
        let mut gml_ids = GmlIdTable::new();
        for file_gml_ids in &self.file_gml_ids {
            merge(&mut gml_ids, file_gml_ids);
        }

        // Count duplicate gml:id occurrences per file
        let mut duplicate_gml_id_count = HashMap::<String, usize>::new();

        for attributes in gml_ids.values() {
            if attributes.len() > 1 {
                // Count each file containing duplicate gml:ids
                for attribute in attributes.iter() {
                    if let Some(name) = attribute.get("name") {
                        *duplicate_gml_id_count.entry(name.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Output statistics for all files (including those with 0 duplicates)
        for filename in &self.filenames {
            let duplicate_count = duplicate_gml_id_count.get(filename).unwrap_or(&0);

            let mut result_feature = Feature::new_with_attributes(Attributes::new());
            result_feature.insert("filename", AttributeValue::String(filename.clone()));
            result_feature.insert(
                "duplicateGmlIdCount",
                AttributeValue::Number(Number::from(*duplicate_count)),
            );

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                ctx,
                result_feature,
                DUPLICATE_GML_ID_STATS_PORT.clone(),
            ));
        }

        // Output individual duplicate gml:id details
        let mut dup_id = 0;
        for (gml_id, attributes) in gml_ids {
            if attributes.len() <= 1 {
                continue;
            }
            for attribute in attributes.iter() {
                let mut result_feature = Feature::new_with_attributes(Attributes::new());
                result_feature.insert(
                    "flag",
                    AttributeValue::String("GMLID_NotUnique".to_string()),
                );
                result_feature.insert("gmlId", AttributeValue::String(gml_id.clone()));
                result_feature.insert("duplicateId", AttributeValue::String(dup_id.to_string()));
                result_feature.insert(
                    "numDuplicates",
                    AttributeValue::Number(Number::from(attributes.len())),
                );
                attribute.iter().for_each(|(k, v)| {
                    result_feature.insert(k.clone(), AttributeValue::String(v.clone()));
                });
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    ctx,
                    result_feature,
                    FEATURES_PORT.clone(),
                ));
            }
            dup_id += 1;
        }
    }
}
