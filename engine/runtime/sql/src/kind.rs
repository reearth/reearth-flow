use crate::errors::Error;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnyKind {
    Postgres,
    MySql,
    Sqlite,
}

impl FromStr for AnyKind {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        match url {
            _ if url.starts_with("postgres://") || url.starts_with("postgresql://") => {
                Ok(AnyKind::Postgres)
            }

            _ if url.starts_with("mysql://") || url.starts_with("mariadb://") => Ok(AnyKind::MySql),

            _ if url.starts_with("sqlite://") => Ok(AnyKind::Sqlite),

            _ => Err(Error::Configuration(format!(
                "unrecognized database url: {url:?}"
            ))),
        }
    }
}
