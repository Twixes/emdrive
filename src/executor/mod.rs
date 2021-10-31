use parking_lot::Mutex;
use std::sync::Arc;

use crate::constructs::TableDefinition;
use crate::{
    constructs::{DataInstance, DataInstanceRaw},
    sql::Statement,
    storage::{NamedRow, Row},
};
use serde::{ser::SerializeSeq, Serialize, Serializer};
use tokio::sync::{mpsc, oneshot};
use tracing::debug;

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

pub type ExecutorPayload = (Statement, oneshot::Sender<QueryResult>);

pub struct Executor {
    tables: Arc<Mutex<Vec<TableDefinition>>>,
    rx: Option<mpsc::Receiver<ExecutorPayload>>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tables: Arc::new(Mutex::new(Vec::new())),
            rx: None,
        }
    }

    pub fn prepare_channel(&mut self) -> mpsc::Sender<ExecutorPayload> {
        let (tx, rx) = mpsc::channel(MAX_IN_FLIGHT_REQUESTS);
        self.rx = Some(rx);
        tx
    }

    pub async fn start(&mut self) {
        let mut rx = self
            .rx
            .take()
            .expect("`prepare_channel` must be ran before `start`");
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
    }
}
