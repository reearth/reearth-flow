//! Test helpers shared by the CityGML 2 and 3 Readers.

use reearth_flow_diagnostics::ErrorCode;
use reearth_flow_runtime::event::{Event, EventHub};
use reearth_flow_runtime::executor_operation::NodeContext;
use reearth_flow_runtime::node::Source;
use tokio::sync::mpsc;

/// Run `start` and collect the codes it published.
///
/// Action Standard §9: a failure the runtime cannot classify reaches the user
/// as `internal.unclassified`/Fatal, with no code to search for and nothing to
/// write a policy against, so both failure paths in `start()` must publish
/// their code before returning `Err`.
///
/// A source's `NodeContext` carries no diagnostics handle, so `report_drop`
/// takes its `None` branch and publishes one raw `Event::Diagnostic` straight
/// to the hub. That is what this subscribes to. A broadcast receiver only sees
/// what is sent after it subscribes, hence the `resubscribe` before `start`.
pub(super) async fn codes_raised_by_start(mut reader: impl Source) -> Vec<ErrorCode> {
    let hub = EventHub::new(64);
    let mut rx = hub.receiver.resubscribe();
    let (tx, _rx) = mpsc::channel(16);
    let ctx = NodeContext {
        event_hub: hub.clone(),
        ..NodeContext::default()
    };
    let _ = reader.start(ctx, tx).await;

    let mut codes = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Event::Diagnostic(diagnostic) = event {
            codes.push(diagnostic.code);
        }
    }
    codes
}

/// A document that cannot be read as the reader's version is classified.
pub(super) async fn assert_unparseable_raises_parse_failed(reader: impl Source) {
    let codes = codes_raised_by_start(reader).await;
    assert!(
        codes.contains(&ErrorCode::CitygmlParseFailed),
        "a document of the wrong CityGML version must be classified, got: {codes:?}"
    );
}

/// Malformed geometry in a readable document is the *resolve* stage failing.
/// It must not be reported under the same code as a document that could not
/// be read at all, because the two need different user actions. Only the
/// new-geometry path reports malformations.
#[cfg(feature = "new-geometry")]
pub(super) async fn assert_malformed_raises_malformed_input(reader: impl Source) {
    let codes = codes_raised_by_start(reader).await;
    assert!(
        codes.contains(&ErrorCode::CitygmlMalformedInput),
        "malformed geometry in a readable document must be classified, got: {codes:?}"
    );
    assert!(
        !codes.contains(&ErrorCode::CitygmlParseFailed),
        "a readable document must not be reported as unparseable, got: {codes:?}"
    );
}
