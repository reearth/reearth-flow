//! C03: the gml:ids are unique across all files.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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

/// The attributes of an input file that an occurrence of a gml:id reports.
#[derive(Debug)]
pub(super) struct FileNames {
    index: String,
    udx_dirs: String,
    name: String,
}

impl FileNames {
    pub(super) fn new(file: &Feature) -> Arc<Self> {
        let attribute = |key: &str| file.get(key).map(|v| v.to_string()).unwrap_or_default();
        Arc::new(Self {
            index: attribute("index"),
            udx_dirs: attribute("udxDirs"),
            name: attribute("name"),
        })
    }
}

/// An occurrence of a gml:id on the element `tag` at `xpath`.
#[derive(Debug, Clone)]
pub(super) struct Occurrence {
    pub(super) tag: String,
    pub(super) xpath: String,
    file: Arc<FileNames>,
}

/// The gml:ids of a member, each with its last occurrence there.
pub(super) type MemberGmlIds = HashMap<String, Occurrence>;

/// The occurrences of each gml:id.
pub(super) type GmlIdTable = HashMap<String, Vec<Occurrence>>;

/// The gml:ids of a member, whose own is `gml_id` on `tag`, and `entries`
/// under it. A gml:id appearing more than once in the member counts once, by
/// its last occurrence.
pub(super) fn member_gml_ids(
    file: &Arc<FileNames>,
    gml_id: &str,
    tag: &str,
    entries: Vec<GmlIdEntry>,
) -> MemberGmlIds {
    let mut gml_ids = MemberGmlIds::new();
    gml_ids.insert(
        gml_id.to_string(),
        Occurrence {
            tag: tag.to_string(),
            xpath: tag.to_string(),
            file: Arc::clone(file),
        },
    );
    for entry in entries {
        gml_ids.insert(
            entry.gml_id,
            Occurrence {
                tag: entry.tag,
                xpath: entry.xpath,
                file: Arc::clone(file),
            },
        );
    }
    gml_ids
}

/// Appends the occurrences of a member to those of its file.
pub(super) fn merge_member(into: &mut GmlIdTable, member: MemberGmlIds) {
    for (gml_id, occurrence) in member {
        into.entry(gml_id).or_default().push(occurrence);
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
    pub(super) fn finish(&mut self, ctx: &NodeContext, fw: &ProcessorChannelForwarder) {
        let mut gml_ids = GmlIdTable::new();
        for file_gml_ids in std::mem::take(&mut self.file_gml_ids) {
            for (gml_id, occurrences) in file_gml_ids {
                gml_ids.entry(gml_id).or_default().extend(occurrences);
            }
        }

        // Count duplicate gml:id occurrences per file
        let mut duplicate_gml_id_count = HashMap::<&str, usize>::new();
        for occurrences in gml_ids.values().filter(|occurrences| occurrences.len() > 1) {
            for occurrence in occurrences {
                *duplicate_gml_id_count
                    .entry(occurrence.file.name.as_str())
                    .or_insert(0) += 1;
            }
        }

        // Output statistics for all files (including those with 0 duplicates)
        for filename in &self.filenames {
            let duplicate_count = duplicate_gml_id_count.get(filename.as_str()).unwrap_or(&0);

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
        for (gml_id, occurrences) in &gml_ids {
            if occurrences.len() <= 1 {
                continue;
            }
            for occurrence in occurrences {
                let mut result_feature = Feature::new_with_attributes(Attributes::new());
                for (key, value) in [
                    ("flag", "GMLID_NotUnique".to_string()),
                    ("gmlId", gml_id.clone()),
                    ("duplicateId", dup_id.to_string()),
                ] {
                    result_feature.insert(key, AttributeValue::String(value));
                }
                result_feature.insert(
                    "numDuplicates",
                    AttributeValue::Number(Number::from(occurrences.len())),
                );
                for (key, value) in [
                    ("tag", &occurrence.tag),
                    ("xpath", &occurrence.xpath),
                    ("index", &occurrence.file.index),
                    ("udxDirs", &occurrence.file.udx_dirs),
                    ("name", &occurrence.file.name),
                ] {
                    result_feature.insert(key, AttributeValue::String(value.clone()));
                }
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
