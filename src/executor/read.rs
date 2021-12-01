use crate::{storage::{Row, filesystem::seek_read_decode_page, paging::Page}, config, constructs::components::TableDefinition};

pub async fn read_all_rows(
    config: &config::Config,
    schema: &str,
    table_definition: &TableDefinition,
) -> Result<Vec<Row>, String> {
    let meta = seek_read_decode_page(&config, &schema, &table_definition, 0).await.unwrap();
    match meta {
        Page::Meta { b_tree_root_page_index, ..} => {
            let data =  seek_read_decode_page(&config, &schema, &table_definition, b_tree_root_page_index).await.unwrap();
            match data {
                Page::BTreeLeaf { rows, ..} => Ok(rows),
                _ => Err("Invalid page type 1".to_string()),
            }
        },
        _ => {
            Err("Invalid page type 0".to_string())
        }
    }
}

#[cfg(test)]
mod read_tests {
    use crate::{storage::{paging::construct_blank_table, filesystem::write_table_file}, constructs::components::{ColumnDefinition, DataType, DataTypeRaw}};

    use super::*;
    use pretty_assertions::assert_eq;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    fn get_test_table() -> TableDefinition {
        TableDefinition::new(
            "tables".into(),
            vec![
                ColumnDefinition {
                    name: "id".into(),
                    data_type: DataType {
                        raw_type: DataTypeRaw::Uuid,
                        is_nullable: false,
                    },
                    primary_key: true,
                    default: None,
                },
                ColumnDefinition {
                    name: "table_name".into(),
                    data_type: DataType {
                        raw_type: DataTypeRaw::String,
                        is_nullable: false,
                    },
                    primary_key: false,
                    default: None,
                },
            ],
        )
    }

    #[tokio::test]
    async fn read_all_rows_empty() {
        let config = config::Config {
            data_directory: env!("TMPDIR").to_string(),
            ..Default::default()
        };
        let schema = "test";
        let data = construct_blank_table();
        let test_table = get_test_table();
        write_table_file(&config, schema, &test_table.name, data)
            .await
            .unwrap();
        let rows = read_all_rows(&config, schema, &test_table).await.unwrap();
        assert_eq!(rows.len(), 0);
    }
}
