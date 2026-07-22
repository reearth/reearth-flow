use std::str::FromStr;

use futures_util::{FutureExt, TryStreamExt};
use kind::AnyKind;
use once_cell::sync::Lazy;
use sqlx::any::install_drivers;
use sqlx::any::Any;
use sqlx::any::AnyPoolOptions;
use sqlx::any::AnyRow;
use sqlx::sqlite::{SqlitePoolOptions, SqliteRow};
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
    url: String,
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
        Ok(Self { pool, url })
    }

    /// Fetch rows using the native SQLite driver instead of the generic `Any`
    /// driver. The `Any` driver has no type mapping for several SQLite types
    /// (notably BOOLEAN), so a `SELECT *` over such a column fails; the native
    /// driver decodes them. Only valid for `sqlite://` connections.
    pub async fn fetch_many_sqlite(&self, query: &str) -> crate::errors::Result<Vec<SqliteRow>> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(self.url.as_str())
            .await
            .map_err(crate::errors::Error::init)?;
        let result: Vec<SqliteRow> = pool
            .fetch(sqlx::query(query))
            .try_collect()
            .boxed()
            .await
            .map_err(crate::errors::Error::fetch)?;
        Ok(result)
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

    pub async fn execute(&self, query: &str) -> crate::errors::Result<u64> {
        let result = self
            .pool
            .execute(sqlx::query(query))
            .await
            .map_err(crate::errors::Error::fetch)?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use super::*;

    // The generic `Any` driver has no mapping for SQLite's BOOLEAN type, so a
    // `SELECT *` over a table with a boolean column fails outright (this is what
    // breaks reading a GeoPackage layer like 3DBAG's `pand`). The native SQLite
    // fetch must read it successfully.
    #[tokio::test]
    async fn fetch_many_sqlite_reads_boolean_column() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", dir.path().join("t.db").display());
        let adapter = SqlAdapter::new(url, 1).await.unwrap();
        adapter
            .execute("CREATE TABLE t (id INTEGER, flag BOOLEAN)")
            .await
            .unwrap();
        adapter
            .execute("INSERT INTO t VALUES (1, 1), (2, 0)")
            .await
            .unwrap();

        // The `Any` driver cannot decode SQLite BOOLEAN.
        assert!(adapter.fetch_many("SELECT * FROM t").await.is_err());

        // The native SQLite fetch can.
        let rows = adapter.fetch_many_sqlite("SELECT * FROM t").await.unwrap();
        assert_eq!(rows.len(), 2);
        let flag: bool = rows[0].try_get("flag").unwrap();
        assert!(flag);
    }
}
