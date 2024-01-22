use serde_json::{Map, Value};
use time::format_description::well_known::Iso8601;
use time::formatting::Formattable;
use time::OffsetDateTime;
use tracing::level_filters::LevelFilter;
use tracing::span::{Attributes, Record};
use tracing::{Event, Id, Metadata, Subscriber};
use tracing_subscriber::layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// # Examples
/// ```
/// use time::macros::format_description;
/// use tracing::{error, info_span};
/// use tracing_subscriber::prelude::*;
/// use tracing_json_span_fields::JsonLayer;
/// tracing_subscriber::registry().with(JsonLayer::default()).init();
/// let _span = info_span!("A span", span_field = 42).entered();
/// error!(logged_message_field = "value", "Logged message");
/// ```
#[derive(Debug)]
struct CustomFieldStorage(Map<String, Value>);

pub trait JsonOutput {
    fn write(&self, value: Value);
}

/// Default [`JsonOutput`] writing to stdout.
#[derive(Default)]
pub struct JsonStdout {
    pretty: bool,
}

impl JsonOutput for JsonStdout {
    fn write(&self, value: Value) {
        println!(
            "{}",
            if self.pretty {
                serde_json::to_string_pretty(&value).unwrap()
            } else {
                serde_json::to_string(&value).unwrap()
            }
        );
    }
}

pub struct JsonLayer<O = JsonStdout, F = Iso8601> {
    output: O,
    timestamp_format: F,
    max_level: LevelFilter,
}

impl Default for JsonLayer {
    fn default() -> Self {
        JsonLayer {
            output: JsonStdout::default(),
            timestamp_format: Iso8601::DEFAULT,
            max_level: LevelFilter::INFO,
        }
    }
}

impl JsonLayer<JsonStdout, Iso8601> {
    pub fn pretty() -> JsonLayer<JsonStdout, Iso8601> {
        JsonLayer::default().with_output(JsonStdout { pretty: true })
    }
}

impl<O, F> JsonLayer<O, F>
where
    F: Formattable,
    O: JsonOutput,
{
    pub fn with_output<O2: JsonOutput>(self, output: O2) -> JsonLayer<O2, F> {
        JsonLayer {
            output,
            timestamp_format: self.timestamp_format,
            max_level: self.max_level,
        }
    }
    pub fn with_level(self, max_level: LevelFilter) -> JsonLayer<O, F> {
        JsonLayer {
            output: self.output,
            timestamp_format: self.timestamp_format,
            max_level,
        }
    }
}

impl<S, O, F> layer::Layer<S> for JsonLayer<O, F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    O: JsonOutput + 'static,
    F: Formattable + 'static,
{
    fn enabled(&self, metadata: &Metadata<'_>, _ctx: Context<'_, S>) -> bool {
        metadata.level() <= &self.max_level
    }

    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = Map::new();
        let mut visitor = JsonVisitor(&mut fields);
        attrs.record(&mut visitor);

        // And stuff it in our newtype.
        let storage = CustomFieldStorage(fields);

        // Get a reference to the internal span data
        let span = ctx.span(id).unwrap();
        // Get the special place where tracing stores custom data
        let mut extensions = span.extensions_mut();
        // And store our data
        extensions.insert::<CustomFieldStorage>(storage);
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(self.max_level)
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        // Get the span whose data is being recorded
        let span = ctx.span(id).unwrap();

        // Get a mutable reference to the data we created in new_span
        let mut extensions_mut = span.extensions_mut();
        let custom_field_storage: &mut CustomFieldStorage =
            extensions_mut.get_mut::<CustomFieldStorage>().unwrap();
        let json_data: &mut Map<String, Value> = &mut custom_field_storage.0;

        // And add to using our old friend the visitor!
        let mut visitor = JsonVisitor(json_data);
        values.record(&mut visitor);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut fields = Map::new();

        // The fields of the spans
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let extensions = span.extensions();
                let storage = extensions.get::<CustomFieldStorage>().unwrap();
                let field_data: &Map<String, Value> = &storage.0;

                for (key, value) in field_data {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }
        // The fields of the event
        let mut visitor = JsonVisitor(&mut fields);
        event.record(&mut visitor);

        // Add default fields
        fields.insert("target".to_string(), event.metadata().target().into());
        fields.insert("name".to_string(), event.metadata().name().into());
        fields.insert(
            "log_level".to_string(),
            event.metadata().level().as_str().into(),
        );
        fields.insert(
            "timestamp".to_string(),
            OffsetDateTime::now_utc()
                .format(&self.timestamp_format)
                .unwrap()
                .into(),
        );
        // And create our output
        let output = fields.into();
        self.output.write(output);
    }
}

struct JsonVisitor<'a>(&'a mut Map<String, Value>);

impl<'a> tracing::field::Visit for JsonVisitor<'a> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(value.to_string()),
        );
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(format!("{:?}", value)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing::subscriber::with_default;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::Registry;

    /// A helper function for asserting a serde::Value matches expectations
    fn assert_json_timestamp_name(expected: Value, value: &mut Value) {
        assert_json_timestamp_name_with_format(expected, value)
    }

    /// A helper function for asserting a serde::Value matches expectations
    fn assert_json_timestamp_name_with_format(expected: Value, value: &mut Value) {
        let map = value.as_object_mut().unwrap();
        assert!(map.contains_key("name"));
        map.remove("name");
        assert!(map.contains_key("timestamp"));
        map.remove("timestamp");
        assert_eq!(expected, *value)
    }

    struct TestOutput {
        data: Arc<Mutex<Vec<Value>>>,
    }

    impl JsonOutput for TestOutput {
        fn write(&self, value: Value) {
            let mut data = self.data.lock().unwrap();
            (*data).push(value);
        }
    }

    #[test]
    fn one_span_some_fields() {
        let data = Arc::new(Mutex::new(vec![]));
        let layer = JsonLayer::default().with_output(TestOutput { data: data.clone() });
        tracing_subscriber::registry().with(layer).init();

        let _span1 = tracing::info_span!("Top level", field_top = 0).entered();
        tracing::info!(field_event = "from event", "FOOBAR");
        tracing::error!("BAZ");

        let mut data = data.lock().unwrap();
        let mut iter = (*data).iter_mut();

        assert_json_timestamp_name(
            serde_json::json!({
                "target": "reearth_flow_logging::tests",
                "log_level": "INFO",
                "message": "FOOBAR",
                "field_top": 0,
                "field_event": "from event"
            }),
            iter.next().unwrap(),
        );
        assert_json_timestamp_name(
            serde_json::json!({
                "target": "reearth_flow_logging::tests",
                "log_level": "ERROR",
                "message": "BAZ",
                "field_top": 0,
            }),
            iter.next().unwrap(),
        );
        assert_eq!(None, iter.next(), "No more logged events");
    }

    #[test]
    fn two_spans_same_fields() {
        let data = Arc::new(Mutex::new(vec![]));
        let layer = JsonLayer::default().with_output(TestOutput { data: data.clone() });

        let subscriber = Registry::default().with(layer);

        with_default(subscriber, || {
            let _span1 = tracing::info_span!("Top level", field_overwrite = 0).entered();
            let _span2 = tracing::info_span!("Second level", field_overwrite = 1).entered();
            tracing::info!(field_event = "from event", "FOOBAR");
        });

        let mut data = data.lock().unwrap();
        let mut iter = (*data).iter_mut();

        assert_json_timestamp_name(
            serde_json::json!({
                "target": "reearth_flow_logging::tests",
                "log_level": "INFO",
                "message": "FOOBAR",
                "field_overwrite": 1,
                "field_event": "from event"
            }),
            iter.next().unwrap(),
        );
        assert_eq!(None, iter.next(), "No more logged events");
    }
}
