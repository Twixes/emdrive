use std::io;

use crate::config;
use crate::storage::filesystem::{
    does_table_file_exist, seek_read_decode_page, seek_write_page, write_table_file,
};
use crate::storage::paging::{construct_blank_table, Page};
use crate::storage::system::SYSTEM_SCHEMA_NAME;
use crate::{constructs::components::TableDefinition, storage::Row};
use tracing::*;

pub async fn ensure_table_file_exists(
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

pub async fn b_tree_insert(
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
