use super::encoding::WriteBlob;
use super::paging::PAGE_SIZE;
use crate::config;
use crate::storage::encoding::PageIndex;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

fn determine_table_dir_path(config: &config::Config, schema: &str, table_name: &str) -> PathBuf {
    Path::new(&config.data_directory) // $EMDRIVE_DATA_DIRECTORY
        .join(schema) // <$EMDRIVE_DATA_DIRECTORY>/<schema>
        .join(table_name) // <$EMDRIVE_DATA_DIRECTORY>/<schema>/<table_name>
}

pub async fn does_table_file_exist(
    config: &config::Config,
    schema: &str,
    table_name: &str,
) -> bool {
    let path = determine_table_dir_path(config, schema, table_name).join("0");
    match fs::metadata(path).await {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}

pub async fn write_table_file(
    config: &config::Config,
    schema: &str,
    table_name: &str,
    data: WriteBlob,
) -> Result<(), std::io::Error> {
    let dir_path = determine_table_dir_path(config, schema, table_name);
    fs::create_dir_all(&dir_path).await?;
    let file_path = dir_path.join("0");
    fs::write(file_path, data).await
}

pub async fn read_seek_page(
    config: &config::Config,
    schema: &str,
    table_name: &str,
    page_index: PageIndex,
) -> Result<Vec<u8>, std::io::Error> {
    let path = determine_table_dir_path(config, schema, table_name).join("0");
    let mut file = fs::File::open(path).await?;
    file.seek(SeekFrom::Start(page_index as u64 * PAGE_SIZE as u64))
        .await?;
    let mut buffer = Vec::with_capacity(PAGE_SIZE);
    file.read_buf(&mut buffer).await?;
    Ok(buffer)
}

pub async fn write_seek_page(
    config: &config::Config,
    schema: &str,
    table_name: &str,
    page_index: PageIndex,
    data: WriteBlob,
) -> Result<(), std::io::Error> {
    let path = determine_table_dir_path(config, schema, table_name).join("0");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .await?;
    file.seek(SeekFrom::Start(page_index as u64 * PAGE_SIZE as u64))
        .await?;
    file.write(&data).await?;
    Ok(())
}

#[cfg(test)]
mod filesystem_tests {
    use super::*;
    use crate::{
        constructs::{DataInstance, DataInstanceRaw},
        storage::{
            encoding::EncodableWithAssumption,
            paging::{construct_blank_table, Page},
            system::SystemTable,
            Row,
        },
    };
    use pretty_assertions::assert_eq;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    #[tokio::test]
    async fn raw_write_read_works() {
        let config = config::Config {
            data_directory: env!("TMPDIR").to_string(),
            ..Default::default()
        };
        let schema = "test";
        let table_name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        write_table_file(&config, schema, &table_name, data.clone())
            .await
            .unwrap();
        let read_data = read_seek_page(&config, schema, &table_name, 0)
            .await
            .unwrap();
        assert_eq!(data, read_data);
    }

    #[tokio::test]
    async fn blank_table_write_read_works() {
        let config = config::Config {
            data_directory: env!("TMPDIR").to_string(),
            ..Default::default()
        };
        let schema = "test";
        let table_name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let data = construct_blank_table();
        write_table_file(&config, schema, &table_name, data)
            .await
            .unwrap();
        let read_data_0 = read_seek_page(&config, schema, &table_name, 0)
            .await
            .unwrap();
        let (page_0, _rest) =
            Page::try_decode_assume(&read_data_0, SystemTable::Tables.get_definition()).unwrap();
        assert_eq!(
            page_0,
            Page::Meta {
                layout_version: 0,
                b_tree_root_page_index: 1
            }
        );
        let read_data_1 = read_seek_page(&config, schema, &table_name, 1)
            .await
            .unwrap();
        let (page_1, _rest) =
            Page::try_decode_assume(&read_data_1, SystemTable::Tables.get_definition()).unwrap();
        assert_eq!(
            page_1,
            Page::BTreeLeaf {
                next_leaf_page_index: 0,
                rows: vec![]
            }
        );
    }

    #[tokio::test]
    async fn page_write_works() {
        let config = config::Config {
            data_directory: env!("TMPDIR").to_string(),
            ..Default::default()
        };
        let schema = "test";
        let table_name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let data = construct_blank_table();
        write_table_file(&config, schema, &table_name, data)
            .await
            .unwrap();
        let page = Page::BTreeLeaf {
            next_leaf_page_index: 0,
            rows: vec![
                Row(vec![
                    DataInstance::Direct(DataInstanceRaw::Uuid(1)),
                    DataInstance::Direct(DataInstanceRaw::String("xyz".into())),
                ]),
                Row(vec![
                    DataInstance::Direct(DataInstanceRaw::Uuid(888)),
                    DataInstance::Direct(DataInstanceRaw::String("abc".into())),
                ]),
            ],
        };
        write_seek_page(&config, schema, &table_name, 1, page.clone().into())
            .await
            .unwrap();
        let read_data = read_seek_page(&config, schema, &table_name, 1)
            .await
            .unwrap();
        let (decoded_page, _rest) =
            Page::try_decode_assume(&read_data, SystemTable::Tables.get_definition()).unwrap();
        assert_eq!(page, decoded_page);
    }
}
