use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub fn add_trace_middleware(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true))
}
