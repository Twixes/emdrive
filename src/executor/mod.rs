mod read;
mod write;

use parking_lot::Mutex;
use std::io;
use std::sync::Arc;

use crate::config;
use crate::storage::system::SystemTable;
use crate::{
    constructs::components::{DataInstance, DataInstanceRaw, TableDefinition},
    sql::Statement,
    storage::{NamedRow, Row},
};
use serde::{ser::SerializeSeq, Serialize, Serializer};
use tokio::sync::{mpsc, oneshot};
use tracing::*;

const MAX_IN_FLIGHT_REQUESTS: usize = 100;

#[derive(Debug)]
pub struct QueryResult {
    pub column_names: Vec<String>,
    pub rows: Vec<Row>,
}

impl Serialize for QueryResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rows.len()))?;
        for row in &self.rows {
            seq.serialize_element(&NamedRow(&self.column_names, &row.0))?;
        }
        seq.end()
    }
}

/// Payload with a statement and a sender to return the result to.
pub type ExecutorPayload = (Statement, oneshot::Sender<QueryResult>);

pub struct Executor {
    config: config::Config,
    tables: Arc<Mutex<Vec<TableDefinition>>>,
    rx: Option<mpsc::Receiver<ExecutorPayload>>,
}

impl Executor {
    pub fn new(config: &config::Config) -> Self {
        Executor {
            config: config.clone(),
            tables: Arc::new(Mutex::new(Vec::new())),
            rx: None,
        }
    }

    pub fn prepare_channel(&mut self) -> mpsc::Sender<ExecutorPayload> {
        let (tx, rx) = mpsc::channel(MAX_IN_FLIGHT_REQUESTS);
        self.rx = Some(rx);
        tx
    }

    pub async fn bootstrap(&mut self) -> Result<(), io::Error> {
        debug!("⬆️ Bootstraping the executor...");
        for table in SystemTable::ALL {
            let table_definition = table.get_definition();
            write::ensure_table_file_exists(&self.config, &table_definition).await?;
        }
        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), io::Error> {
        let mut rx = self
            .rx
            .take()
            .expect("`prepare_channel` must be ran before `start`");
        self.bootstrap().await?;
        debug!("🗡 Executor engaged");
        while let Some(payload) = rx.recv().await {
            let (statement, tx) = payload;
            debug!("➡️ Executing statement: {:?}", statement);
            // TODO: Implement real query execution
            let result = QueryResult {
                column_names: vec!["id".to_string()],
                rows: vec![Row(vec![DataInstance::Direct(DataInstanceRaw::UInt64(1))])],
            };
            tx.send(result).unwrap();
        }
        debug!("🎗 Executor disengaged");
        Ok(())
    }
}
