use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use reearth_flow_sql::SqlAdapter;
use reearth_flow_types::{Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::errors::SourceError;

#[derive(Debug, Clone, Default)]
pub struct SqlReaderFactory;

impl SourceFactory for SqlReaderFactory {
    fn name(&self) -> &str {
        "SqlReader"
    }

    fn description(&self) -> &str {
        "Read Features from SQL Database"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SqlReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn tags(&self) -> &[&'static str] {
        &["database"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let param: SqlReaderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::SqlReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::SqlReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::SqlReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let compiled = SqlReaderCompiledParam {
            sql: param.sql.compile().map_err(|e| {
                SourceError::SqlReaderFactory(format!("Failed to compile sql: {e:?}"))
            })?,
            database_url: param.database_url.compile().map_err(|e| {
                SourceError::SqlReaderFactory(format!("Failed to compile database_url: {e:?}"))
            })?,
        };
        Ok(Box::new(SqlReader { param: compiled }))
    }
}

/// # SQL Reader Parameters
/// Configure the SQL query and database connection for reading features from a database
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SqlReaderParam {
    /// # SQL Query
    /// SQL query expression to execute for retrieving data
    pub(super) sql: Code,
    /// # Database URL
    /// Database connection URL (e.g. `sqlite:///tests/sqlite/sqlite.db`, `mysql://user:password@localhost:3306/db`, `postgresql://user:password@localhost:5432/db`)
    pub(super) database_url: Code,
}

#[derive(Debug, Clone)]
struct SqlReaderCompiledParam {
    sql: CompiledCode,
    database_url: CompiledCode,
}

#[derive(Debug, Clone)]
pub struct SqlReader {
    param: SqlReaderCompiledParam,
}

#[async_trait::async_trait]
impl Source for SqlReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "SqlReader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let database_url = self
            .param
            .database_url
            .eval_string_env_only(ctx.expr_engine.vars())
            .map_err(|e| {
                crate::errors::SourceError::SqlReader(format!("Failed to evaluate: {e}"))
            })?;
        let sql = self
            .param
            .sql
            .eval_string_env_only(ctx.expr_engine.vars())
            .map_err(|e| {
                crate::errors::SourceError::SqlReader(format!("Failed to evaluate: {e}"))
            })?;
        let adapter = SqlAdapter::new(database_url, 10).await.map_err(|e| {
            crate::errors::SourceError::SqlReader(format!("Failed to create adapter: {e}"))
        })?;
        let result = adapter
            .fetch_many(sql.as_str())
            .await
            .map_err(|e| crate::errors::SourceError::SqlReader(format!("Failed to fetch: {e}")))?;
        let features = result
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<Feature>, _>>()?;
        for feature in features {
            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(crate::errors::SourceError::sql_reader)?;
        }
        Ok(())
    }
}
