use serde::ser::SerializeMap;
use slog::o;
use slog::Key;
use slog::Record;
use slog::{FnValue, PushFnValue};
use slog::{OwnedKVList, SendSyncRefUnwindSafeKV, KV};
use slog_term::Decorator;
use std::{fmt, io, result};

use std::cell::RefCell;
use std::fmt::Write;

thread_local! {
    static TL_BUF: RefCell<String> = RefCell::new(String::with_capacity(128))
}

pub(crate) struct SerdeSerializer<S: serde::Serializer> {
    /// Current state of map serializing: `serde::Serializer::MapState`
    ser_map: S::SerializeMap,
}

impl<S: serde::Serializer> SerdeSerializer<S> {
    /// Start serializing map of values
    fn start(ser: S, len: Option<usize>) -> result::Result<Self, slog::Error> {
        let ser_map = ser
            .serialize_map(len)
            .map_err(|e| io::Error::other(format!("serde serialization error: {e}")))?;
        Ok(SerdeSerializer { ser_map })
    }

    /// Finish serialization, and return the serializer
    fn end(self) -> result::Result<S::Ok, S::Error> {
        self.ser_map.end()
    }
}

macro_rules! impl_m(
    ($s:expr, $key:expr, $val:expr) => ({
        let k_s:  &str = $key.as_ref();
        $s.ser_map.serialize_entry(k_s, $val)
             .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("serde serialization error: {}", e)))?;
        Ok(())
    });
);

impl<S> slog::Serializer for SerdeSerializer<S>
where
    S: serde::Serializer,
{
    fn emit_bool(&mut self, key: Key, val: bool) -> slog::Result {
        impl_m!(self, key, &val)
    }

    fn emit_unit(&mut self, key: Key) -> slog::Result {
        impl_m!(self, key, &())
    }

    fn emit_char(&mut self, key: Key, val: char) -> slog::Result {
        impl_m!(self, key, &val)
    }

    fn emit_none(&mut self, key: Key) -> slog::Result {
        let val: Option<()> = None;
        impl_m!(self, key, &val)
    }
    fn emit_u8(&mut self, key: Key, val: u8) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_i8(&mut self, key: Key, val: i8) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_u16(&mut self, key: Key, val: u16) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_i16(&mut self, key: Key, val: i16) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_usize(&mut self, key: Key, val: usize) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_isize(&mut self, key: Key, val: isize) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_u32(&mut self, key: Key, val: u32) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_i32(&mut self, key: Key, val: i32) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_f32(&mut self, key: Key, val: f32) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_u64(&mut self, key: Key, val: u64) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_i64(&mut self, key: Key, val: i64) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_f64(&mut self, key: Key, val: f64) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_u128(&mut self, key: Key, val: u128) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_i128(&mut self, key: Key, val: i128) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_str(&mut self, key: Key, val: &str) -> slog::Result {
        impl_m!(self, key, &val)
    }
    fn emit_arguments(&mut self, key: Key, val: &fmt::Arguments) -> slog::Result {
        TL_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();

            buf.write_fmt(*val)
                .map_err(|e| io::Error::other(format!("Error formatting arguments: {e}")))?;

            let res = { || impl_m!(self, key, &*buf) }();
            buf.clear();
            res
        })
    }
}

/// Json `Drain`
///
/// Each record will be printed as a Json map
/// to a given `io`
pub(crate) struct Json<D>
where
    D: Decorator,
{
    decorator: D,
    values: Vec<OwnedKVList>,
}

impl<D> Json<D>
where
    D: Decorator,
{
    pub fn new(decorator: D) -> Json<D> {
        JsonBuilder::new(decorator).add_default_keys().build()
    }

    fn log_impl<W, F>(
        &self,
        serializer: &mut serde_json::ser::Serializer<&mut W, F>,
        rinfo: &Record,
        logger_values: &OwnedKVList,
    ) -> io::Result<()>
    where
        W: io::Write,
        F: serde_json::ser::Formatter,
    {
        let mut serializer = SerdeSerializer::start(&mut *serializer, None)?;

        for kv in &self.values {
            kv.serialize(rinfo, &mut serializer)?;
        }

        logger_values.serialize(rinfo, &mut serializer)?;

        rinfo.kv().serialize(rinfo, &mut serializer)?;

        let res = serializer.end();

        res.map_err(io::Error::other)?;

        Ok(())
    }
}

impl<D> slog::Drain for Json<D>
where
    D: Decorator,
{
    type Ok = ();
    type Err = io::Error;
    fn log(&self, record: &Record, values: &OwnedKVList) -> io::Result<()> {
        self.decorator.with_record(record, values, |decorator| {
            let mut buffer = Vec::new();
            let mut serializer = serde_json::Serializer::new(&mut buffer);
            self.log_impl(&mut serializer, record, values)?;
            serializer.into_inner();
            let json_str = String::from_utf8(buffer)
                .map_err(|e| io::Error::other(format!("Invalid UTF-8 sequence: {e}")))?;
            writeln!(decorator, "{json_str}")?;
            decorator.flush()?;
            Ok(())
        })
    }
}

/// Json `Drain` builder
///
/// Create with `Json::new`.
pub struct JsonBuilder<D: Decorator> {
    values: Vec<OwnedKVList>,
    decorator: D,
}

impl<D> JsonBuilder<D>
where
    D: Decorator,
{
    pub(crate) fn new(decorator: D) -> Self {
        JsonBuilder {
            values: vec![],
            decorator,
        }
    }

    /// Build `Json` `Drain`
    ///
    /// This consumes the builder.
    pub fn build(self) -> Json<D> {
        Json {
            values: self.values,
            decorator: self.decorator,
        }
    }

    /// Add custom values to be printed with this formatter
    pub fn add_key_value<T>(mut self, value: slog::OwnedKV<T>) -> Self
    where
        T: SendSyncRefUnwindSafeKV + 'static,
    {
        self.values.push(value.into());
        self
    }

    /// Add default key-values:
    ///
    /// * `ts` - timestamp
    /// * `level` - record logging level name
    /// * `msg` - msg - formatted logging message
    pub fn add_default_keys(self) -> Self {
        self.add_key_value(o!(
            "ts" => FnValue(move |_ : &Record| {
                    time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .ok()
            }),
            "level" => FnValue(move |rinfo : &Record| {
                rinfo.level().as_str()
            }),
            "msg" => PushFnValue(move |record : &Record, ser| {
                ser.emit(record.msg())
            }),
        ))
    }
}
