//! The features the checks send for a file.

use reearth_flow_runtime::{
    executor_operation::ExecutorContext, forwarder::ProcessorChannelForwarder, node::FEATURES_PORT,
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};

/// Sends the features of the file `file`.
#[derive(Clone, Copy)]
pub(super) struct Reporter<'a> {
    ctx: &'a ExecutorContext,
    fw: &'a ProcessorChannelForwarder,
    file: &'a Feature,
}

impl<'a> Reporter<'a> {
    pub(super) fn new(
        ctx: &'a ExecutorContext,
        fw: &'a ProcessorChannelForwarder,
        file: &'a Feature,
    ) -> Self {
        Self { ctx, fw, file }
    }

    /// A feature with the attributes `keys` of the file, instead of a clone
    /// of the whole (potentially large) file feature.
    pub(super) fn file_feature(&self, keys: &[&str]) -> Feature {
        let mut attributes = Attributes::new();
        for key in keys {
            if let Some(value) = self.file.get(*key) {
                attributes.insert(Attribute::new(*key), value.clone());
            }
        }
        Feature::new_with_attributes(attributes)
    }

    pub(super) fn send(&self, feature: Feature) {
        self.fw.send(
            self.ctx
                .new_with_feature_and_port(feature, FEATURES_PORT.clone()),
        );
    }

    /// Sends the errors of the member `gml_id`.
    pub(super) fn member(&self, gml_id: &str, feature_type: &str) -> MemberReporter<'a> {
        let mut base = self.file_feature(&["index", "udxDirs", "name", "path"]);
        base.insert("gmlId", AttributeValue::String(gml_id.to_string()));
        base.insert(
            "featureType",
            AttributeValue::String(feature_type.to_string()),
        );
        MemberReporter {
            reporter: *self,
            base,
        }
    }
}

/// Sends the errors of a member, each with the attributes of the file and the
/// member.
pub(super) struct MemberReporter<'a> {
    reporter: Reporter<'a>,
    base: Feature,
}

impl MemberReporter<'_> {
    /// Sends an error `flag` with string `attributes`. An attribute of the
    /// member keeps its place.
    pub(super) fn send<'k>(
        &self,
        flag: &str,
        attributes: impl IntoIterator<Item = (&'k str, String)>,
    ) {
        self.send_with(flag, |feature| {
            for (key, value) in attributes {
                feature.insert(key, AttributeValue::String(value));
            }
        });
    }

    /// Sends an error `flag` with the attributes `fill` inserts.
    pub(super) fn send_with(&self, flag: &str, fill: impl FnOnce(&mut Feature)) {
        let mut feature = self.base.clone();
        feature.insert("flag", AttributeValue::String(flag.to_string()));
        fill(&mut feature);
        self.reporter.send(feature);
    }
}
