use std::path::Path;
use tokio::fs;

use crate::config;

use super::encoding::WriteBlob;

pub async fn does_table_file_exist(
    config: &config::Config,
    schema: &str,
    table_name: &str,
) -> bool {
    let path = Path::new(&config.data_directory) // $EMDRIVE_DATA_DIRECTORY
        .join(schema) // <$EMDRIVE_DATA_DIRECTORY>/<schema>
        .join(table_name) // <$EMDRIVE_DATA_DIRECTORY>/<schema>/<table_name>
        .join("0"); // <$EMDRIVE_DATA_DIRECTORY>/<schema>/<table_name>/<core data file>
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
    let dir_path = Path::new(&config.data_directory)
        .join(schema)
        .join(table_name);
    fs::create_dir_all(&dir_path).await?;
    let file_path = dir_path.join("0");
    fs::write(file_path, data).await
}
