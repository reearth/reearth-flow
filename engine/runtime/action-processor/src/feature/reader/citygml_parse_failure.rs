//! How the Feature CityGML 2 and 3 Readers fail on a document they cannot parse.

use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_runtime::executor_operation::ExecutorContext;

/// Classify a parse failure and return what the reader's error appends to it.
///
/// Action Standard §9: at its default of fatal, `report` records
/// `citygml.parse_failed` in the node's fatal slot, which keeps the first
/// fatal, so it wins over the generic `internal.unclassified` the runtime adds
/// for the reader's `Err`.
///
/// The read fails whatever the policy says. `Parser::parse` streams, committing
/// each city object as it reads it, so a file that breaks partway has already
/// added its first objects to the parser, and `finish` would emit them as
/// though the file were complete. Honouring a relaxed disposition would turn a
/// broken file into silently partial output (§4.3, §9 on state left behind), so
/// a relaxed policy is refused with the reason instead.
pub(super) fn classify_parse_failure(ctx: &ExecutorContext) -> &'static str {
    let relaxed = ctx
        .report(DiagnosticDraft::new(ErrorCode::CitygmlParseFailed))
        .is_ok();
    if relaxed {
        " (citygml.parse_failed cannot be relaxed at this reader: part of the file may \
         already have been read, and skipping it would emit that part as though it were \
         complete)"
    } else {
        ""
    }
}

#[cfg(test)]
pub(super) mod test_support {
    use std::sync::Arc;

    use reearth_flow_citygml::parser::CityGmlVersion;
    use reearth_flow_diagnostics::{
        Disposition, DispositionPolicy, ErrorCode, OverrideInput, PolicyInput,
    };
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::errors::BoxedError;
    use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_runtime::node::{NodeHandle, Processor, FEATURES_PORT};
    use reearth_flow_types::{Attributes, Feature};

    /// A document whose first city object is complete and whose second closes
    /// an element with the wrong tag, so the XML breaks partway through.
    fn breaks_partway(version: CityGmlVersion) -> String {
        let (citygml, gml) = match version {
            CityGmlVersion::V2 => ("2.0", "http://www.opengis.net/gml"),
            CityGmlVersion::V3 => ("3.0", "http://www.opengis.net/gml/3.2"),
        };
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><core:CityModel xmlns:core="http://www.opengis.net/citygml/{citygml}" xmlns:bldg="http://www.opengis.net/citygml/building/{citygml}" xmlns:gml="{gml}"><core:cityObjectMember><bldg:Building gml:id="b1"><bldg:function>1000</bldg:function></bldg:Building></core:cityObjectMember><core:cityObjectMember><bldg:Building gml:id="b2"><bldg:function>1000</bldg:usage></bldg:Building></core:cityObjectMember></core:CityModel>"#
        )
    }

    fn relaxing_parse_failed() -> DispositionPolicy {
        DispositionPolicy::compile(PolicyInput {
            overrides: vec![OverrideInput {
                node: None,
                code: Some("citygml.parse_failed".to_string()),
                category: None,
                disposition: Disposition::WarnDrop,
            }],
            ..Default::default()
        })
        .expect("a code-only override should compile")
    }

    /// Runs `process` over a document that breaks partway, with a real
    /// diagnostics handle so the policy is actually consulted rather than
    /// falling back to the default. `reader` builds the reader for the
    /// document's `file://` URL.
    fn process_breaking_document<P: Processor>(
        action: &str,
        version: CityGmlVersion,
        policy: DispositionPolicy,
        reader: impl FnOnce(&str) -> P,
    ) -> (Result<(), BoxedError>, Arc<NodeDiagnosticsHandle>) {
        let dir = tempfile::tempdir().expect("a temporary directory should be creatable");
        let path = dir.path().join("breaks-partway.gml");
        std::fs::write(&path, breaks_partway(version)).expect("the fixture should be writable");
        let mut reader = reader(&format!("file://{}", path.display()));

        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "processor".into(),
            action.into(),
            Arc::default(),
            Arc::new(policy),
            false,
        ));
        let mut ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &NodeContext::default(),
            Feature::new_with_attributes(Attributes::new()),
            FEATURES_PORT.clone(),
        );
        ctx.diagnostics = Some(handle.clone());
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        (reader.process(ctx, &fw), handle)
    }

    /// The default policy fails the read, with `citygml.parse_failed` holding
    /// the fatal slot rather than the runtime's generic wrapper.
    pub(crate) fn assert_parse_failure_is_classified<P: Processor>(
        action: &str,
        version: CityGmlVersion,
        reader: impl FnOnce(&str) -> P,
    ) {
        let (result, handle) =
            process_breaking_document(action, version, DispositionPolicy::default(), reader);
        assert!(
            result.is_err(),
            "a document that cannot be parsed must fail the read"
        );
        let fatal = handle
            .inner
            .take_fatal()
            .expect("the classified code must occupy the fatal slot");
        assert_eq!(
            fatal.code,
            ErrorCode::CitygmlParseFailed,
            "the code must win the first-fatal slot over the runtime's generic wrapper"
        );
    }

    /// A policy relaxing `citygml.parse_failed` still fails the read, and the
    /// error says why it was not honoured.
    pub(crate) fn assert_relaxing_parse_failed_is_refused<P: Processor>(
        action: &str,
        version: CityGmlVersion,
        reader: impl FnOnce(&str) -> P,
    ) {
        let (result, _) =
            process_breaking_document(action, version, relaxing_parse_failed(), reader);
        let msg = result
            .expect_err("relaxing must not turn the failure into a skip")
            .to_string();
        assert!(
            msg.contains("cannot be relaxed"),
            "the refusal must say why the policy was not honoured, got: {msg}"
        );
    }
}
