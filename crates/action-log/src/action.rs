use std::path::PathBuf;

use slog::{Drain, Logger, OwnedKVList, Record};
use slog_term::{timestamp_local, Decorator};
use sloggers::file::FileLoggerBuilder;
use sloggers::types::Severity;
use sloggers::BuildWithCustomFormat;

pub(crate) fn action_logger(root_path: PathBuf, action: &str) -> Logger {
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

impl<D> CustomFormat<D>
where
    D: Decorator,
{
    pub fn new(decorator: D) -> Self {
        CustomFormat { decorator }
    }

    fn format_custom(&self, record: &Record, values: &OwnedKVList) -> std::io::Result<()> {
        self.decorator.with_record(record, values, |mut decorator| {
            decorator.start_timestamp()?;
            timestamp_local(&mut decorator)?;

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
