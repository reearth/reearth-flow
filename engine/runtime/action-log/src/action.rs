use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use slog::{Drain, Logger, OwnedKVList, Record};
use slog_term::Decorator;
use sloggers::file::FileLoggerBuilder;
use sloggers::null::NullLoggerBuilder;
use sloggers::types::Severity;
use sloggers::{Build, BuildWithCustomFormat};

use reearth_flow_common::str::{is_boolean, parse_boolean};

static ACTION_LOG_DISABLE: Lazy<Mutex<Option<String>>> =
    Lazy::new(|| Mutex::new(env::var("ACTION_LOG_DISABLE").ok()));

pub(crate) fn action_logger(root_path: PathBuf, action: &str) -> Logger {
    let disable = ACTION_LOG_DISABLE.lock().unwrap().clone();
    match disable {
        Some(disable) if is_boolean(&disable) && parse_boolean(&disable) => {
            NullLoggerBuilder.build().unwrap()
        }
        _ => {
            let mut builder = FileLoggerBuilder::new(format!(
                "{}/{}.log",
                root_path.to_str().unwrap_or_default(),
                action
            ));
            builder.level(Severity::Info);
            builder
                .build_with_custom_format(|decorator| Ok(CustomFormat::new(decorator)))
                .unwrap()
        }
    }
}

pub struct CustomFormat<D>
where
    D: Decorator,
{
    decorator: D,
}

impl<D> Drain for CustomFormat<D>
where
    D: Decorator,
{
    type Ok = ();
    type Err = std::io::Error;

    fn log(
        &self,
        record: &Record,
        values: &OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        self.format_custom(record, values)
    }
}

const TIMESTAMP_FORMAT: &[time::format_description::FormatItem] = time::macros::format_description!(
    "[year]/[month]/[day] [hour]:[minute]:[second] [offset_hour]:[offset_minute]"
);

impl<D> CustomFormat<D>
where
    D: Decorator,
{
    pub fn new(decorator: D) -> Self {
        CustomFormat { decorator }
    }

    fn format_custom(&self, record: &Record, values: &OwnedKVList) -> std::io::Result<()> {
        self.decorator.with_record(record, values, |decorator| {
            decorator.start_level()?;
            write!(decorator, "[{:?}]", record.level())?;

            decorator.start_whitespace()?;
            write!(decorator, " ")?;

            decorator.start_timestamp()?;
            let now: time::OffsetDateTime = std::time::SystemTime::now().into();
            write!(
                decorator,
                "{}",
                now.format(TIMESTAMP_FORMAT)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            )?;

            decorator.start_whitespace()?;
            write!(decorator, " ")?;

            decorator.start_whitespace()?;
            write!(decorator, " ")?;

            decorator.start_msg()?;
            write!(decorator, "{}", record.msg())?;

            writeln!(decorator)?;
            decorator.flush()?;

            Ok(())
        })
    }
}
