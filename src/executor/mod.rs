use std::sync::{Arc, Mutex};

use crate::constructs::TableDefinition;
use crate::{
    constructs::{DataInstance, DataInstanceRaw},
    sql::Statement,
    storage::Row,
};
use log::debug;
use tokio::sync::{mpsc, oneshot};

const MAX_IN_FLIGHT_REQUESTS: usize = 100;

#[derive(Debug)]
pub struct QueryResult {
    pub column_names: Vec<String>,
    pub rows: Vec<Row>,
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

    pub async fn run(&mut self) {
        while let Some(payload) = self
            .rx
            .take()
            .expect("executor.prepare_channel() must be ran before executor.run()")
            .recv()
            .await
        {
            let (statement, tx) = payload;
            debug!("☄️ Executing statement: {:?}", statement);
            // TODO: Implement real query execution
            let result = QueryResult {
                column_names: vec!["id".to_string()],
                rows: vec![Row(vec![DataInstance::Direct(DataInstanceRaw::UInt64(1))])],
            };
            tx.send(result).unwrap();
        }
    }
}
