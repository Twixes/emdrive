use parking_lot::Mutex;
use std::io;
use std::sync::Arc;

use crate::config;
use crate::storage::filesystem::{
    does_table_file_exist, seek_read_decode_page, seek_write_page, write_table_file,
};
use crate::storage::paging::{construct_blank_table, Page};
use crate::storage::system::{SystemTable, SYSTEM_SCHEMA_NAME};
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

async fn ensure_table_file_exists(
    config: &config::Config,
    table_definition: &TableDefinition,
) -> io::Result<()> {
    if !does_table_file_exist(&config, SYSTEM_SCHEMA_NAME, &table_definition.name).await {
        let blank_table_blob = construct_blank_table();
        match write_table_file(
            &config,
            SYSTEM_SCHEMA_NAME,
            &table_definition.name,
            blank_table_blob,
        )
        .await
        {
            Ok(_) => debug!("Initialized system table `{}`", table_definition.name),
            Err(error) => {
                trace!(
                    "Failed to initialize system table `{}`: {}",
                    table_definition.name,
                    error
                );
                return Err(error);
            }
        }
    }
    Ok(())
}

async fn b_tree_insert(
    config: &config::Config,
    schema: &str,
    table_definition: &TableDefinition,
    row: Row,
) -> Result<(), String> {
    let b_tree_root_page_index =
        match seek_read_decode_page(config, schema, table_definition, 0).await? {
            Page::Meta {
                b_tree_root_page_index,
                ..
            } => b_tree_root_page_index,
            _ => panic!(
                "Found a non-meta page at the beginning of table {}.{}'s data file ",
                schema, table_definition.name
            ),
        };
    let mut b_tree_root_page =
        seek_read_decode_page(config, schema, table_definition, b_tree_root_page_index).await?;
    match b_tree_root_page {
        Page::BTreeLeaf { ref mut rows, .. } => rows.push(row),
        _ => panic!(
            "Found a non-B-tree page at `b_tree_root_page_index` of table {}.{}'s data file",
            schema, table_definition.name
        ),
    };
    seek_write_page(
        config,
        schema,
        &table_definition.name,
        b_tree_root_page_index,
        b_tree_root_page.into(),
    )
    .await
    .unwrap();
    Ok(())
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
        for table in SystemTable::ALL {
            let table_definition = table.get_definition();
            ensure_table_file_exists(&self.config, &table_definition).await?;
        }
        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), io::Error> {
        let mut rx = self
            .rx
            .take()
            .expect("`prepare_channel` must be ran before `start`");
        debug!("🔢 Bootstraping the executor...");
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
