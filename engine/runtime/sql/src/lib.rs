use std::str::FromStr;

use futures_util::{FutureExt, TryStreamExt};
use kind::AnyKind;
use once_cell::sync::Lazy;
use sqlx::any::install_drivers;
use sqlx::any::Any;
use sqlx::any::AnyPoolOptions;
use sqlx::any::AnyRow;
use sqlx::Executor;
use sqlx::Pool;

pub(crate) mod errors;
pub(crate) mod kind;

static INSTALL_DRIVERS: Lazy<()> = Lazy::new(|| {
    install_drivers(&[
        sqlx::mysql::any::DRIVER,
        sqlx::postgres::any::DRIVER,
        sqlx::sqlite::any::DRIVER,
    ])
    .expect("non-default drivers already installed")
});

#[derive(Debug, Clone)]
pub struct SqlAdapter {
    pool: Pool<Any>,
}

impl SqlAdapter {
    pub async fn new<U: Into<String>>(url: U, pool_size: u32) -> crate::errors::Result<Self> {
        let _ = &*INSTALL_DRIVERS;
        let url: String = url.into();
        let _ = AnyKind::from_str(url.as_str())?;
        let pool = AnyPoolOptions::new()
            .max_connections(pool_size)
            .connect(url.as_str())
            .await
            .map_err(crate::errors::Error::init)?;
        Ok(Self { pool })
    }

    pub async fn fetch_many(&self, query: &str) -> crate::errors::Result<Vec<AnyRow>> {
        let result: Vec<AnyRow> = self
            .pool
            .fetch(sqlx::query(query))
            .try_collect()
            .boxed()
            .await
            .map_err(crate::errors::Error::fetch)?;
        Ok(result)
    }
}
